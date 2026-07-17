import test from "node:test"; import assert from "node:assert/strict"; import { calculateTwap, calculateVelocityBps, calculateVolatilityBps } from "../src/metrics.js";
const samples=[{price:100n,liquidityUsd:1n,quoteLiquidityUsd:1n,volumeUsd:1n,netBuyPressureBps:5000,observedAt:0},{price:200n,liquidityUsd:1n,quoteLiquidityUsd:1n,volumeUsd:1n,netBuyPressureBps:5000,observedAt:60}];
test("metrics are deterministic",()=>{assert.equal(calculateTwap(samples),150n);assert.equal(calculateVelocityBps(samples),10000);assert.equal(calculateVolatilityBps(samples,150n),3333);});
