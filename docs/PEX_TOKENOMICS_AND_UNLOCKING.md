# Pera-X (PEX) Tokenomics and Market-Conditional Unlocking Policy

## 1. Token Overview

Pera-X (PEX) is the ecosystem token for the Pera-X utility platform. PEX is designed as the ecosystem asset, while internal platform Credits are used as the spending balance for services such as calls, virtual numbers, data, bills, AI tools, and other utilities.

```text
PEX = token / ecosystem asset
Credits = internal service spending balance
```

Users may acquire Credits using PEX, card, stablecoin, or eligible-country virtual accounts. Platform services deduct Credits, while PEX remains the ecosystem token for holding, rewards, discounts, buyback-and-burn participation, liquidity support, and ecosystem value.

## 2. Core Token Parameters

| Parameter | Value |
|---|---:|
| Token Name | Pera-X |
| Symbol | PEX |
| Network | Solana |
| Total Supply | 1,000,000,000 PEX |
| Initial Price | $0.000012 |
| Initial Valuation | $12,000 |

## 3. Token Allocation Structure

| Category | Allocation | Token Amount | Purpose |
|---|---:|---:|---|
| Liquidity Pool | 38% | 380,000,000 PEX | DEX liquidity, reduced volatility, launch confidence, and trading trust. |
| Community / Utility Rewards | 17% | 170,000,000 PEX | User rewards, referral bonuses, service discounts, loyalty incentives, and community growth. |
| Treasury | 12% | 120,000,000 PEX | Future development, strategic expansion, emergency support, listings, integrations, audits, and long-term sustainability. |
| Ecosystem / Marketing | 12% | 120,000,000 PEX | Partnerships, campaigns, influencer marketing, product adoption, brand growth, and ecosystem expansion. |
| Trading Company Operations & Revenue Settlement | 7% | 70,000,000 PEX | Service settlement, liquidity support, buyback-and-burn execution, treasury strengthening, and utility expansion. |
| Team | 6% | 60,000,000 PEX | Founding and core development team, vested gradually to show long-term commitment and avoid early selling pressure. |
| Private / Strategic Investors | 5% | 50,000,000 PEX | Strategic investors supporting funding, partnerships, liquidity, infrastructure, and growth. |
| Advisors | 3% | 30,000,000 PEX | Technical, legal, tokenomics, telecom, fintech, exchange, and strategic advisory support. |
| **Total** | **100%** | **1,000,000,000 PEX** |  |

## 4. Team Allocation Split

| Team Wallet | Allocation | Token Amount |
|---|---:|---:|
| Development Team | 2% | 20,000,000 PEX |
| Founder | 2% | 20,000,000 PEX |
| Future Team Incentives | 1% | 10,000,000 PEX |
| Team Emergency Reserve | 1% | 10,000,000 PEX |
| **Total Team** | **6%** | **60,000,000 PEX** |

## 5. Advisor Allocation Split

| Advisor Wallet | Allocation | Token Amount |
|---|---:|---:|
| Advisor Wallet 1 | 1% | 10,000,000 PEX |
| Advisor Wallet 2 | 1% | 10,000,000 PEX |
| Advisor Wallet 3 | 1% | 10,000,000 PEX |
| **Total Advisors** | **3%** | **30,000,000 PEX** |

## 6. Initial Liquidity Guidance

At the approved launch price of $0.000012, the full 38% liquidity allocation equals:

```text
380,000,000 PEX * $0.000012 = $4,560
```

Approved initial Meteora liquidity structure:

```text
380,000,000 PEX + $4,560 USDC
```

This uses the full 38% liquidity allocation:

```text
Total Liquidity Allocation: 380,000,000 PEX
Initial Liquidity Usage: 380,000,000 PEX
Remaining Liquidity Reserve: 0 PEX
```

## 7. Unlocking Philosophy

PEX reserve allocations will not be unlocked blindly by date alone. Unlocking will be based on market conditions, liquidity strength, trading volume, price stability, ecosystem growth, and business need.

The purpose of unlocking is not to dump tokens into the market. The purpose is to support controlled growth, improve liquidity, reduce unhealthy volatility, and protect long-term holders.

## 8. Reactive Market-Conditional Unlocking Model

Pera-X will use a Reactive Market-Conditional Unlocking Model.

The system may monitor the market every 10 minutes, but release approval is recorded only when defined market-health conditions are met.

```text
Monitor every 10 minutes.
Release approval only when market conditions are healthy.
```

Unlocking is designed to support liquidity and establish stronger support levels, not to force the market down.

## 9. Price Trigger and Support Logic

### Stage 1

```text
Initial price: $0.000012
Trigger zone: around $0.00003
Target support after controlled unlock: around $0.00002
New base price: $0.00002
```

### Stage 2

```text
New base price: $0.00002
Trigger zone: around $0.00006
Target support after controlled unlock: around $0.00004
New base price: $0.00004
```

### Stage 3

```text
New base price: $0.00004
Trigger zone: around $0.00008
Target support after controlled unlock: around $0.00006
New base price: $0.00006
```

From Stage 3 upward, the margin of price movement becomes smaller as the token matures, but the unlocking model remains active.

## 10. TWAP Protection

PEX should use Time Weighted Average Price (TWAP) protection before any market-condition release approval. The price must stay around or above the trigger level for a reasonable period before release approval is considered.

Example:

```text
Trigger price: $0.00003
Price must remain healthy around this level for 30–60 minutes.
Only then can release approval be recorded.
```

This prevents release approval based on one-minute candles, artificial pumps, or low-liquidity price spikes.

## 11. Volume and Liquidity Conditions

Before any release approval, the system must check:

1. Whether trading volume is real and healthy.
2. Whether liquidity is deep enough.
3. Whether the price is stable and not just a quick spike.
4. Whether there is enough buy-side demand.
5. Whether the market has held above the trigger level for enough time.
6. Whether the release has a real business or ecosystem purpose.

If these conditions are not met, release approval should not happen.

## 12. 1000% Price Pump Rule

If PEX increases by a very large amount, such as 1000% within 24 hours, the system should not approve all available tokens at once.

Instead:

```text
Large price movement detected
10-minute monitoring confirms movement
TWAP and liquidity checks are applied
Only controlled tranches may be approved
Daily release limits protect the market
```

## 13. Daily Unlock Cap

A daily release limit must protect the market from excessive supply release.

Recommended cap:

```text
Maximum release approval per 24 hours: 1% of total supply
```

For 1,000,000,000 PEX:

```text
1% = 10,000,000 PEX maximum per 24 hours
```

This cap may be reduced depending on liquidity depth and market condition.

## 14. Cooldown Period

After any release approval, there must be a cooldown period before another release approval can happen.

Recommended cooldown:

```text
2 to 6 hours after each release approval
```

This allows the market to absorb newly released supply and prevents repeated release pressure.

## 15. Controlled Tranche Unlocking

Tokens should be released in small, controlled tranches. The market-condition engine should never approve a large reserve portion at once.

Each tranche should be based on:

1. Market price.
2. Trading volume.
3. Liquidity depth.
4. Buy-side demand.
5. Previous release effect.
6. Business purpose.
7. Community confidence.

## 16. Market-Condition Oracle Release Authority

Pera-X uses a 100% market-condition release approval model. Release approval is not controlled by manual multisig voting. The authorized market-condition oracle records release approvals only after the required market-health gates are satisfied.

The oracle-controlled release model is designed to keep the process fast, rule-based, and consistent. Emergency pause and system maintenance remain separate safety controls, but they do not replace the market-condition release authority.

Approved process:

```text
Market bot monitors market every 10 minutes
Oracle verifies price, TWAP, liquidity, volume, buy pressure, caps, cooldown, and business purpose
Oracle records release approval on-chain when all gates are satisfied
ReleaseRecord PDA prevents duplicate release IDs
Emergency pause can stop the system if market/security risk is detected
```

## 17. Emergency Pause Rule

The unlocking system must include an emergency pause.

Release approvals should stop immediately if:

1. Market manipulation is detected.
2. Liquidity becomes weak.
3. Price crashes suddenly.
4. Abnormal sell pressure appears.
5. A DEX or liquidity pool issue occurs.
6. Bot or oracle data becomes unreliable.
7. A security issue is detected.
8. Community confidence is at risk.

## 18. Purpose-Based Unlocking

Every release approval must have a clear reason.

Valid reasons may include:

1. Liquidity support.
2. Utility expansion.
3. Exchange or DEX growth.
4. Service settlement.
5. Buyback-and-burn support.
6. Partnership execution.
7. Treasury strengthening.
8. Community rewards.
9. Strategic investor support.
10. Development need.

Tokens should not be released without a defined purpose.

## 19. Public Explanation

For public documentation, Pera-X should use this wording:

```text
Pera-X uses a market-conditional release model. Reserve tokens are not released on fixed dates alone. Release approval is based on price stability, liquidity health, trading volume, ecosystem growth, and approved business needs. The system monitors market conditions regularly and may approve controlled tranches only when the market is healthy enough to absorb new supply.
```

Avoid wording such as:

```text
We release tokens to bring the price down.
```

Use:

```text
Release approval is designed to support liquidity, reduce unhealthy volatility, and establish stronger support levels.
```

## 20. Final Principle Summary

Pera-X will use Reactive Market-Conditional Unlocking with TWAP Protection.

The market will be monitored every 10 minutes, but release approval will only happen when price, liquidity, volume, TWAP, cooldown, daily cap, monthly cap, and business-purpose conditions are satisfied.

Release approval will happen in controlled tranches, not large releases.

The goal is to protect holders, support liquidity, reduce unhealthy volatility, and build long-term confidence in the Pera-X ecosystem.
