# Pera-X Ecosystem

Pera-X is a Solana-based utility token and Web2/Web3 ecosystem project.

The project separates the Web2 gateway from the Web3 smart contract layer:

```text
perax-ecosystem/
├── perax-gateway/      # Reserved for Axum backend services
├── perax-contracts/    # Anchor workspace for Solana programs and PEX tooling
├── docs/               # Tokenomics, deployment, and policy documents
└── scripts/            # Workstation setup scripts
```

## Core Model

Pera-X uses a two-balance model:

```text
PEX = ecosystem token / asset
Credits = internal platform spending balance
```

Users may buy Credits using PEX, card, stablecoin, or eligible-country virtual accounts. Platform services such as calls, virtual numbers, data, bills, AI tools, and other utilities spend Credits. PEX remains the ecosystem token for holding, rewards, discounts, buyback-and-burn participation, liquidity support, and ecosystem value.

## Approved PEX Token Parameters

| Parameter | Value |
|---|---:|
| Token Name | Pera-X |
| Symbol | PEX |
| Network | Solana |
| Total Supply | 1,000,000,000 PEX |
| Decimals | 6 |
| Initial Price | $0.000012 |
| Initial Valuation | $12,000 |

## Approved Allocation Summary

| Category | Allocation | Token Amount |
|---|---:|---:|
| Liquidity Pool | 38% | 380,000,000 PEX |
| Community / Utility Rewards | 17% | 170,000,000 PEX |
| Treasury | 12% | 120,000,000 PEX |
| Ecosystem / Marketing | 12% | 120,000,000 PEX |
| Trading Company Operations & Revenue Settlement | 7% | 70,000,000 PEX |
| Team | 6% | 60,000,000 PEX |
| Private / Strategic Investors | 5% | 50,000,000 PEX |
| Advisors | 3% | 30,000,000 PEX |
| **Total** | **100%** | **1,000,000,000 PEX** |

## Initial Liquidity Guidance

At the approved initial price:

```text
1 PEX = $0.000012
```

Recommended initial Meteora liquidity:

```text
380,000,000 PEX + $4,560 USDC
```

Remaining liquidity reserve:

```text
0 PEX
```

This uses the full 38% liquidity allocation at the approved launch price of $0.000012.

## Adaptive Price Control

Pera-X reserve releases use Adaptive Price Control (APC). The first activation remains exactly three times the launch price: `$0.000036`. After that point, the contract derives each sequential band from immutable APC policy and fresh signed market observations; it does not reuse a fixed price multiplier.

Routine APC releases are autonomous and require neither manual nor multisig approval. The program enforces permanent observation, band, and release PDAs; existing reserve-vault custody; hourly, pump-window, daily, monthly, and per-band caps; actual USDC counterweight custody; deferred-burn escrow during pump protection; and atomic approved-adapter recovery into a locked PEX vault. Safety administrators may pause the system but cannot authorize routine releases.

The ten numerical APC policy inputs remain explicitly pending formal approval and are not represented as production values in the machine-readable tokenomics file.

## Important Documents

| Document | Purpose |
|---|---|
| `docs/PEX_TOKENOMICS_AND_UNLOCKING.md` | PEX tokenomics and APC principles. |
| `docs/APC_LOGIC_SPECIFICATION.md` | Contract, custody, observation, band, burn, and recovery specification. |
| `docs/APC_VALIDATION_STATUS.md` | Source validation status and remaining deployment gates. |
| `docs/PEX_DEPLOYMENT_CHECKLIST.md` | Pre-deployment checklist and safety process. |
| `perax-contracts/DEPLOYMENT_FLOW.md` | Command flow for validation, planning, and future deployment. |
| `perax-contracts/config/pex-tokenomics.json` | Machine-readable PEX tokenomics policy. |
| `perax-contracts/config/pex-allocation-wallets.example.json` | Safe wallet allocation template with placeholder addresses only. |

## Contract Workspace

`perax-contracts` is an Anchor workspace for the Solana smart contract layer.

The current core program supports:

1. Pera-X state initialization.
2. Trading company token account configuration.
3. Utility payment transfer to the trading company token account.
4. External utility payment recording.
5. Trading company burn execution.
6. Pause controls.
7. Authority transfer controls.
8. Adaptive Price Control configuration, observations, sequential bands, and reserve releases.
9. PDA-controlled USDC counterweight custody and deferred-burn PEX custody.
10. Approved atomic recovery-adapter execution into a locked Recovery Vault.

## Safe Commands

From `perax-contracts/`:

```bash
npm install
npm run validate:tokenomics
npm run plan:allocation
npm run typecheck
anchor test
```

Deployment must stop if any command fails.

## Security Rules

Never commit:

```text
.env
private keys
seed phrases
Solana keypairs
production wallet config
```

The production wallet config must remain local only:

```text
perax-contracts/config/pex-allocation-wallets.json
```

The committed wallet file must remain only:

```text
perax-contracts/config/pex-allocation-wallets.example.json
```

## WSL Rust Workstation

Use WSL2 Ubuntu as the standard development environment for Rust, Anchor, Solana, Redis, and backend services.

From an elevated PowerShell window:

```powershell
.\scripts\setup-wsl.ps1
```

If Ubuntu fails during VM creation with an HCS error, run the repair script from an elevated PowerShell window and reboot:

```powershell
.\scripts\repair-wsl-hcs.ps1
```

After Ubuntu opens and your Linux user is created:

```bash
cd /mnt/c/PROJECTS/"smartcontract PEX"/perax-ecosystem
bash scripts/bootstrap-ubuntu.sh
```

Optional: copy `.wslconfig.example` to `%UserProfile%\.wslconfig`, then run `wsl --shutdown` to apply memory/CPU limits.

## Current Status

Source-complete, pending formal numerical approval and deployment review. No APC account has been initialized on-chain.

Completed:

1. Tokenomics documentation.
2. Market-conditional unlocking policy.
3. Machine-readable tokenomics config.
4. Wallet allocation template.
5. Tokenomics validation script.
6. Allocation plan dry-run script.
7. Environment example.
8. Deployment checklist.
9. Deployment flow guide.
10. Secret and production wallet `.gitignore` protection.

Pending:

1. Real wallet addresses.
2. PEX mint creation script.
3. Allocation transfer script.
4. Anchor deployment script.
5. Meteora liquidity setup process.
6. Formal approval of the ten APC numerical policies before initialization.

<!-- APC_POLICY_V1_SYNC -->
## APC Numerical Policy Version 1

Correction 2 uses immutable APC Policy V1, policy hash `17f93bacb0cfa5346a466258117908068f1f0cd67054f8b61c7d40818dfe84bb`. The canonical machine-readable source is `perax-contracts/config/apc-policy-v1.json`; contract initialization rejects every numerical or hash difference. The deterministic selection evaluated 2,916 candidates, 1,920 market scenarios and 25,000 randomized invariant cases. The selected policy produced a 498-bps worst modeled APC-added impact and 1.365× minimum counterweight coverage.

Key controls: 750–2,000 bps adaptive bands; 2,000,000 PEX base band cap; 2,500,000 PEX hourly cap; 6,000,000 PEX six-hour pump cap; 70% counterweight allocation; 10% deferred-burn resumption; 30% protected recovery reserve; and four drawdown support bands at 10%, 25%, 50% and 75%.

No deployment, initialization, reserve movement or migration is authorized by this policy approval. Runtime freeze still requires the complete validation pipeline and independent security approval.
