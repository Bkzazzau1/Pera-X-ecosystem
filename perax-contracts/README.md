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

