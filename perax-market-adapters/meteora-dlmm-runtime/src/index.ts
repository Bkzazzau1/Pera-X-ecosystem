import { randomBytes } from "node:crypto";
import { createRequire } from "node:module";
import { readFile, rename, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { mkdir } from "node:fs/promises";

import anchor, { AnchorProvider, Program, type Idl, web3 } from "@coral-xyz/anchor";
import type { AccountMeta, PublicKey, TransactionInstruction } from "@solana/web3.js";

const { BN } = anchor;
const require = createRequire(import.meta.url);
const Meteora = require("@meteora-ag/dlmm") as MeteoraModule;

const UTF8 = new TextEncoder();
const TOKEN_PROGRAM_ID = new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ASSOCIATED_TOKEN_PROGRAM_ID = new web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SWAP_EXACT_OUT2_DISCRIMINATOR = Buffer.from([43, 215, 247, 132, 137, 60, 243, 81]);
const U64_MAX = (1n << 64n) - 1n;

export type SettlementRecord = {
  settlementId: Uint8Array;
  marketMode: string;
  disposition: string;
  status: string;
  pexObligation: bigint;
  marketPexRequired: bigint;
  policyVaultPexRequired: bigint;
  marketPexReceived: bigint;
  policyVaultPexReceived: bigint;
  directPexReceived: bigint;
  settlementRecordAddress?: string;
};

export type AtomicMarketPurchase = {
  maximumQuoteAmount: bigint;
  minimumPexOut: bigint;
  instructionData: Uint8Array;
  remainingAccounts?: Array<{
    publicKey: string;
    isWritable: boolean;
    isSigner?: boolean;
  }>;
};

export type SettlementRuntimeContext = {
  provider: AnchorProvider;
  programId: PublicKey;
  statePda: PublicKey;
  pexMint: PublicKey;
  idl: unknown;
};

type RuntimeBindings = {
  venue: {
    buildAtomicPexPurchase(input: {
      settlement: SettlementRecord;
      pexAmount: bigint;
    }): Promise<AtomicMarketPurchase>;
  };
  observations: { getFreshObservationId(): Promise<Uint8Array> };
  resolveQuoteSource(
    settlement: SettlementRecord,
    purchase: AtomicMarketPurchase,
  ): Promise<{ authority: PublicKey; tokenAccount: PublicKey }>;
  isTerminalError(error: unknown): boolean;
};

type MeteoraModule = {
  create(connection: web3.Connection, pool: PublicKey, options?: Record<string, unknown>): Promise<MeteoraPool>;
  IDL: { address?: string };
  MEMO_PROGRAM_ID: PublicKey;
};

type MeteoraPool = {
  pubkey: PublicKey;
  lbPair: {
    tokenXMint: PublicKey;
    tokenYMint: PublicKey;
    oracle: PublicKey;
  };
  tokenX: TokenReserve;
  tokenY: TokenReserve;
  refetchStates(): Promise<void>;
  getActiveBin(): Promise<{ pricePerToken: string }>;
  getOracle(): Promise<{
    getUiPriceByTime(start: InstanceType<typeof BN>, end: InstanceType<typeof BN>): {
      value: { toString(): string };
      duration: InstanceType<typeof BN>;
    } | null;
  }>;
  getBinArrayForSwap(swapForY: boolean): Promise<Array<{ publicKey: PublicKey }>>;
  swapQuoteExactOut(
    outAmount: InstanceType<typeof BN>,
    swapForY: boolean,
    slippage: InstanceType<typeof BN>,
    bins: Array<{ publicKey: PublicKey }>,
  ): {
    maxInAmount: InstanceType<typeof BN>;
    binArraysPubkey: PublicKey[];
    priceImpact: { toString(): string };
  };
  swapExactOut(input: {
    inToken: PublicKey;
    outToken: PublicKey;
    outAmount: InstanceType<typeof BN>;
    maxInAmount: InstanceType<typeof BN>;
    lbPair: PublicKey;
    user: PublicKey;
    binArraysPubkey: PublicKey[];
  }): Promise<{ instructions: TransactionInstruction[] }>;
};

type TokenReserve = {
  publicKey: PublicKey;
  reserve: PublicKey;
  mint: { decimals: number };
  amount: bigint;
  owner: PublicKey;
};

type RuntimeConfig = {
  pool: PublicKey;
  quoteMint: PublicKey;
  quoteTokenAccount: PublicKey;
  maximumSlippageBps: number;
  observationTwapSeconds: number;
  observationFlowWindowSeconds: number;
  observationProbePexAmount: bigint;
  observationStatePath: string;
};

type ReserveSample = {
  observedAt: number;
  quoteReserve: string;
  pexReserve: string;
  spotPriceScaled: string;
};

type MarketSnapshot = {
  observedAt: number;
  spotPrice: bigint;
  twapPrice: bigint;
  twapMinutes: bigint;
  liquidityUsd: bigint;
  quoteLiquidityUsd: bigint;
  volumeUsd: bigint;
  netBuyPressureBps: number;
  priceVelocityBps: number;
  volatilityBps: number;
  estimatedPriceImpactBps: number;
  sample: ReserveSample;
};

class ObservationWarmupError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ObservationWarmupError";
  }
}

export async function createSettlementRuntime(
  context: SettlementRuntimeContext,
): Promise<RuntimeBindings> {
  const runtime = await MeteoraSettlementRuntime.create(context, process.env);
  return {
    venue: runtime,
    observations: runtime,
    resolveQuoteSource: runtime.resolveQuoteSource.bind(runtime),
    isTerminalError,
  };
}

export class MeteoraSettlementRuntime {
  private readonly peraxProgram: DynamicProgram;
  private readonly apcConfigPda: PublicKey;
  private readonly apcStatePda: PublicKey;
  private readonly store: JsonSampleStore;
  private serial: Promise<unknown> = Promise.resolve();

  private constructor(
    private readonly context: SettlementRuntimeContext,
    private readonly config: RuntimeConfig,
    private readonly pool: MeteoraPool,
    private readonly quoteIsX: boolean,
  ) {
    this.peraxProgram = new Program(
      context.idl as Idl,
      context.provider,
    ) as unknown as DynamicProgram;
    this.apcConfigPda = derivePda(context.programId, "apc-config", context.statePda.toBuffer());
    this.apcStatePda = derivePda(context.programId, "apc-state", this.apcConfigPda.toBuffer());
    this.store = new JsonSampleStore(config.observationStatePath);
  }

  static async create(
    context: SettlementRuntimeContext,
    environment: NodeJS.ProcessEnv,
  ): Promise<MeteoraSettlementRuntime> {
    const config = readRuntimeConfig(environment);
    const apcProgram = new Program(
      context.idl as Idl,
      context.provider,
    ) as unknown as DynamicProgram;
    const apcConfigPda = derivePda(context.programId, "apc-config", context.statePda.toBuffer());
    const apcConfig = await fetchProgramAccount(apcProgram, "apcConfig", apcConfigPda);
    const approvedPool = publicKeyField(apcConfig, "approvedPool");
    const approvedMarketProgram = publicKeyField(apcConfig, "approvedRecoveryProgram");
    const oracleFeed = publicKeyField(apcConfig, "oracleFeed");
    const quoteMint = publicKeyField(apcConfig, "quoteMint");

    requireEqual(config.pool, approvedPool, "Configured Meteora pool is not the APC-approved pool");
    requireEqual(config.quoteMint, quoteMint, "Configured quote mint is not the APC quote mint");
    requireEqual(
      context.provider.publicKey,
      oracleFeed,
      "Settlement executor signer must be the configured autonomous oracle signer",
    );
    const officialProgram = new web3.PublicKey(requiredString(Meteora.IDL.address, "Meteora IDL address"));
    requireEqual(
      approvedMarketProgram,
      officialProgram,
      "APC-approved market program is not the supported Meteora DLMM program",
    );

    const pool = await Meteora.create(context.provider.connection, config.pool, {
      programId: approvedMarketProgram,
      skipSolWrappingOperation: true,
    });
    requireEqual(pool.pubkey, config.pool, "Meteora SDK loaded a different pool");
    const xIsPex = pool.tokenX.publicKey.equals(context.pexMint);
    const yIsPex = pool.tokenY.publicKey.equals(context.pexMint);
    if (xIsPex === yIsPex) {
      throw new Error("Approved Meteora pool must contain PEX exactly once");
    }
    const quoteIsX = pool.tokenX.publicKey.equals(config.quoteMint);
    const quoteIsY = pool.tokenY.publicKey.equals(config.quoteMint);
    if (quoteIsX === quoteIsY || quoteIsX === xIsPex) {
      throw new Error("Approved Meteora pool must be the configured PEX/quote pair");
    }
    requireEqual(pool.tokenX.owner, TOKEN_PROGRAM_ID, "Meteora token X must use classic SPL Token");
    requireEqual(pool.tokenY.owner, TOKEN_PROGRAM_ID, "Meteora token Y must use classic SPL Token");

    await validateClassicTokenAccount(
      context.provider.connection,
      config.quoteTokenAccount,
      config.quoteMint,
      context.provider.publicKey,
    );
    return new MeteoraSettlementRuntime(context, config, pool, quoteIsX);
  }

  async buildAtomicPexPurchase(input: {
    settlement: SettlementRecord;
    pexAmount: bigint;
  }): Promise<AtomicMarketPurchase> {
    assertU64(input.pexAmount, "PEX exact-output amount");
    await this.pool.refetchStates();
    const swapForY = this.quoteIsX;
    const bins = await this.pool.getBinArrayForSwap(swapForY);
    const quote = this.pool.swapQuoteExactOut(
      new BN(input.pexAmount.toString()),
      swapForY,
      new BN(this.config.maximumSlippageBps),
      bins,
    );
    const maximumQuoteAmount = BigInt(quote.maxInAmount.toString());
    assertU64(maximumQuoteAmount, "Meteora maximum quote amount");
    const transaction = await this.pool.swapExactOut({
      inToken: this.config.quoteMint,
      outToken: this.context.pexMint,
      outAmount: new BN(input.pexAmount.toString()),
      maxInAmount: quote.maxInAmount,
      lbPair: this.config.pool,
      user: this.context.provider.publicKey,
      binArraysPubkey: quote.binArraysPubkey,
    });
    const instruction = transaction.instructions.find(isSwapExactOut2Instruction);
    if (!instruction) {
      throw new Error("Meteora SDK did not produce swapExactOut2");
    }
    const settlementPexVault = deriveSettlementPexVault(
      this.context.programId,
      this.context.pexMint,
      input.settlement.settlementId,
    );
    const normalized = normalizeMeteoraExactOutInstruction(
      instruction,
      this.config.pool,
      this.config.quoteTokenAccount,
      settlementPexVault,
      this.context.provider.publicKey,
      maximumQuoteAmount,
      input.pexAmount,
    );
    return {
      maximumQuoteAmount,
      minimumPexOut: input.pexAmount,
      instructionData: normalized.data,
      remainingAccounts: normalized.keys.map((key) => ({
        publicKey: key.pubkey.toBase58(),
        isWritable: key.isWritable,
        ...(key.isSigner ? { isSigner: true } : {}),
      })),
    };
  }

  async getFreshObservationId(): Promise<Uint8Array> {
    return this.runSerial(async () => {
      const apcConfig = await fetchProgramAccount(this.peraxProgram, "apcConfig", this.apcConfigPda);
      const apcState = await fetchProgramAccount(this.peraxProgram, "apcState", this.apcStatePda);
      const priceScale = bigintField(apcConfig, "priceScale");
      const snapshot = await this.collectSnapshot(priceScale);
      const observationId = Uint8Array.from(randomBytes(32));
      const observationPda = derivePda(
        this.context.programId,
        "apc-observation",
        Buffer.from(observationId),
      );
      const sequence = bigintField(apcState, "lastObservationSequence") + 1n;
      assertU64(sequence, "APC observation sequence");
      await programMethod(this.peraxProgram, "submitApcObservation")({
          observationId: Array.from(observationId),
          sequence: new BN(sequence.toString()),
          pool: this.config.pool,
          spotPrice: new BN(snapshot.spotPrice.toString()),
          twapPrice: new BN(snapshot.twapPrice.toString()),
          twapMinutes: new BN(snapshot.twapMinutes.toString()),
          liquidityUsd: new BN(snapshot.liquidityUsd.toString()),
          quoteLiquidityUsd: new BN(snapshot.quoteLiquidityUsd.toString()),
          volumeUsd: new BN(snapshot.volumeUsd.toString()),
          netBuyPressureBps: snapshot.netBuyPressureBps,
          priceVelocityBps: snapshot.priceVelocityBps,
          volatilityBps: snapshot.volatilityBps,
          estimatedPriceImpactBps: snapshot.estimatedPriceImpactBps,
          observedAt: new BN(snapshot.observedAt),
        })
        .accountsStrict({
          state: this.context.statePda,
          apcConfig: this.apcConfigPda,
          apcState: this.apcStatePda,
          observation: observationPda,
          oracleFeed: this.context.provider.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed", preflightCommitment: "confirmed" });
      return observationId;
    });
  }

  async resolveQuoteSource(
    _settlement: SettlementRecord,
    purchase: AtomicMarketPurchase,
  ): Promise<{ authority: PublicKey; tokenAccount: PublicKey }> {
    const account = await validateClassicTokenAccount(
      this.context.provider.connection,
      this.config.quoteTokenAccount,
      this.config.quoteMint,
      this.context.provider.publicKey,
    );
    if (account.amount < purchase.maximumQuoteAmount) {
      throw new ObservationWarmupError("Quote source balance is below the contract-bounded purchase ceiling");
    }
    return {
      authority: this.context.provider.publicKey,
      tokenAccount: this.config.quoteTokenAccount,
    };
  }

  private async collectSnapshot(priceScale: bigint): Promise<MarketSnapshot> {
    await this.pool.refetchStates();
    const now = Math.floor(Date.now() / 1_000);
    const active = await this.pool.getActiveBin();
    const oracle = await this.pool.getOracle();
    const twap = oracle.getUiPriceByTime(
      new BN(now - this.config.observationTwapSeconds),
      new BN(now),
    );
    if (!twap) {
      throw new ObservationWarmupError("Meteora on-chain oracle does not yet cover the configured TWAP window");
    }
    const spotPrice = orientPrice(active.pricePerToken, this.quoteIsX, priceScale);
    const twapPrice = orientPrice(twap.value.toString(), this.quoteIsX, priceScale);
    const twapMinutes = BigInt(twap.duration.toString()) / 60n;
    if (twapMinutes <= 0n) {
      throw new ObservationWarmupError("Meteora TWAP duration is below one minute");
    }
    const quoteReserve = this.quoteIsX ? this.pool.tokenX.amount : this.pool.tokenY.amount;
    const pexReserve = this.quoteIsX ? this.pool.tokenY.amount : this.pool.tokenX.amount;
    const quoteDecimals = this.quoteIsX
      ? this.pool.tokenX.mint.decimals
      : this.pool.tokenY.mint.decimals;
    const pexDecimals = this.quoteIsX
      ? this.pool.tokenY.mint.decimals
      : this.pool.tokenX.mint.decimals;
    if (quoteDecimals !== 6 || pexDecimals !== 6) {
      throw new Error("Pera-X Meteora runtime currently requires six-decimal PEX and quote mints");
    }

    const sample: ReserveSample = {
      observedAt: now,
      quoteReserve: quoteReserve.toString(),
      pexReserve: pexReserve.toString(),
      spotPriceScaled: spotPrice.toString(),
    };
    const samples = await this.store.record(
      sample,
      Math.max(this.config.observationFlowWindowSeconds, this.config.observationTwapSeconds) * 2,
    );
    const windowStart = now - this.config.observationFlowWindowSeconds;
    const flowSamples = samples.filter((value) => value.observedAt >= windowStart);
    const flow = calculateFlowMetrics(flowSamples, twapPrice);

    const quoteScale = 10n ** BigInt(quoteDecimals);
    const pexScale = 10n ** BigInt(pexDecimals);
    const pexValueQuoteBase = ceilDiv(
      pexReserve * spotPrice * quoteScale,
      pexScale * priceScale,
    );
    const quoteLiquidityUsd = ceilDiv(quoteReserve, quoteScale);
    const liquidityUsd = ceilDiv(quoteReserve + pexValueQuoteBase, quoteScale);

    const bins = await this.pool.getBinArrayForSwap(this.quoteIsX);
    const impactQuote = this.pool.swapQuoteExactOut(
      new BN(this.config.observationProbePexAmount.toString()),
      this.quoteIsX,
      new BN(this.config.maximumSlippageBps),
      bins,
    );
    const estimatedPriceImpactBps = decimalPercentToBps(impactQuote.priceImpact.toString());

    return {
      observedAt: now,
      spotPrice,
      twapPrice,
      twapMinutes,
      liquidityUsd: assertPositiveU64(liquidityUsd, "liquidity USD"),
      quoteLiquidityUsd: assertPositiveU64(quoteLiquidityUsd, "quote liquidity USD"),
      volumeUsd: assertPositiveU64(flow.volumeUsd, "flow volume USD"),
      netBuyPressureBps: flow.netBuyPressureBps,
      priceVelocityBps: flow.priceVelocityBps,
      volatilityBps: flow.volatilityBps,
      estimatedPriceImpactBps,
      sample,
    };
  }

  private async runSerial<T>(operation: () => Promise<T>): Promise<T> {
    const next = this.serial.then(operation, operation);
    this.serial = next.then(() => undefined, () => undefined);
    return next;
  }
}

export function normalizeMeteoraExactOutInstruction(
  instruction: TransactionInstruction,
  approvedPool: PublicKey,
  quoteSource: PublicKey,
  pexDestination: PublicKey,
  authority: PublicKey,
  maximumQuoteAmount: bigint,
  exactPexOut: bigint,
): TransactionInstruction {
  if (!isSwapExactOut2Instruction(instruction)) {
    throw new Error("Only Meteora swapExactOut2 is permitted");
  }
  if (instruction.data.length !== 28 || instruction.data.readUInt32LE(24) !== 0) {
    throw new Error("Meteora exact-out instruction must not contain transfer-hook slices");
  }
  if (instruction.keys.length < 17) {
    throw new Error("Meteora exact-out instruction is missing bin-array accounts");
  }
  const keys = instruction.keys.map((key) => ({ ...key }));
  requireEqual(keys[0]!.pubkey, approvedPool, "Meteora instruction uses the wrong pool");
  requireEqual(keys[10]!.pubkey, authority, "Meteora instruction uses the wrong authority");
  keys[4] = { pubkey: quoteSource, isSigner: false, isWritable: true };
  keys[5] = { pubkey: pexDestination, isSigner: false, isWritable: true };
  const encodedMaximum = readU64Le(instruction.data, 8);
  const encodedOutput = readU64Le(instruction.data, 16);
  if (encodedMaximum !== maximumQuoteAmount || encodedOutput !== exactPexOut) {
    throw new Error("Meteora exact-out instruction amounts do not match the SDK quote");
  }
  return new web3.TransactionInstruction({
    programId: instruction.programId,
    keys,
    data: Buffer.from(instruction.data),
  });
}

export function calculateFlowMetrics(
  samples: ReserveSample[],
  twapPriceScaled: bigint,
): {
  volumeUsd: bigint;
  netBuyPressureBps: number;
  priceVelocityBps: number;
  volatilityBps: number;
} {
  if (samples.length < 2) {
    throw new ObservationWarmupError("At least two durable reserve samples are required");
  }
  let buyFlow = 0n;
  let sellFlow = 0n;
  for (let index = 1; index < samples.length; index += 1) {
    const previous = BigInt(samples[index - 1]!.quoteReserve);
    const current = BigInt(samples[index]!.quoteReserve);
    const delta = current - previous;
    if (delta > 0n) buyFlow += delta;
    if (delta < 0n) sellFlow += -delta;
  }
  const totalFlow = buyFlow + sellFlow;
  if (totalFlow <= 0n) {
    throw new ObservationWarmupError("No measurable quote-reserve flow exists in the observation window");
  }
  const firstPrice = BigInt(samples[0]!.spotPriceScaled);
  const lastPrice = BigInt(samples[samples.length - 1]!.spotPriceScaled);
  if (firstPrice <= 0n || lastPrice <= 0n || twapPriceScaled <= 0n) {
    throw new Error("Observation price history is invalid");
  }
  const priceVelocityBps = toBoundedMetric(
    (absolute(lastPrice - firstPrice) * 10_000n) / firstPrice,
  );
  let maximumDeviation = 0n;
  for (const sample of samples) {
    const price = BigInt(sample.spotPriceScaled);
    maximumDeviation = maximumDeviation > absolute(price - twapPriceScaled)
      ? maximumDeviation
      : absolute(price - twapPriceScaled);
  }
  return {
    volumeUsd: ceilDiv(totalFlow, 1_000_000n),
    netBuyPressureBps: Number((buyFlow * 10_000n) / totalFlow),
    priceVelocityBps,
    volatilityBps: toBoundedMetric((maximumDeviation * 10_000n) / twapPriceScaled),
  };
}

export function orientPrice(
  tokenYPerTokenX: string,
  quoteIsX: boolean,
  priceScale: bigint,
): bigint {
  const raw = parseDecimalFraction(tokenYPerTokenX);
  if (priceScale <= 0n) {
    throw new Error("Meteora price is invalid");
  }
  const scaled = quoteIsX
    ? (raw.denominator * priceScale) / raw.numerator
    : (raw.numerator * priceScale) / raw.denominator;
  if (scaled <= 0n || scaled > U64_MAX) {
    throw new Error("Scaled PEX price is outside u64 range");
  }
  return scaled;
}

export function isTerminalError(error: unknown): boolean {
  if (error instanceof ObservationWarmupError) return false;
  const message = error instanceof Error ? error.message : String(error);
  return /must be|wrong pool|wrong authority|does not match|invalid bindings|unsupported|requires six-decimal|not the APC|not the supported Meteora|conflicts with/i.test(message);
}

class JsonSampleStore {
  constructor(private readonly filePath: string) {}

  async record(sample: ReserveSample, retentionSeconds: number): Promise<ReserveSample[]> {
    const existing = await this.read();
    const cutoff = sample.observedAt - retentionSeconds;
    const retained = existing.filter((value) => value.observedAt >= cutoff);
    const last = retained.at(-1);
    if (last && last.observedAt === sample.observedAt) {
      retained[retained.length - 1] = sample;
    } else {
      retained.push(sample);
    }
    await mkdir(dirname(this.filePath), { recursive: true });
    const temporary = `${this.filePath}.${process.pid}.tmp`;
    await writeFile(temporary, JSON.stringify(retained, null, 2) + "\n", "utf8");
    await rename(temporary, this.filePath);
    return retained;
  }

  private async read(): Promise<ReserveSample[]> {
    try {
      const parsed = JSON.parse(await readFile(this.filePath, "utf8")) as unknown;
      if (!Array.isArray(parsed)) throw new Error("Observation state must be an array");
      return parsed.map(validateSample);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") return [];
      throw error;
    }
  }
}

type DynamicProgram = {
  account: Record<string, { fetch(address: PublicKey): Promise<Record<string, unknown>> }>;
  methods: Record<string, (params: unknown) => {
    accountsStrict(accounts: Record<string, PublicKey>): {
      rpc(options?: Record<string, unknown>): Promise<string>;
    };
  }>;
};


function parseDecimalFraction(value: string): { numerator: bigint; denominator: bigint } {
  const normalized = value.trim();
  const match = /^(?:\+)?(\d+)(?:\.(\d*))?(?:[eE]([+-]?\d+))?$/.exec(normalized);
  if (!match) throw new Error("Decimal market value is invalid");
  const whole = match[1]!;
  const fraction = match[2] ?? "";
  const exponent = Number(match[3] ?? "0");
  if (!Number.isSafeInteger(exponent) || Math.abs(exponent) > 100) {
    throw new Error("Decimal exponent is outside the supported range");
  }
  let numerator = BigInt(whole + fraction);
  let denominator = 10n ** BigInt(fraction.length);
  if (exponent > 0) numerator *= 10n ** BigInt(exponent);
  if (exponent < 0) denominator *= 10n ** BigInt(-exponent);
  if (numerator <= 0n) throw new Error("Decimal market value must be positive");
  return { numerator, denominator };
}

function fetchProgramAccount(
  program: DynamicProgram,
  name: string,
  address: PublicKey,
): Promise<Record<string, unknown>> {
  const client = program.account[name];
  if (!client) throw new Error(`Pera-X IDL is missing account client ${name}`);
  return client.fetch(address);
}

function programMethod(
  program: DynamicProgram,
  name: string,
): (params: unknown) => {
  accountsStrict(accounts: Record<string, PublicKey>): {
    rpc(options?: Record<string, unknown>): Promise<string>;
  };
} {
  const method = program.methods[name];
  if (!method) throw new Error(`Pera-X IDL is missing method ${name}`);
  return method;
}

function readRuntimeConfig(environment: NodeJS.ProcessEnv): RuntimeConfig {
  return {
    pool: new web3.PublicKey(requiredEnvironment(environment, "METEORA_DLMM_POOL")),
    quoteMint: new web3.PublicKey(requiredEnvironment(environment, "PERAX_QUOTE_MINT_ADDRESS")),
    quoteTokenAccount: new web3.PublicKey(
      requiredEnvironment(environment, "PERAX_SETTLEMENT_QUOTE_TOKEN_ACCOUNT"),
    ),
    maximumSlippageBps: positiveInteger(
      requiredEnvironment(environment, "METEORA_MAX_SLIPPAGE_BPS"),
      "METEORA_MAX_SLIPPAGE_BPS",
      9_999,
    ),
    observationTwapSeconds: positiveInteger(
      requiredEnvironment(environment, "PERAX_OBSERVATION_TWAP_SECONDS"),
      "PERAX_OBSERVATION_TWAP_SECONDS",
      86_400,
    ),
    observationFlowWindowSeconds: positiveInteger(
      requiredEnvironment(environment, "PERAX_OBSERVATION_FLOW_WINDOW_SECONDS"),
      "PERAX_OBSERVATION_FLOW_WINDOW_SECONDS",
      86_400,
    ),
    observationProbePexAmount: positiveBigInt(
      requiredEnvironment(environment, "PERAX_OBSERVATION_PROBE_PEX_AMOUNT"),
      "PERAX_OBSERVATION_PROBE_PEX_AMOUNT",
    ),
    observationStatePath: resolve(
      requiredEnvironment(environment, "PERAX_OBSERVATION_STATE_PATH"),
    ),
  };
}

async function validateClassicTokenAccount(
  connection: web3.Connection,
  address: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
): Promise<{ amount: bigint }> {
  const account = await connection.getAccountInfo(address, "confirmed");
  if (!account || !account.owner.equals(TOKEN_PROGRAM_ID) || account.data.length < 72) {
    throw new Error("Configured quote source is not a classic SPL token account");
  }
  requireEqual(new web3.PublicKey(account.data.subarray(0, 32)), mint, "Quote source mint mismatch");
  requireEqual(new web3.PublicKey(account.data.subarray(32, 64)), owner, "Quote source owner mismatch");
  return { amount: readU64Le(account.data, 64) };
}

function deriveSettlementPexVault(
  programId: PublicKey,
  pexMint: PublicKey,
  settlementId: Uint8Array,
): PublicKey {
  if (settlementId.length !== 32) throw new Error("Settlement ID must contain 32 bytes");
  const record = derivePda(programId, "settlement", Buffer.from(settlementId));
  const authority = derivePda(
    programId,
    "settlement-custody-authority",
    record.toBuffer(),
  );
  return web3.PublicKey.findProgramAddressSync(
    [authority.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), pexMint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  )[0];
}

function isSwapExactOut2Instruction(instruction: TransactionInstruction): boolean {
  return instruction.data.length >= 8
    && Buffer.from(instruction.data.subarray(0, 8)).equals(SWAP_EXACT_OUT2_DISCRIMINATOR);
}

function derivePda(programId: PublicKey, label: string, ...seeds: Buffer[]): PublicKey {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from(UTF8.encode(label)), ...seeds],
    programId,
  )[0];
}

function publicKeyField(record: Record<string, unknown>, field: string): PublicKey {
  const value = record[field];
  if (value instanceof web3.PublicKey) return value;
  if (typeof value === "string") return new web3.PublicKey(value);
  if (value && typeof value === "object" && "toBase58" in value) {
    return new web3.PublicKey((value as { toBase58(): string }).toBase58());
  }
  throw new Error(`Missing public key field ${field}`);
}

function bigintField(record: Record<string, unknown>, field: string): bigint {
  const value = record[field];
  if (typeof value === "bigint") return value;
  if (typeof value === "number" && Number.isSafeInteger(value)) return BigInt(value);
  if (typeof value === "string" && /^\d+$/.test(value)) return BigInt(value);
  if (value && typeof value === "object" && "toString" in value) {
    const text = String(value);
    if (/^\d+$/.test(text)) return BigInt(text);
  }
  throw new Error(`Missing integer field ${field}`);
}

function validateSample(value: unknown): ReserveSample {
  if (!value || typeof value !== "object") throw new Error("Observation sample is invalid");
  const sample = value as Record<string, unknown>;
  const observedAt = Number(sample.observedAt);
  const quoteReserve = String(sample.quoteReserve ?? "");
  const pexReserve = String(sample.pexReserve ?? "");
  const spotPriceScaled = String(sample.spotPriceScaled ?? "");
  if (!Number.isSafeInteger(observedAt) || observedAt <= 0
      || !/^\d+$/.test(quoteReserve) || !/^\d+$/.test(pexReserve)
      || !/^\d+$/.test(spotPriceScaled)) {
    throw new Error("Observation sample fields are invalid");
  }
  return { observedAt, quoteReserve, pexReserve, spotPriceScaled };
}

function decimalPercentToBps(value: string): number {
  const impact = parseDecimalFraction(value);
  const bps = ceilDiv(impact.numerator * 100n, impact.denominator);
  if (bps < 0n || bps > 10_000_000n) {
    throw new Error("Meteora price impact is outside the APC metric range");
  }
  return Number(bps);
}

function toBoundedMetric(value: bigint): number {
  if (value < 0n || value > 10_000_000n) {
    throw new Error("Calculated APC metric exceeds the contract range");
  }
  return Number(value);
}

function readU64Le(buffer: Uint8Array, offset: number): bigint {
  if (offset < 0 || offset + 8 > buffer.length) throw new Error("u64 read is out of bounds");
  let value = 0n;
  for (let index = 7; index >= 0; index -= 1) {
    value = (value << 8n) | BigInt(buffer[offset + index]!);
  }
  return value;
}

function ceilDiv(numerator: bigint, denominator: bigint): bigint {
  if (numerator < 0n || denominator <= 0n) throw new Error("Invalid ceiling division");
  return (numerator + denominator - 1n) / denominator;
}

function absolute(value: bigint): bigint {
  return value < 0n ? -value : value;
}

function assertU64(value: bigint, label: string): void {
  if (value <= 0n || value > U64_MAX) throw new Error(`${label} is outside u64 range`);
}

function assertPositiveU64(value: bigint, label: string): bigint {
  assertU64(value, label);
  return value;
}

function positiveBigInt(value: string, label: string): bigint {
  if (!/^\d+$/.test(value)) throw new Error(`${label} must be a positive integer`);
  const parsed = BigInt(value);
  assertU64(parsed, label);
  return parsed;
}

function positiveInteger(value: string, label: string, maximum: number): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0 || parsed > maximum) {
    throw new Error(`${label} must be an integer between 1 and ${maximum}`);
  }
  return parsed;
}

function requiredEnvironment(environment: NodeJS.ProcessEnv, name: string): string {
  const value = environment[name]?.trim();
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} is required`);
  }
  return value.trim();
}

function requireEqual(actual: PublicKey, expected: PublicKey, message: string): void {
  if (!actual.equals(expected)) throw new Error(message);
}
