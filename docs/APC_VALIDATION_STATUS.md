# APC Validation Status

## Source scope

Correction 2 implements separate APC state, permanent observations and bands, APC release records, old-growth-route shutdown, real USDC counterweight custody, deferred-burn custody, atomic recovery settlement, a locked Recovery Vault, machine-readable policy validation, initialization planning, and an off-chain market-engine service.

## Required checks

The implementation is accepted only when all of the following succeed on the final branch:

```text
npm run validate:tokenomics
npm run typecheck
cargo test --locked --all-targets
cargo check --locked --all-targets
anchor build
anchor test --provider.cluster localnet
npm --prefix ../perax-market-engine install
npm --prefix ../perax-market-engine run typecheck
npm --prefix ../perax-market-engine test
```

## Deployment status

No program deployment, migration, mint, authority change, APC initialization, reserve transfer, counterweight transfer, or recovery swap is performed by this source correction.

The machine-readable APC numerical policy remains pending formal approval. Deployment and initialization scripts must refuse execution while that status remains pending.
