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

At the initial price of $0.000012, an initial $3,000 liquidity position would require:

```text
$3,000 / $0.000012 = 250,000,000 PEX
```

Suggested initial liquidity structure:

```text
250,000,000 PEX + $3,000 USDC
```

This uses part of the 38% liquidity allocation:

```text
Total Liquidity Allocation: 380,000,000 PEX
Initial Liquidity Usage: 250,000,000 PEX
Remaining Liquidity Reserve: 130,000,000 PEX
```

## 7. Unlocking Philosophy

PEX reserve allocations will not be unlocked blindly by date alone. Unlocking will be based on market conditions, liquidity strength, trading volume, price stability, ecosystem growth, and business need.

The purpose of unlocking is not to dump tokens into the market. The purpose is to support controlled growth, improve liquidity, reduce unhealthy volatility, and protect long-term holders.

## 8. Reactive Market-Conditional Unlocking Model

Pera-X will use a Reactive Market-Conditional Unlocking Model.

The system may monitor the market every 10 minutes, but tokens will only unlock when defined health conditions are met.

```text
Monitor every 10 minutes.
Unlock only when market conditions are healthy.
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

PEX should use Time Weighted Average Price (TWAP) protection before any unlock. The price must stay around or above the trigger level for a reasonable period before unlocking is considered.

Example:

```text
Trigger price: $0.00003
Price must remain healthy around this level for 30–60 minutes.
Only then can an unlock review happen.
```

This prevents unlocks based on one-minute candles, artificial pumps, or low-liquidity price spikes.

## 11. Volume and Liquidity Conditions

Before any unlock, the system must check:

1. Whether trading volume is real and healthy.
2. Whether liquidity is deep enough.
3. Whether the price is stable and not just a quick spike.
4. Whether there is enough buy-side demand.
5. Whether the market has held above the trigger level for enough time.
6. Whether the unlock has a real business or ecosystem purpose.

If these conditions are not met, the unlock should not happen.

## 12. 1000% Price Pump Rule

If PEX increases by a very large amount, such as 1000% within 24 hours, the system should not unlock all available tokens at once.

Instead:

```text
Large price movement detected
10-minute monitoring confirms movement
TWAP and liquidity checks are applied
Only controlled tranches may unlock
Daily unlock limits protect the market
```

## 13. Daily Unlock Cap

A daily unlock limit must protect the market from excessive supply release.

Recommended cap:

```text
Maximum unlock per 24 hours: 1% of total supply
```

For 1,000,000,000 PEX:

```text
1% = 10,000,000 PEX maximum per 24 hours
```

This cap may be reduced depending on liquidity depth, market condition, and internal approval.

## 14. Cooldown Period

After any unlock, there must be a cooldown period before another unlock can happen.

Recommended cooldown:

```text
2 to 6 hours after each unlock
```

This allows the market to absorb newly released supply and prevents repeated unlock pressure.

## 15. Controlled Tranche Unlocking

Tokens should be unlocked in small, controlled tranches. Unlocking should never release a large reserve portion at once.

Each tranche should be based on:

1. Market price.
2. Trading volume.
3. Liquidity depth.
4. Buy-side demand.
5. Previous unlock effect.
6. Business purpose.
7. Community confidence.

## 16. Manual or Multisig Approval

At the early stage, Pera-X should not rely only on a fully automatic unlock bot.

The system may recommend unlocks automatically, but final execution should require approval from the project authority, trading company, or multisig wallet.

Recommended process:

```text
Bot monitors market every 10 minutes
Bot recommends unlock
Team / trading company / multisig reviews
Unlock is approved or rejected
```

## 17. Emergency Pause Rule

The unlocking system must include an emergency pause.

Unlocking should stop immediately if:

1. Market manipulation is detected.
2. Liquidity becomes weak.
3. Price crashes suddenly.
4. Abnormal sell pressure appears.
5. A DEX or liquidity pool issue occurs.
6. Bot or oracle data becomes unreliable.
7. A security issue is detected.
8. Community confidence is at risk.

## 18. Purpose-Based Unlocking

Every unlock must have a clear reason.

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

Tokens should not be unlocked without a defined purpose.

## 19. Public Explanation

For public documentation, Pera-X should use this wording:

```text
Pera-X uses a market-conditional unlocking model. Reserve tokens are not released on fixed dates alone. Unlocking is based on price stability, liquidity health, trading volume, ecosystem growth, and approved business needs. The system monitors market conditions regularly and may release controlled tranches only when the market is healthy enough to absorb new supply.
```

Avoid wording such as:

```text
We unlock tokens to bring the price down.
```

Use:

```text
Unlocking is designed to support liquidity, reduce unhealthy volatility, and establish stronger support levels.
```

## 20. Final Principle Summary

Pera-X will use Reactive Market-Conditional Unlocking with TWAP Protection.

The market will be monitored every 10 minutes, but tokens will only unlock when price, liquidity, volume, TWAP, cooldown, daily cap, and approval conditions are satisfied.

Unlocking will happen in controlled tranches, not large releases.

The goal is to protect holders, support liquidity, reduce unhealthy volatility, and build long-term confidence in the Pera-X ecosystem.
