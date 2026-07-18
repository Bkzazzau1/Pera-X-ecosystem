# Pera-X Policy-Enforced Settlement Router

## Purpose

The settlement router connects the existing product catalog, PEX payments, fiat/stablecoin funding, APC observations, reserve vaults, Counterweight recovery custody, burn, lock, and customer-delivery flows without giving the market bot discretion over the final source split.

The bot submits factual inputs and executes the contract-derived plan. It cannot select `MarketPurchase`, `PolicyVault`, or `Hybrid` after the plan is created.

## Settlement lifecycle

```text
Authoritative product policy
        +
Fresh APC observation
        +
Factual funding method
        ↓
plan_settlement
        ↓
Contract derives:
- quote value
- PEX obligation
- risk tier
- market/vault split
- final disposition
        ↓
Dedicated SettlementCustody PDA + dedicated PEX ATA
        ↓
Direct PEX funding, atomic market purchase, policy-vault funding, or hybrid
        ↓
finalize_settlement
        ↓
Utility destination, customer wallet, burn, or locked recovery vault
```

Every settlement uses its own custody PDA and PEX token account. PEX held for one order cannot be consumed by another concurrent order.

## Contract-derived market modes

### DirectPex

Used only when the factual funding method is PEX. The payer transfers PEX directly into the settlement's isolated token vault. No market purchase or policy-vault release occurs.

### MarketPurchase

Used when policy requires all PEX to be sourced from the approved market adapter. Recovery always selects this mode. An effective price below the protected APC reference also selects this mode.

### PolicyVault

Used when the immutable risk-share policy assigns zero market share. Only the configured active `MarketReserve` vault can fund this route.

### Hybrid

The contract divides the PEX obligation between the approved atomic market adapter and the approved `MarketReserve` vault. The bot follows the recorded amounts exactly.

## Pump protection

For non-PEX settlements, `PumpControl` and `AwaitingAbsorption` reject planning. The system does not chase an overheated price or release reserve inventory during the absorption window.

Direct PEX product payments remain possible because they do not require a new market purchase or policy-vault release.

## Atomic market purchase requirements

`execute_settlement_market_purchase` accepts only the immutable program and pool copied from the APC configuration at settlement-policy initialization.

The instruction:

1. Verifies the original APC observation is still fresh.
2. Recalculates the effective price and requires it to match the planned price.
3. Calculates the allowed quote spend and immutable slippage ceiling.
4. Calls the approved adapter through CPI.
5. Reloads the quote source account and isolated PEX vault.
6. Requires an actual quote-token decrease.
7. Requires an actual PEX increase at or above the contract-required minimum.
8. Applies daily quote-spend and PEX-receipt caps.

A reported swap, database entry, or bot-supplied received amount cannot finalize a market purchase.

## Policy-vault funding

The policy-vault route accepts only the configured `MarketReserve` account. It uses reserve-vault accounting to exclude unsolicited balances and transfers only the contract-derived remaining amount with the reserve-authority PDA.

The router updates both reserve `total_released` accounting and its own daily policy-vault cap.

## Final dispositions

- `UtilityPayment`: sends the exact obligation to the fixed token account stored in the product policy.
- `CustomerDelivery`: sends the exact obligation to a PEX account owned by the recorded beneficiary.
- `Burn`: burns the exact obligation from isolated custody.
- `Lock`: transfers all acquired PEX into the configured locked recovery vault.

Any PEX received above the obligation is transferred to the locked recovery vault rather than becoming untracked inventory.

## Product identifiers

Backend product IDs are converted to the on-chain `[u8; 32]` identifier with SHA-256 over the UTF-8 service code.

```text
SHA256(service_code UTF-8 bytes)
```

The same derivation must be used when initializing `ProductSettlementPolicy` and when creating backend checkout orders.

## Initialization order

1. Initialize the core Pera-X state.
2. Initialize exact reserve vaults.
3. Initialize APC and Counterweight configuration.
4. Configure the approved executable atomic adapter and approved pool in APC.
5. Ensure an active `MarketReserve` vault exists for settlement policy.
6. Initialize the settlement policy once.
7. Initialize one product policy for every supported service code.
8. Generate the updated IDL and connect the market-engine `SettlementProgramClient`.
9. Connect a venue implementation that builds the approved adapter instruction.
10. Run source guards, Rust tests, Anchor build, market-engine typecheck/tests, and local-validator transaction tests.

## Production activation block

Source presence does not authorize production deployment. Production activation remains blocked until all of the following are approved and tested:

- `market_share_bps_by_risk`
- maximum market slippage
- per-settlement quantity limit
- daily market quote cap
- daily market PEX cap
- daily policy-vault PEX cap
- approved executable adapter program
- approved market pool
- approved `MarketReserve` configuration
- every product price, funding mask, quantity cap, disposition, and fixed destination
- production quote source custody
- generated IDL and program client
- successful Anchor build and local-validator transaction tests
- independent smart-contract security review

No placeholder numerical policy should be deployed as a production policy.

## Validation commands

```bash
cd perax-contracts
npm install
npm run validate:settlement
cargo +1.79.0 test --locked --manifest-path programs/perax-core/Cargo.toml
cargo +1.79.0 check --locked --all-targets
RUSTUP_TOOLCHAIN=1.85.0 anchor build

cd ../perax-market-engine
npm ci
npm run typecheck
npm test
```
