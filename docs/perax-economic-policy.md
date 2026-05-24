# Pera-X Economic Policy

## 1. Purpose

Pera-X is a utility-driven economic token designed to support real-world digital services such as international calls, foreign numbers, data, utility bills, and other future services provided by the Pera-X ecosystem.

The goal of Pera-X is not only passive holding. The project is designed to encourage real utility, controlled market participation, sustainable token demand, and transparent burn activity.

Pera-X uses a Trading Company as the operational engine of the token economy. The Trading Company receives token-based utility payments, provides access to services, supports liquidity and market operations, and executes burn or buyback actions based on approved economic policy.

## 2. Core Economic Principle

Pera-X is governed by a controlled economic policy system where market conditions, utility usage, holding behavior, liquidity health, and ecosystem growth influence token actions.

The system should avoid random manual decisions. Human administrators may configure approved policy boundaries, but actual recurring decisions should be executed or recommended by bots using defined policy rules.

The main principle is:

> Utility creates demand. Demand supports value. Burn and market-condition controls protect long-term economic balance.

## 3. Trading Company Role

The Trading Company is the central operational wallet and economic engine for Pera-X.

When a user pays with Pera-X for any utility, the Trading Company wallet receives the token first. After receiving the token, the Trading Company provides the requested utility or service to the user.

The Trading Company wallet may be used for:

- Receiving utility payments in Pera-X.
- Executing burn based on the declared daily burn rate.
- Supporting service settlement and operational costs.
- Holding token reserves for market operations.
- Buying Pera-X from the market when its wallet balance becomes low.
- Supporting liquidity and ecosystem stability according to approved policy.

The Trading Company should always remain one of the most important actors in the ecosystem because it connects real-world service demand to token movement.

## 4. Utility Payment Flow

When a user pays directly with Pera-X:

1. User selects a service or utility.
2. Backend calculates the token amount required.
3. User pays Pera-X into the Trading Company wallet.
4. Backend verifies payment confirmation.
5. Trading Company provides access to the requested utility.
6. Bot applies the declared burn policy from the Trading Company wallet.
7. Transaction and burn events are logged.

The preferred model is:

```text
User pays Pera-X → Trading Company wallet → Service provided → Bot burn execution
```

This is different from immediately splitting every payment into multiple wallets. At the early stage, the Trading Company wallet should receive the payment first, then the burn and rebalancing policy should act from there.

## 5. Fiat, Card, or Non-Token Payment Flow

When a user pays with card, bank transfer, fiat, stablecoin, or any method other than Pera-X:

1. User pays through the selected payment method.
2. Backend records the equivalent Pera-X economic value.
3. Backend calculates the burn obligation using the current declared burn rate.
4. Burn is executed from the Trading Company wallet.
5. If the Trading Company wallet balance becomes low, the bot may trigger a buyback or treasury transfer according to approved policy.

This means Pera-X burn can still happen even when users do not pay directly with Pera-X, provided the business policy declares that the utility transaction contributes to token economy.

## 6. Daily Burn Policy

Pera-X uses a dynamic burn policy.

The burn rate is declared every 24 hours and must remain within this range:

```text
Minimum burn rate: 2%
Maximum burn rate: 30%
```

The daily burn rate should be selected by a market-condition bot or policy engine, not randomly.

The bot may consider:

- Market price condition.
- Liquidity depth.
- Utility usage rate.
- Holder-to-user ratio.
- Trading volume.
- Trading Company wallet balance.
- Treasury health.
- Volatility.
- Buy pressure and sell pressure.
- Ecosystem revenue.

General guidance:

- If market condition is healthy and utility usage is strong, burn rate can stay low.
- If many people are holding but utility usage is weak, burn rate may increase to strengthen token economic activity.
- If token price is under pressure, burn policy may increase, but only within safe and transparent boundaries.
- If liquidity is too thin, the bot should avoid aggressive actions that harm market stability.

## 7. Burn Execution

Burn should normally be executed from the Trading Company wallet.

The Trading Company burns the required percentage from tokens it received or from tokens held for ecosystem operations.

The burn execution must be logged with:

- Burn amount.
- Burn rate.
- Date/time.
- Reason category.
- Related utility volume.
- Trading Company wallet balance before and after burn where possible.

The burn policy should be transparent enough to build trust, but the system does not need to expose every private trading strategy.

## 8. Treasury Policy

The treasury is the long-term reserve of the ecosystem. The treasury should not be treated as a normal spending wallet.

The treasury may support:

- Market stabilization.
- Trading Company replenishment.
- Liquidity support.
- Ecosystem development.
- Strategic partnerships.
- Emergency reserve.

Recommended early structure:

```text
Trading Company wallet = operational engine
Treasury wallet = long-term reserve and controlled support wallet
Liquidity wallet = DEX liquidity and market-making support
Cliff/locked wallets = market-condition-based unlocking reserves
```

Treasury movement should be guided by policy and bot recommendation. For major treasury movement, a multisig or approval layer should be introduced before production launch.

## 9. Buyback Policy

If users pay with fiat/card/stablecoin or if the Trading Company wallet becomes low, the system may buy Pera-X from the market or receive Pera-X from treasury according to policy.

Buyback should be considered when:

- Trading Company wallet balance is below required operational threshold.
- Utility demand is higher than available token reserves.
- Market condition supports buyback without creating harmful volatility.
- Ecosystem revenue supports the action.

Buyback can support utility settlement, burn obligations, and market confidence.

## 10. Discount and Reward Policy

Pera-X should encourage utility, not only holding.

Services may already be cheap even without token discounts. Therefore, discounts should be controlled and should reward valuable behavior.

Recommended discount tiers:

```text
Basic utility user: 0%–5%
Active utility user: 5%–15%
Long-term holder + utility user: 15%–30%
Premium loyalty tier: 30%–50%
```

High discounts should not be automatic. They should depend on:

- Holding duration.
- Amount of Pera-X held.
- Real utility usage.
- User loyalty score.
- Market health.
- System profitability.
- Anti-abuse checks.

Users may receive rewards as Pera-X tokens where appropriate.

## 11. Market-Condition-Based Unlocking

Pera-X does not rely only on time-based cliff or vesting.

Unlocking should be based on market conditions and approved economic rules.

Example policy direction:

- If price reaches a defined multiple of the initial price, the bot may unlock a controlled percentage from locked wallets.
- Unlocking does not mean automatic selling.
- The Trading Company should always receive a meaningful proportion of any unlocked economic allocation calculated by the bot.
- Unlocking should avoid sudden market dumping.
- Unlock events should be logged and explainable.

A possible rule example:

```text
If price reaches 3x initial price and liquidity/volume conditions are healthy, the bot may unlock a controlled percentage in a way that supports market balance and long-term utility growth.
```

This must be implemented carefully so the policy is viewed as market stabilization and ecosystem management, not manipulation.

## 12. Bot Governance

Bots should be the primary policy execution engine.

Bots may recommend or execute:

- Daily burn rate.
- Buyback action.
- Trading Company replenishment.
- Treasury movement recommendation.
- Unlock recommendation.
- Discount tier updates.
- Risk alerts.

However, bot actions must remain inside approved policy boundaries.

Recommended controls:

- Hard minimum and maximum burn rate.
- Maximum daily treasury movement.
- Maximum unlock percentage per event.
- Emergency pause.
- Multisig confirmation for sensitive actions.
- Full event logging.

## 13. On-Chain vs Backend Responsibilities

### Smart Contract Responsibilities

The smart contract should enforce only critical trust rules:

- Token transfer collection.
- Burn execution.
- Authority control.
- Pause/unpause.
- Event logging.
- Wallet validation.
- Replay protection where needed.
- Bot signer validation where needed.

### Backend/Bot Responsibilities

The backend and bot should handle flexible economic intelligence:

- Daily burn decision.
- Market-condition scoring.
- Utility rate analysis.
- Holding behavior analysis.
- Discount calculation.
- Buyback logic.
- Treasury recommendation.
- Unlock recommendation.
- Fraud and abuse checks.
- Service provisioning.

This separation keeps the smart contract safe while allowing the business policy to evolve.

## 14. Emergency and Risk Controls

The ecosystem must include emergency controls.

Minimum controls:

- Pause utility payments.
- Pause burn execution.
- Pause unlock execution.
- Freeze bot execution if suspicious.
- Multisig approval for high-risk treasury movements.
- Audit logs for every sensitive action.

Emergency controls should not be abused. They exist to protect users, treasury, liquidity, and the long-term ecosystem.

## 15. Trader and Market Participation Program

To attract traders and liquidity participants, Pera-X may introduce a Market Participation Program.

Possible features:

- Liquidity provider rewards.
- Utility-volume rewards.
- Holding-based loyalty tiers.
- Transparent burn history.
- Buyback-and-burn reporting.
- Trading Company market support.
- Bot-governed unlocking.
- Clear market-condition policy.

The message to traders should be:

> Pera-X rewards both real utility and market participation. The token is designed to support actual usage, not only speculation.

## 16. Current Implementation Direction

The current smart contract foundation should be treated as an early enforcement layer.

Before production, the contract should be adjusted to match this policy more closely:

- Utility payments should primarily credit the Trading Company wallet.
- Burn should be executed based on bot-declared daily burn rate.
- Split-payment logic may be refactored into a Trading Company collection and controlled burn model.
- Bot signer or backend authorization should be added later.
- Replay protection should be added after backend payment reference format is finalized.
- Treasury and unlocking rules should not be finalized in code until the backend policy engine is available.

## 17. Open Decisions

The following decisions must be finalized before production:

1. Exact Trading Company wallet structure.
2. Treasury wallet structure.
3. Liquidity wallet structure.
4. Bot signer and multisig model.
5. Burn declaration format.
6. Market-condition scoring formula.
7. Discount tier formula.
8. Buyback trigger thresholds.
9. Unlock trigger thresholds.
10. Emergency governance process.

## 18. Policy Status

This document is the first formal economic policy draft for Pera-X. It should guide backend, smart contract, and treasury implementation.

Any future code related to burning, treasury, unlocking, discount, or bot execution should be checked against this document before implementation.
