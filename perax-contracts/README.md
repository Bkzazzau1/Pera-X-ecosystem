# Perax Contracts

Anchor workspace for the Perax Solana programs.

## Layout

```text
perax-contracts/
├── Anchor.toml
├── Cargo.toml
├── programs/
│   └── perax-core/
│       ├── Cargo.toml
│       └── src/lib.rs
├── migrations/
└── tests/
```

## First-Time Setup

Install the Solana and Anchor CLIs, then run:

```powershell
anchor keys sync
anchor build
anchor test
```

The current program id is a placeholder. `anchor keys sync` will update `Anchor.toml` and `declare_id!` after a real keypair exists under `target/deploy`.



## Adaptive Price Control

Growth releases use separate APC PDAs and `execute_apc_release`. The legacy growth variant of `execute_market_conditional_release` always fails with `UseApcRelease`; emergency release remains available.

APC initialization is separate from core initialization. Run `npm run plan:apc` first. Execution remains blocked while `adaptivePriceControl.policyStatus` is `pending_formal_numerical_approval`.
