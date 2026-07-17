import { randomBytes } from "node:crypto";
import { calculateTwap, calculateVelocityBps, calculateVolatilityBps, effectivePrice } from "./metrics.js";
import type { ApcProgramClient, ApcSnapshot, ExecutionVenue, MarketDataSource } from "./types.js";
export class ApcMarketEngine {
  constructor(private readonly market: MarketDataSource, private readonly program: ApcProgramClient, private readonly venue: ExecutionVenue, private readonly maxBandsPerCycle=32) {}
  async runCycle(sequence: bigint): Promise<void> {
    const market=await this.market.readApprovedPool();
    if (!market.samples.length) throw new Error("approved pool returned no samples");
    const latest=market.samples[market.samples.length-1]!;
    const twap=calculateTwap(market.samples);
    const snapshot:ApcSnapshot={observationId:randomBytes(32),sequence,pool:market.pool,spotPrice:latest.price,twapPrice:twap,twapMinutes:BigInt(Math.max(1,Math.floor((latest.observedAt-market.samples[0]!.observedAt)/60))),liquidityUsd:latest.liquidityUsd,quoteLiquidityUsd:latest.quoteLiquidityUsd,volumeUsd:latest.volumeUsd,netBuyPressureBps:latest.netBuyPressureBps,priceVelocityBps:calculateVelocityBps(market.samples),volatilityBps:calculateVolatilityBps(market.samples,twap),estimatedPriceImpactBps:market.estimatedPriceImpactBps,observedAt:BigInt(latest.observedAt)};
    await this.program.submitObservation(snapshot);
    let state=await this.program.readState();
    for(let activated=0;activated<this.maxBandsPerCycle && effectivePrice(snapshot.spotPrice,snapshot.twapPrice)>=state.nextBandPrice;activated++){
      await this.program.activateNextBand(snapshot.observationId,state.currentBandIndex+1);
      state=await this.program.readState();
    }
    if(state.status==="recovery"){await this.program.executeRecoveryPurchase(snapshot.observationId);return;}
    const amount=await this.program.readPermittedReleaseAmount(state.currentBandIndex,snapshot.observationId);
    if(amount<=0n)return;
    await this.program.executePermittedRelease(state.currentBandIndex,snapshot.observationId,amount);
    const settlement=await this.venue.placeControlledSell(amount);
    if(settlement.actualUsdcProceeds>0n)await this.program.depositCounterweightProceeds(settlement.actualUsdcProceeds,settlement.settlementReference);
  }
}
