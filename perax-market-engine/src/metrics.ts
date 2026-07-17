import type { PriceSample } from "./types.js";
const BPS = 10_000n;
export function effectivePrice(spot: bigint, twap: bigint): bigint { if (spot <= 0n || twap <= 0n) throw new Error("prices must be positive"); return spot < twap ? spot : twap; }
export function calculateTwap(samples: PriceSample[]): bigint { if (!samples.length) throw new Error("samples required"); return samples.reduce((sum, sample) => sum + sample.price, 0n) / BigInt(samples.length); }
export function calculateVelocityBps(samples: PriceSample[]): number { if (samples.length < 2) return 0; const first=samples[0]!.price; const last=samples[samples.length-1]!.price; if (first<=0n) throw new Error("invalid first price"); const value=((last-first)*BPS)/first; return Number(value < 0n ? -value : value); }
export function calculateVolatilityBps(samples: PriceSample[], twap: bigint): number { if (!samples.length || twap<=0n) return 0; const maxDeviation=samples.reduce((max,s)=>{const d=s.price>twap?s.price-twap:twap-s.price;return d>max?d:max;},0n); return Number((maxDeviation*BPS)/twap); }
