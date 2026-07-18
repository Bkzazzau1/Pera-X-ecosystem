import { randomBytes } from "node:crypto";
import { calculateTwap, calculateVelocityBps, calculateVolatilityBps, effectivePrice } from "./metrics.js";
import { allocatePolicyProceeds, APC_POLICY_V1 } from "./policy.js";
import type { ApcProgramClient, ApcSnapshot, ExecutionVenue, MarketDataSource } from "./types.js";

export class ApcMarketEngine {
  constructor(
    private readonly market: MarketDataSource,
    private readonly program: ApcProgramClient,
    private readonly venue: ExecutionVenue,
    private readonly maxBandsPerCycle = 32,
  ) {}

  async runCycle(sequence: bigint): Promise<void> {
    const market = await this.market.readApprovedPool();
    if (!market.samples.length) throw new Error("approved pool returned no samples");
    const latest = market.samples[market.samples.length - 1]!;
    const first = market.samples[0]!;
    const twap = calculateTwap(market.samples);
    const twapMinutes = Math.max(1, Math.floor((latest.observedAt - first.observedAt) / 60));
    if (twapMinutes < APC_POLICY_V1.minimumTwapMinutes) throw new Error("approved pool TWAP window is below APC Policy V1");
    if (latest.liquidityUsd < APC_POLICY_V1.minimumLiquidityUsd) throw new Error("approved pool liquidity is below APC Policy V1");
    if (latest.quoteLiquidityUsd < APC_POLICY_V1.minimumQuoteLiquidityUsd) throw new Error("approved quote liquidity is below APC Policy V1");
    if (latest.volumeUsd < APC_POLICY_V1.minimumVolumeUsd) throw new Error("approved pool volume is below APC Policy V1");
    if (latest.netBuyPressureBps < APC_POLICY_V1.minimumBuyPressureBps) throw new Error("approved buy pressure is below APC Policy V1");

    const snapshot: ApcSnapshot = {
      observationId: randomBytes(32), sequence, pool: market.pool,
      spotPrice: latest.price, twapPrice: twap, twapMinutes: BigInt(twapMinutes),
      liquidityUsd: latest.liquidityUsd, quoteLiquidityUsd: latest.quoteLiquidityUsd,
      volumeUsd: latest.volumeUsd, netBuyPressureBps: latest.netBuyPressureBps,
      priceVelocityBps: calculateVelocityBps(market.samples),
      volatilityBps: calculateVolatilityBps(market.samples, twap),
      estimatedPriceImpactBps: market.estimatedPriceImpactBps,
      observedAt: BigInt(latest.observedAt),
    };
    await this.program.submitObservation(snapshot);
    let state = await this.program.readState();
    for (let activated = 0; activated < this.maxBandsPerCycle && effectivePrice(snapshot.spotPrice, snapshot.twapPrice) >= state.nextBandPrice; activated += 1) {
      await this.program.activateNextBand(snapshot.observationId, state.currentBandIndex + 1);
      state = await this.program.readState();
    }
    if (state.status === "recovery") {
      await this.program.executeRecoveryPurchase(snapshot.observationId);
      return;
    }
    const amount = await this.program.readPermittedReleaseAmount(state.currentBandIndex, snapshot.observationId);
    if (amount <= 0n) return;
    await this.program.executePermittedRelease(state.currentBandIndex, snapshot.observationId, amount);
    const settlement = await this.venue.placeControlledSell(amount);
    const allocation = allocatePolicyProceeds(settlement.actualUsdcProceeds);
    if (allocation.counterweightVault > 0n) {
      await this.program.depositCounterweightProceeds(allocation.counterweightVault, settlement.settlementReference);
    }
    await this.venue.recordPolicyAllocation?.(allocation, settlement.settlementReference);
  }
}
