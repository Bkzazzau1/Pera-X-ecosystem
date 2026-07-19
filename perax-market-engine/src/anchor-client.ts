import anchor, {
  AnchorProvider,
  Program,
  type Idl,
  web3,
} from "@coral-xyz/anchor";
const { BN } = anchor;
type AnchorBn = InstanceType<typeof BN>;
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import type {
  AccountMeta,
  Commitment,
  PublicKey,
  Signer,
} from "@solana/web3.js";

import { assertSettlementIdlCompatible, type SettlementIdl } from "./idl.js";
import type {
  AtomicMarketPurchase,
  SettlementDisposition,
  SettlementFundingMethod,
  SettlementMarketMode,
  SettlementPlanInput,
  SettlementProgramClient,
  SettlementRecordView,
  SettlementStatus,
} from "./types.js";

const U64_MAX = (1n << 64n) - 1n;
const UTF8 = new TextEncoder();

type DynamicRecord = Record<string, unknown>;
type AccountClient = {
  fetch(address: PublicKey): Promise<DynamicRecord>;
  fetchNullable?: (address: PublicKey) => Promise<DynamicRecord | null>;
};
type MethodBuilder = {
  accountsStrict(accounts: Record<string, PublicKey>): MethodBuilder;
  remainingAccounts(accounts: AccountMeta[]): MethodBuilder;
  signers(signers: Signer[]): MethodBuilder;
  rpc(options?: {
    commitment?: Commitment;
    preflightCommitment?: Commitment;
  }): Promise<string>;
};
type DynamicProgram = {
  programId: PublicKey;
  methods: Record<string, (params: unknown) => MethodBuilder>;
  account: Record<string, AccountClient>;
};

export type SettlementTokenSource = {
  authority: PublicKey;
  tokenAccount: PublicKey;
  signers?: Signer[];
};

export type AnchorSettlementProgramClientConfig = {
  provider: AnchorProvider;
  idl: SettlementIdl;
  expectedProgramId: string;
  expectedStatePda?: string;
  pexMint: PublicKey | string;
  commitment?: Commitment;
  program?: Program<Idl>;
  resolveDirectPexSource?: (
    settlement: SettlementRecordView,
    amount: bigint,
  ) => Promise<SettlementTokenSource> | SettlementTokenSource;
  resolveQuoteSource?: (
    settlement: SettlementRecordView,
    purchase: AtomicMarketPurchase,
  ) => Promise<SettlementTokenSource> | SettlementTokenSource;
  resolveCustomerDestination?: (
    beneficiary: PublicKey,
    pexMint: PublicKey,
    settlement: SettlementRecordView,
  ) => Promise<PublicKey> | PublicKey;
};

export class AnchorSettlementProgramClient implements SettlementProgramClient {
  readonly programId: PublicKey;
  readonly statePda: PublicKey;
  readonly settlementPolicyPda: PublicKey;

  private readonly program: DynamicProgram;
  private readonly pexMint: PublicKey;
  private readonly commitment: Commitment;

  constructor(private readonly config: AnchorSettlementProgramClientConfig) {
    const expectedProgramId = new web3.PublicKey(config.expectedProgramId);
    const compatibleIdl = assertSettlementIdlCompatible(
      config.idl,
      expectedProgramId.toBase58(),
    );
    const anchorProgram = (config.program ??
      new Program(compatibleIdl as Idl, config.provider)) as unknown as DynamicProgram;
    if (!anchorProgram.programId.equals(expectedProgramId)) {
      throw new Error("Anchor program ID does not match the validated settlement IDL");
    }

    this.program = anchorProgram;
    this.programId = expectedProgramId;
    this.pexMint = toPublicKey(config.pexMint);
    this.commitment = config.commitment ?? "confirmed";
    this.statePda = derivePda(this.programId, "perax-state");
    this.settlementPolicyPda = derivePda(
      this.programId,
      "settlement-policy",
      this.statePda.toBuffer(),
    );

    if (
      config.expectedStatePda &&
      this.statePda.toBase58() !== new web3.PublicKey(config.expectedStatePda).toBase58()
    ) {
      throw new Error("Configured Pera-X state PDA does not match program-derived state PDA");
    }
  }

  async planSettlement(input: SettlementPlanInput): Promise<SettlementRecordView> {
    assertBytes32(input.settlementId, "settlementId");
    assertBytes32(input.productId, "productId");
    assertBytes32(input.observationId, "observationId");
    const quantity = toU64(input.quantity, "quantity");
    const beneficiary = new web3.PublicKey(input.beneficiary);
    const addresses = this.deriveSettlementAddresses(
      input.settlementId,
      input.productId,
      input.observationId,
    );

    const existing = await this.fetchNullable("settlementRecord", addresses.settlementRecord);
    if (existing) {
      this.assertExistingPlanMatches(existing, input, beneficiary);
      return this.recordView(addresses.settlementRecord, existing);
    }

    const method = this.method("planSettlement")({
      settlementId: Array.from(input.settlementId),
      productId: Array.from(input.productId),
      observationId: Array.from(input.observationId),
      fundingMethod: anchorEnum(input.fundingMethod),
      quantity,
      beneficiary,
    });
    const signature = await this.send(
      method.accountsStrict({
        state: this.statePda,
        settlementPolicy: this.settlementPolicyPda,
        productPolicy: addresses.productPolicy,
        apcConfig: addresses.apcConfig,
        apcState: addresses.apcState,
        observation: addresses.observation,
        settlementRecord: addresses.settlementRecord,
        settlementCustody: addresses.settlementCustody,
        settlementAuthority: addresses.settlementAuthority,
        settlementPexVault: addresses.settlementPexVault,
        pexMint: this.pexMint,
        initiator: this.config.provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      }),
    );
    return this.readSettlement(input.settlementId, signature);
  }

  async fundDirectPex(
    settlementId: Uint8Array,
    amount: bigint,
  ): Promise<SettlementRecordView> {
    assertBytes32(settlementId, "settlementId");
    const amountBn = toU64(amount, "direct PEX amount");
    const current = await this.readSettlement(settlementId);
    if (current.status === "ready" || current.status === "finalized") {
      return current;
    }
    if (!this.config.resolveDirectPexSource) {
      throw new Error(
        "Direct PEX settlement requires a configured source authority and token account",
      );
    }
    const source = await this.config.resolveDirectPexSource(current, amount);
    const addresses = this.deriveSettlementAddresses(
      settlementId,
      requiredBytes(current.productId, "productId"),
      requiredBytes(current.observationId, "observationId"),
    );
    const method = this.method("fundDirectPexSettlement")({
      settlementId: Array.from(settlementId),
      amount: amountBn,
    });
    const signature = await this.send(
      method.accountsStrict({
        state: this.statePda,
        settlementPolicy: this.settlementPolicyPda,
        settlementRecord: addresses.settlementRecord,
        settlementCustody: addresses.settlementCustody,
        sourceAuthority: source.authority,
        sourceTokenAccount: source.tokenAccount,
        settlementPexVault: addresses.settlementPexVault,
        pexMint: this.pexMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      }),
      source.signers,
    );
    return this.readSettlement(settlementId, signature);
  }

  async executeMarketPurchase(
    settlementId: Uint8Array,
    purchase: AtomicMarketPurchase,
  ): Promise<SettlementRecordView> {
    assertBytes32(settlementId, "settlementId");
    const current = await this.readSettlement(settlementId);
    if (current.status === "ready" || current.status === "finalized") {
      return current;
    }
    if (!this.config.resolveQuoteSource) {
      throw new Error(
        "Market settlement requires a configured quote-token source authority and account",
      );
    }
    const source = await this.config.resolveQuoteSource(current, purchase);
    const policy = await this.fetch("settlementPolicy", this.settlementPolicyPda);
    const addresses = this.deriveSettlementAddresses(
      settlementId,
      requiredBytes(current.productId, "productId"),
      requiredBytes(current.observationId, "observationId"),
    );
    const remainingAccounts = (purchase.remainingAccounts ?? []).map((account) => ({
      pubkey: new web3.PublicKey(account.publicKey),
      isWritable: account.isWritable,
      isSigner: account.isSigner ?? false,
    }));
    this.assertRemainingSignersAvailable(remainingAccounts, source.signers);

    let method = this.method("executeSettlementMarketPurchase")({
      settlementId: Array.from(settlementId),
      maximumQuoteAmount: toU64(purchase.maximumQuoteAmount, "maximumQuoteAmount"),
      minimumPexOut: toU64(purchase.minimumPexOut, "minimumPexOut"),
      swapInstructionData: Buffer.from(purchase.instructionData),
    }).accountsStrict({
      state: this.statePda,
      settlementPolicy: this.settlementPolicyPda,
      settlementRecord: addresses.settlementRecord,
      settlementCustody: addresses.settlementCustody,
      apcConfig: addresses.apcConfig,
      observation: addresses.observation,
      quoteSourceAuthority: source.authority,
      quoteSourceTokenAccount: source.tokenAccount,
      settlementPexVault: addresses.settlementPexVault,
      quoteMint: publicKeyField(policy, "quoteMint"),
      pexMint: this.pexMint,
      approvedMarketPool: publicKeyField(policy, "approvedMarketPool"),
      marketProgram: publicKeyField(policy, "approvedMarketProgram"),
      tokenProgram: TOKEN_PROGRAM_ID,
    });
    if (remainingAccounts.length > 0) {
      method = method.remainingAccounts(remainingAccounts);
    }
    const signature = await this.send(method, source.signers);
    return this.readSettlement(settlementId, signature);
  }

  async executePolicyVaultFunding(
    settlementId: Uint8Array,
  ): Promise<SettlementRecordView> {
    assertBytes32(settlementId, "settlementId");
    const current = await this.readSettlement(settlementId);
    if (current.status === "ready" || current.status === "finalized") {
      return current;
    }
    const policy = await this.fetch("settlementPolicy", this.settlementPolicyPda);
    const reserveVaultConfig = publicKeyField(
      policy,
      "approvedPolicyVaultConfig",
    );
    const reserve = await this.fetch("reserveVaultConfig", reserveVaultConfig);
    const allocationId = bytesField(reserve, "allocationId");
    const derivedVaultAuthority = derivePda(
      this.programId,
      "reserve-authority",
      Buffer.from(allocationId),
    );
    const storedVaultAuthority = publicKeyField(reserve, "vaultAuthority");
    if (!derivedVaultAuthority.equals(storedVaultAuthority)) {
      throw new Error("Reserve vault authority does not match its policy-derived PDA");
    }

    const addresses = this.deriveSettlementAddresses(
      settlementId,
      requiredBytes(current.productId, "productId"),
      requiredBytes(current.observationId, "observationId"),
    );
    const method = this.method("executeSettlementVaultFunding")({
      settlementId: Array.from(settlementId),
    });
    const signature = await this.send(
      method.accountsStrict({
        state: this.statePda,
        settlementPolicy: this.settlementPolicyPda,
        settlementRecord: addresses.settlementRecord,
        settlementCustody: addresses.settlementCustody,
        reserveVaultConfig,
        vaultAuthority: storedVaultAuthority,
        vaultTokenAccount: publicKeyField(reserve, "vaultTokenAccount"),
        settlementPexVault: addresses.settlementPexVault,
        pexMint: this.pexMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      }),
    );
    return this.readSettlement(settlementId, signature);
  }

  async finalizeSettlement(settlementId: Uint8Array): Promise<SettlementRecordView> {
    assertBytes32(settlementId, "settlementId");
    const current = await this.readSettlement(settlementId);
    const recordAddress = new web3.PublicKey(
      requiredString(current.settlementRecordAddress, "settlementRecordAddress"),
    );
    if (current.status === "finalized") {
      const signature =
        current.transactionSignature ?? (await this.latestSignature(recordAddress));
      return { ...current, transactionSignature: signature };
    }
    if (current.status !== "ready") {
      throw new Error(`Settlement cannot finalize from status ${current.status}`);
    }

    const rawRecord = await this.fetch("settlementRecord", recordAddress);
    const policy = await this.fetch("settlementPolicy", this.settlementPolicyPda);
    const productId = bytesField(rawRecord, "productId");
    const productPolicy = derivePda(
      this.programId,
      "product-settlement",
      Buffer.from(productId),
    );
    const settlementCustody = derivePda(
      this.programId,
      "settlement-custody",
      Buffer.from(settlementId),
    );
    const settlementAuthority = derivePda(
      this.programId,
      "settlement-custody-authority",
      recordAddress.toBuffer(),
    );
    const settlementPexVault = getAssociatedTokenAddressSync(
      this.pexMint,
      settlementAuthority,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const destinationTokenAccount = await this.resolveDestination(
      current,
      rawRecord,
      policy,
    );

    const method = this.method("finalizeSettlement")({
      settlementId: Array.from(settlementId),
    });
    const signature = await this.send(
      method.accountsStrict({
        state: this.statePda,
        settlementPolicy: this.settlementPolicyPda,
        productPolicy,
        settlementRecord: recordAddress,
        settlementCustody,
        settlementAuthority,
        settlementPexVault,
        destinationTokenAccount,
        lockVault: publicKeyField(policy, "lockVault"),
        pexMint: this.pexMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      }),
    );
    return this.readSettlement(settlementId, signature);
  }

  async readSettlement(
    settlementId: Uint8Array,
    transactionSignature?: string,
  ): Promise<SettlementRecordView> {
    assertBytes32(settlementId, "settlementId");
    const settlementRecord = derivePda(
      this.programId,
      "settlement",
      Buffer.from(settlementId),
    );
    const raw = await this.fetch("settlementRecord", settlementRecord);
    const view = this.recordView(settlementRecord, raw, transactionSignature);
    if (view.status === "finalized" && !view.transactionSignature) {
      return { ...view, transactionSignature: await this.latestSignature(settlementRecord) };
    }
    return view;
  }

  private deriveSettlementAddresses(
    settlementId: Uint8Array,
    productId: Uint8Array,
    observationId: Uint8Array,
  ) {
    const apcConfig = derivePda(
      this.programId,
      "apc-config",
      this.statePda.toBuffer(),
    );
    const settlementRecord = derivePda(
      this.programId,
      "settlement",
      Buffer.from(settlementId),
    );
    const settlementAuthority = derivePda(
      this.programId,
      "settlement-custody-authority",
      settlementRecord.toBuffer(),
    );
    return {
      productPolicy: derivePda(
        this.programId,
        "product-settlement",
        Buffer.from(productId),
      ),
      apcConfig,
      apcState: derivePda(this.programId, "apc-state", apcConfig.toBuffer()),
      observation: derivePda(
        this.programId,
        "apc-observation",
        Buffer.from(observationId),
      ),
      settlementRecord,
      settlementCustody: derivePda(
        this.programId,
        "settlement-custody",
        Buffer.from(settlementId),
      ),
      settlementAuthority,
      settlementPexVault: getAssociatedTokenAddressSync(
        this.pexMint,
        settlementAuthority,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
    };
  }

  private async resolveDestination(
    current: SettlementRecordView,
    rawRecord: DynamicRecord,
    policy: DynamicRecord,
  ): Promise<PublicKey> {
    switch (current.disposition) {
      case "utilityPayment":
        return publicKeyField(rawRecord, "destinationTokenAccount");
      case "customerDelivery": {
        const beneficiary = publicKeyField(rawRecord, "beneficiary");
        if (this.config.resolveCustomerDestination) {
          return this.config.resolveCustomerDestination(
            beneficiary,
            this.pexMint,
            current,
          );
        }
        return getAssociatedTokenAddressSync(
          this.pexMint,
          beneficiary,
          true,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID,
        );
      }
      case "burn":
      case "lock":
        return publicKeyField(policy, "lockVault");
      default:
        return assertNever(current.disposition);
    }
  }

  private recordView(
    settlementRecord: PublicKey,
    raw: DynamicRecord,
    transactionSignature?: string,
  ): SettlementRecordView {
    const view: SettlementRecordView = {
      settlementId: bytesField(raw, "settlementId"),
      productId: bytesField(raw, "productId"),
      observationId: bytesField(raw, "observationId"),
      beneficiary: publicKeyField(raw, "beneficiary").toBase58(),
      destinationTokenAccount: publicKeyField(
        raw,
        "destinationTokenAccount",
      ).toBase58(),
      marketMode: enumField<SettlementMarketMode>(raw, "marketMode", [
        "directPex",
        "marketPurchase",
        "policyVault",
        "hybrid",
      ]),
      disposition: enumField<SettlementDisposition>(raw, "disposition", [
        "utilityPayment",
        "customerDelivery",
        "burn",
        "lock",
      ]),
      status: enumField<SettlementStatus>(raw, "status", [
        "planned",
        "funding",
        "ready",
        "finalized",
      ]),
      pexObligation: bigintField(raw, "pexObligation"),
      marketPexRequired: bigintField(raw, "marketPexRequired"),
      policyVaultPexRequired: bigintField(raw, "policyVaultPexRequired"),
      marketPexReceived: bigintField(raw, "marketPexReceived"),
      policyVaultPexReceived: bigintField(raw, "policyVaultPexReceived"),
      directPexReceived: bigintField(raw, "directPexReceived"),
      effectivePrice: bigintField(raw, "effectivePrice"),
      settlementRecordAddress: settlementRecord.toBase58(),
    };
    if (transactionSignature) {
      view.transactionSignature = transactionSignature;
    }
    return view;
  }

  private assertExistingPlanMatches(
    raw: DynamicRecord,
    input: SettlementPlanInput,
    beneficiary: PublicKey,
  ): void {
    const conflicts =
      !equalBytes(bytesField(raw, "settlementId"), input.settlementId) ||
      !equalBytes(bytesField(raw, "productId"), input.productId) ||
      !equalBytes(bytesField(raw, "observationId"), input.observationId) ||
      bigintField(raw, "quantity") !== input.quantity ||
      !publicKeyField(raw, "beneficiary").equals(beneficiary) ||
      enumField<SettlementFundingMethod>(raw, "fundingMethod", [
        "pex",
        "stablecoin",
        "fiat",
        "virtualAccount",
      ]) !== input.fundingMethod;
    if (conflicts) {
      throw new Error(
        "Existing settlement record conflicts with the requested immutable settlement plan",
      );
    }
  }

  private assertRemainingSignersAvailable(
    accounts: AccountMeta[],
    signers: Signer[] | undefined,
  ): void {
    const available = new Set<string>([
      this.config.provider.publicKey.toBase58(),
      ...(signers ?? []).map((signer) => signer.publicKey.toBase58()),
    ]);
    const missing = accounts
      .filter((account) => account.isSigner && !available.has(account.pubkey.toBase58()))
      .map((account) => account.pubkey.toBase58());
    if (missing.length > 0) {
      throw new Error(
        `Atomic adapter requested unavailable signer accounts: ${missing.join(", ")}`,
      );
    }
  }

  private method(name: string): (params: unknown) => MethodBuilder {
    const method = this.program.methods[name];
    if (!method) {
      throw new Error(`Validated settlement IDL does not expose method ${name}`);
    }
    return method;
  }

  private async send(builder: MethodBuilder, signers?: Signer[]): Promise<string> {
    const uniqueSigners = dedupeSigners(signers ?? []);
    let transaction = builder;
    if (uniqueSigners.length > 0) {
      transaction = transaction.signers(uniqueSigners);
    }
    const signature = await transaction.rpc({
      commitment: this.commitment,
      preflightCommitment: this.commitment,
    });
    await this.config.provider.connection.confirmTransaction(
      signature,
      this.commitment,
    );
    return signature;
  }

  private async fetch(name: string, address: PublicKey): Promise<DynamicRecord> {
    const client = this.program.account[name];
    if (!client) {
      throw new Error(`Validated settlement IDL does not expose account ${name}`);
    }
    return client.fetch(address);
  }

  private async fetchNullable(
    name: string,
    address: PublicKey,
  ): Promise<DynamicRecord | null> {
    const client = this.program.account[name];
    if (!client) {
      throw new Error(`Validated settlement IDL does not expose account ${name}`);
    }
    if (client.fetchNullable) {
      return client.fetchNullable(address);
    }
    try {
      return await client.fetch(address);
    } catch (error) {
      if (String(error).toLowerCase().includes("account does not exist")) {
        return null;
      }
      throw error;
    }
  }

  private async latestSignature(address: PublicKey): Promise<string> {
    const signatures = await this.config.provider.connection.getSignaturesForAddress(
      address,
      { limit: 1 },
      this.commitment === "finalized" ? "finalized" : "confirmed",
    );
    const signature = signatures[0]?.signature;
    if (!signature) {
      throw new Error(
        "Finalized settlement exists but its confirming transaction signature was not found",
      );
    }
    return signature;
  }
}

export function createAnchorSettlementProgramClient(
  config: AnchorSettlementProgramClientConfig,
): AnchorSettlementProgramClient {
  return new AnchorSettlementProgramClient(config);
}

function derivePda(
  programId: PublicKey,
  seed: string,
  ...parts: Uint8Array[]
): PublicKey {
  return web3.PublicKey.findProgramAddressSync(
    [UTF8.encode(seed), ...parts.map((part) => Buffer.from(part))],
    programId,
  )[0];
}

function anchorEnum(value: SettlementFundingMethod): Record<string, Record<string, never>> {
  return { [value]: {} };
}

function toPublicKey(value: PublicKey | string): PublicKey {
  return typeof value === "string" ? new web3.PublicKey(value) : value;
}

function toU64(value: bigint, label: string): AnchorBn {
  if (value < 0n || value > U64_MAX) {
    throw new Error(`${label} is outside the unsigned 64-bit range`);
  }
  return new BN(value.toString());
}

function field(record: DynamicRecord, name: string): unknown {
  if (name in record) {
    return record[name];
  }
  const snake = name.replace(/[A-Z]/g, (character) => `_${character.toLowerCase()}`);
  if (snake in record) {
    return record[snake];
  }
  throw new Error(`Decoded account is missing field ${name}`);
}

function bigintField(record: DynamicRecord, name: string): bigint {
  const value = field(record, name);
  if (typeof value === "bigint") {
    return value;
  }
  if (typeof value === "number" && Number.isSafeInteger(value)) {
    return BigInt(value);
  }
  if (typeof value === "string" && /^\d+$/.test(value)) {
    return BigInt(value);
  }
  if (isObject(value) && typeof value.toString === "function") {
    const text = value.toString();
    if (/^\d+$/.test(text)) {
      return BigInt(text);
    }
  }
  throw new Error(`Decoded account field ${name} is not an unsigned integer`);
}

function publicKeyField(record: DynamicRecord, name: string): PublicKey {
  const value = field(record, name);
  if (value instanceof web3.PublicKey) {
    return value;
  }
  if (typeof value === "string") {
    return new web3.PublicKey(value);
  }
  if (isObject(value) && typeof value.toBase58 === "function") {
    return new web3.PublicKey(value.toBase58());
  }
  throw new Error(`Decoded account field ${name} is not a public key`);
}

function bytesField(record: DynamicRecord, name: string): Uint8Array {
  const value = field(record, name);
  if (value instanceof Uint8Array) {
    return Uint8Array.from(value);
  }
  if (Array.isArray(value) && value.every((item) => Number.isInteger(item))) {
    return Uint8Array.from(value as number[]);
  }
  if (typeof Buffer !== "undefined" && Buffer.isBuffer(value)) {
    return Uint8Array.from(value);
  }
  throw new Error(`Decoded account field ${name} is not a byte array`);
}

function enumField<T extends string>(
  record: DynamicRecord,
  name: string,
  allowed: readonly T[],
): T {
  const value = field(record, name);
  const raw =
    typeof value === "string"
      ? value
      : isObject(value)
        ? Object.keys(value)[0]
        : undefined;
  const normalized = normalizeEnum(raw);
  const match = allowed.find((candidate) => normalizeEnum(candidate) === normalized);
  if (!match) {
    throw new Error(`Decoded account field ${name} has an unsupported enum value`);
  }
  return match;
}

function normalizeEnum(value: unknown): string {
  return String(value ?? "")
    .replace(/[^a-zA-Z0-9]/g, "")
    .toLowerCase();
}

function assertBytes32(value: Uint8Array, label: string): void {
  if (!(value instanceof Uint8Array) || value.length !== 32) {
    throw new Error(`${label} must contain exactly 32 bytes`);
  }
}

function requiredBytes(value: Uint8Array | undefined, label: string): Uint8Array {
  if (!value) {
    throw new Error(`Settlement record is missing ${label}`);
  }
  assertBytes32(value, label);
  return value;
}

function requiredString(value: string | undefined, label: string): string {
  if (!value || value.trim().length === 0) {
    throw new Error(`Settlement record is missing ${label}`);
  }
  return value;
}

function equalBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function dedupeSigners(signers: Signer[]): Signer[] {
  const seen = new Set<string>();
  return signers.filter((signer) => {
    const key = signer.publicKey.toBase58();
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertNever(value: never): never {
  throw new Error(`Unsupported settlement disposition: ${String(value)}`);
}
