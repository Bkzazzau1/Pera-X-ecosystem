import { readFile } from "node:fs/promises";
import { isAbsolute, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import anchor, { AnchorProvider, web3 } from "@coral-xyz/anchor";
import type { Commitment } from "@solana/web3.js";

import { createAnchorSettlementProgramClient } from "./anchor-client.js";
import { createSettlementExecutorServer } from "./executor.js";
import { loadSettlementIdl } from "./idl.js";
import {
  assertSettlementRuntimeBindings,
  type SettlementRuntimeModule,
} from "./runtime.js";

export type SettlementExecutorEnvironment = NodeJS.ProcessEnv;

export async function startSettlementExecutorFromEnvironment(
  environment: SettlementExecutorEnvironment = process.env,
) {
  const rpcUrl = requiredEnvironment(environment, "SOLANA_RPC_URL");
  const programIdText = requiredEnvironment(environment, "PERAX_PROGRAM_ID");
  const statePdaText = requiredEnvironment(environment, "PERAX_STATE_PDA");
  const pexMintText = requiredEnvironment(environment, "PEX_MINT_ADDRESS");
  const idlPath = requiredEnvironment(environment, "PERAX_SETTLEMENT_IDL_PATH");
  const signerPath = requiredEnvironment(
    environment,
    "PERAX_SETTLEMENT_SIGNER_PATH",
  );
  const runtimeModulePath = requiredEnvironment(
    environment,
    "PERAX_SETTLEMENT_RUNTIME_MODULE",
  );
  const bearerToken = requiredEnvironment(
    environment,
    "PERAX_SETTLEMENT_EXECUTOR_TOKEN",
  );
  if (bearerToken.length < 24) {
    throw new Error(
      "PERAX_SETTLEMENT_EXECUTOR_TOKEN must contain at least 24 characters",
    );
  }

  const commitment = parseCommitment(
    environment.PERAX_SETTLEMENT_COMMITMENT,
  );
  const programId = new web3.PublicKey(programIdText);
  const statePda = new web3.PublicKey(statePdaText);
  const pexMint = new web3.PublicKey(pexMintText);
  const signer = await loadSettlementKeypair(signerPath);
  const wallet = new anchor.Wallet(signer);
  const connection = new web3.Connection(rpcUrl, commitment);
  const provider = new AnchorProvider(connection, wallet, {
    commitment,
    preflightCommitment: commitment,
  });
  const idl = await loadSettlementIdl(idlPath, programId.toBase58());
  const runtimeModule = await loadSettlementRuntimeModule(runtimeModulePath);
  const runtime = assertSettlementRuntimeBindings(
    await runtimeModule.createSettlementRuntime({
      provider,
      programId,
      statePda,
      pexMint,
      idl,
    }),
  );

  const program = createAnchorSettlementProgramClient({
    provider,
    idl,
    expectedProgramId: programId.toBase58(),
    expectedStatePda: statePda.toBase58(),
    pexMint,
    commitment,
    resolveQuoteSource: runtime.resolveQuoteSource,
    ...(runtime.resolveDirectPexSource
      ? { resolveDirectPexSource: runtime.resolveDirectPexSource }
      : {}),
    ...(runtime.resolveCustomerDestination
      ? { resolveCustomerDestination: runtime.resolveCustomerDestination }
      : {}),
  });
  const server = createSettlementExecutorServer({
    program,
    venue: runtime.venue,
    observations: runtime.observations,
    bearerToken,
    expected: {
      programId: programId.toBase58(),
      statePda: statePda.toBase58(),
      pexMintAddress: pexMint.toBase58(),
      solanaRpcUrl: rpcUrl,
    },
    ...(runtime.isTerminalError
      ? { isTerminalError: runtime.isTerminalError }
      : {}),
  });

  const host = environment.PERAX_SETTLEMENT_EXECUTOR_HOST?.trim() || "127.0.0.1";
  const port = positivePort(environment.PERAX_SETTLEMENT_EXECUTOR_PORT);
  await new Promise<void>((resolveListen, reject) => {
    const onError = (error: Error) => {
      server.off("listening", onListening);
      reject(error);
    };
    const onListening = () => {
      server.off("error", onError);
      resolveListen();
    };
    server.once("error", onError);
    server.once("listening", onListening);
    server.listen(port, host);
  });

  const address = server.address();
  const printable =
    typeof address === "object" && address
      ? `${address.address}:${address.port}`
      : String(address);
  console.info(`Pera-X settlement executor listening on ${printable}`);
  return server;
}

export async function loadSettlementKeypair(
  filePath: string,
): Promise<web3.Keypair> {
  const content = await readFile(filePath, "utf8");
  const parsed = JSON.parse(content) as unknown;
  if (
    !Array.isArray(parsed) ||
    parsed.length !== 64 ||
    !parsed.every(
      (value) => Number.isInteger(value) && value >= 0 && value <= 255,
    )
  ) {
    throw new Error("Settlement signer file must contain a 64-byte keypair array");
  }
  return web3.Keypair.fromSecretKey(Uint8Array.from(parsed as number[]));
}

export async function loadSettlementRuntimeModule(
  modulePath: string,
): Promise<SettlementRuntimeModule> {
  const url = runtimeModuleUrl(modulePath);
  const imported = (await import(url)) as Partial<SettlementRuntimeModule>;
  if (typeof imported.createSettlementRuntime !== "function") {
    throw new Error(
      "Settlement runtime module must export createSettlementRuntime(context)",
    );
  }
  return imported as SettlementRuntimeModule;
}

export function runtimeModuleUrl(modulePath: string): string {
  const value = modulePath.trim();
  if (!value) {
    throw new Error("Settlement runtime module path is required");
  }
  if (value.startsWith("file:")) {
    return value;
  }
  const filePath = isAbsolute(value) ? value : resolve(process.cwd(), value);
  return pathToFileURL(filePath).href;
}

export function requiredEnvironment(
  environment: SettlementExecutorEnvironment,
  name: string,
): string {
  const value = environment[name]?.trim();
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function parseCommitment(value: string | undefined): Commitment {
  const normalized = value?.trim().toLowerCase() || "confirmed";
  if (
    normalized !== "processed" &&
    normalized !== "confirmed" &&
    normalized !== "finalized"
  ) {
    throw new Error(
      "PERAX_SETTLEMENT_COMMITMENT must be processed, confirmed, or finalized",
    );
  }
  return normalized;
}

function positivePort(value: string | undefined): number {
  const port = value?.trim() ? Number(value) : 8788;
  if (!Number.isSafeInteger(port) || port <= 0 || port > 65_535) {
    throw new Error("PERAX_SETTLEMENT_EXECUTOR_PORT must be a valid TCP port");
  }
  return port;
}

const invokedPath = process.argv[1]
  ? pathToFileURL(resolve(process.argv[1])).href
  : undefined;
if (invokedPath === import.meta.url) {
  startSettlementExecutorFromEnvironment().catch((error: unknown) => {
    console.error(
      "Pera-X settlement executor failed to start:",
      error instanceof Error ? error.message : String(error),
    );
    process.exitCode = 1;
  });
}
