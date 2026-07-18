import { timingSafeEqual } from "node:crypto";
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";

import { SettlementCoordinator } from "./settlement.js";
import type {
  SettlementExecutionVenue,
  SettlementExecutorRequest,
  SettlementExecutorResponse,
  SettlementObservationProvider,
  SettlementProgramClient,
} from "./types.js";

const MAX_BODY_BYTES = 1_048_576;

export type SettlementExecutorDependencies = {
  program: SettlementProgramClient;
  venue: SettlementExecutionVenue;
  observations: SettlementObservationProvider;
  bearerToken: string;
  expected: {
    programId: string;
    statePda: string;
    pexMintAddress: string;
    solanaRpcUrl?: string;
  };
  isTerminalError?: (error: unknown) => boolean;
};

export function createSettlementExecutorServer(
  dependencies: SettlementExecutorDependencies,
) {
  const token = dependencies.bearerToken.trim();
  if (token.length < 24) {
    throw new Error("Settlement executor bearer token must be at least 24 characters");
  }

  const coordinator = new SettlementCoordinator(
    dependencies.program,
    dependencies.venue,
  );

  return createServer(async (request, response) => {
    try {
      if (request.method === "GET" && request.url === "/healthz") {
        sendJson(response, 200, {
          ok: true,
          service: "perax-settlement-executor",
        });
        return;
      }
      if (request.method !== "POST" || request.url !== "/execute/settlement") {
        sendJson(response, 404, { error: "not found" });
        return;
      }
      if (!authorized(request, token)) {
        sendJson(response, 401, { error: "unauthorized" });
        return;
      }

      const payload = validateRequest(await readJsonBody(request));
      validateConfiguredIdentifiers(payload, dependencies.expected);
      const observationId = await dependencies.observations.getFreshObservationId();
      if (observationId.length !== 32) {
        throw new Error("Observation provider returned an invalid observation ID");
      }

      const settlement = await coordinator.execute({
        settlementId: hexTo32Bytes(payload.settlementIdHex, "settlementIdHex"),
        productId: hexTo32Bytes(payload.productIdHex, "productIdHex"),
        observationId,
        fundingMethod: payload.fundingMethod,
        quantity: BigInt(payload.quantity),
        beneficiary: payload.beneficiaryWallet,
      });

      if (settlement.status !== "finalized") {
        throw new Error(
          `Settlement coordinator returned non-final status: ${settlement.status}`,
        );
      }
      if (!settlement.settlementRecordAddress || !settlement.transactionSignature) {
        throw new Error(
          "Finalized program result is missing settlement record address or transaction signature",
        );
      }

      const result: SettlementExecutorResponse = {
        status: "finalized",
        terminalFailure: false,
        settlementRecordAddress: settlement.settlementRecordAddress,
        transactionSignature: settlement.transactionSignature,
      };
      sendJson(response, 200, result);
    } catch (error) {
      const terminalFailure = dependencies.isTerminalError?.(error) ?? false;
      const result: SettlementExecutorResponse = {
        status: "failed",
        terminalFailure,
        error: errorMessage(error),
      };
      sendJson(response, terminalFailure ? 422 : 503, result);
    }
  });
}

function validateConfiguredIdentifiers(
  payload: SettlementExecutorRequest,
  expected: SettlementExecutorDependencies["expected"],
): void {
  if (payload.programId !== expected.programId) {
    throw new Error("programId does not match executor configuration");
  }
  if (payload.statePda !== expected.statePda) {
    throw new Error("statePda does not match executor configuration");
  }
  if (payload.pexMintAddress !== expected.pexMintAddress) {
    throw new Error("pexMintAddress does not match executor configuration");
  }
  if (expected.solanaRpcUrl && payload.solanaRpcUrl !== expected.solanaRpcUrl) {
    throw new Error("solanaRpcUrl does not match executor configuration");
  }
}

function validateRequest(value: unknown): SettlementExecutorRequest {
  if (!isObject(value)) {
    throw new Error("Request body must be a JSON object");
  }

  const fundingMethod = value.fundingMethod;
  if (
    fundingMethod !== "pex" &&
    fundingMethod !== "stablecoin" &&
    fundingMethod !== "fiat" &&
    fundingMethod !== "virtualAccount"
  ) {
    throw new Error("fundingMethod is invalid");
  }

  return {
    solanaRpcUrl: requiredString(value.solanaRpcUrl, "solanaRpcUrl"),
    programId: requiredString(value.programId, "programId"),
    statePda: requiredString(value.statePda, "statePda"),
    pexMintAddress: requiredString(value.pexMintAddress, "pexMintAddress"),
    orderReference: requiredString(value.orderReference, "orderReference"),
    settlementIdHex: requiredHex(value.settlementIdHex, "settlementIdHex"),
    productIdHex: requiredHex(value.productIdHex, "productIdHex"),
    fundingMethod,
    quantity: positiveSafeInteger(value.quantity, "quantity"),
    beneficiaryWallet: requiredString(
      value.beneficiaryWallet,
      "beneficiaryWallet",
    ),
    previousStatus: requiredString(value.previousStatus, "previousStatus"),
    attempt: nonNegativeInteger(value.attempt, "attempt"),
  };
}

function authorized(request: IncomingMessage, expectedToken: string): boolean {
  const header = request.headers.authorization;
  if (!header?.startsWith("Bearer ")) {
    return false;
  }
  const supplied = Buffer.from(header.slice(7).trim());
  const expected = Buffer.from(expectedToken);
  return supplied.length === expected.length && timingSafeEqual(supplied, expected);
}

async function readJsonBody(request: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  let total = 0;
  for await (const chunk of request) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    total += buffer.length;
    if (total > MAX_BODY_BYTES) {
      throw new Error("Request body exceeds the one-megabyte limit");
    }
    chunks.push(buffer);
  }
  const text = Buffer.concat(chunks).toString("utf8");
  if (!text) {
    throw new Error("Request body is empty");
  }
  return JSON.parse(text) as unknown;
}

function sendJson(
  response: ServerResponse,
  statusCode: number,
  body: unknown,
): void {
  const encoded = JSON.stringify(body);
  response.writeHead(statusCode, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(encoded),
  });
  response.end(encoded);
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} is required`);
  }
  return value.trim();
}

function requiredHex(value: unknown, label: string): string {
  const normalized = requiredString(value, label)
    .replace(/^0x/i, "")
    .toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(normalized)) {
    throw new Error(`${label} must contain exactly 32 bytes of hexadecimal data`);
  }
  return normalized;
}

function positiveSafeInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${label} must be a positive safe integer`);
  }
  return value;
}

function nonNegativeInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
  return value;
}

function hexTo32Bytes(value: string, label: string): Uint8Array {
  const normalized = requiredHex(value, label);
  return Uint8Array.from(Buffer.from(normalized, "hex"));
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
