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

<!-- APC_POLICY_V1_SYNC -->
## APC Numerical Policy Version 1

Correction 2 uses immutable APC Policy V1, policy hash `17f93bacb0cfa5346a466258117908068f1f0cd67054f8b61c7d40818dfe84bb`. The canonical machine-readable source is `perax-contracts/config/apc-policy-v1.json`; contract initialization rejects every numerical or hash difference. The deterministic selection evaluated 2,916 candidates, 1,920 market scenarios and 25,000 randomized invariant cases. The selected policy produced a 498-bps worst modeled APC-added impact and 1.365× minimum counterweight coverage.

Key controls: 750–2,000 bps adaptive bands; 2,000,000 PEX base band cap; 2,500,000 PEX hourly cap; 6,000,000 PEX six-hour pump cap; 70% counterweight allocation; 10% deferred-burn resumption; 30% protected recovery reserve; and four drawdown support bands at 10%, 25%, 50% and 75%.

No deployment, initialization, reserve movement or migration is authorized by this policy approval. Runtime freeze still requires the complete validation pipeline and independent security approval.
