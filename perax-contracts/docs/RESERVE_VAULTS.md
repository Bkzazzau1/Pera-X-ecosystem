# Pera-X Core: Program-Controlled Reserve Vaults

## Current status

This document describes source code under active development. The hardened reserve-vault program has not yet been activated on devnet, and existing devnet reserve balances must be treated as remaining in their legacy token accounts until the public registry and deployment record prove otherwise.

## PDA layout

```text
Reserve configuration: ["reserve-config", allocation_id]
Reserve authority:     ["reserve-authority", allocation_id]
Reserve token account: canonical PEX ATA owned by reserve authority PDA
Release record:        ["vault-release", release_id]
```

Every approved allocation receives a separate `ReserveVaultConfig`, authority PDA and PEX token account. The program accepts only the 13 allocation IDs and maximum caps recorded in the public deployment record.

## Custody policy

Each configuration stores:

- `authorized_source_owner`
- `authorized_source_token_account`
- `approved_destination_owner`
- `approved_destination_token_account`
- `authorized_deposited`
- `unsolicited_balance`
- `total_released`

Migration deposits succeed only when the configured source owner signs and the exact configured legacy PEX token account is supplied. Deposits from another allocation account cannot consume the vault's allocation cap.

Direct SPL-token transfers cannot be prevented. They are therefore recorded separately as `unsolicited_balance`. They never increase `authorized_deposited` or the amount the program may release for that allocation.

## Approved destinations

Market-releasable vaults must be initialized with one approved ordinary-wallet owner and its exact PEX token account. A release must match both values. A PDA-owned destination, including another reserve vault, is rejected. After a valid release, the approved destination owner may transfer or sell the released PEX normally.

Liquidity and vesting allocations do not receive an ordinary market-release destination and cannot use `execute_market_conditional_release`.

## Vault classes and approved caps

| Allocation | Vault class | Approved cap |
|---|---|---:|
| liquidity_pool | Liquidity | 380,000,000 PEX |
| community_utility_rewards | CommunityRewards | 170,000,000 PEX |
| treasury | MarketReserve | 120,000,000 PEX |
| ecosystem_marketing | MarketReserve | 120,000,000 PEX |
| trading_company_operations | Operations | 70,000,000 PEX |
| development_team | Vesting | 20,000,000 PEX |
| founder | Vesting | 20,000,000 PEX |
| future_team_incentives | MarketReserve | 10,000,000 PEX |
| team_emergency_reserve | EmergencyReserve | 10,000,000 PEX |
| private_strategic_investors | Vesting | 50,000,000 PEX |
| advisor_wallet_1 | Vesting | 10,000,000 PEX |
| advisor_wallet_2 | Vesting | 10,000,000 PEX |
| advisor_wallet_3 | Vesting | 10,000,000 PEX |

Growth releases are limited to `MarketReserve`, `Operations` and `CommunityRewards`. Emergency releases are limited to `EmergencyReserve`.

## Atomic release

`execute_market_conditional_release` validates the oracle signer, market snapshot, vault class, authorized available balance, approved destination and replay ID. It then performs the PDA-signed PEX transfer, updates state and vault accounting, and creates the permanent release record in the same Solana transaction. Any failure rolls back every operation.

The legacy approval-only instruction always returns `UseVaultControlledRelease`.

## Local and CI validation

The repository CI is configured to run:

```bash
npm install
npm run validate:tokenomics
anchor build
cargo test --manifest-path programs/perax-core/Cargo.toml
npm run typecheck
anchor test --provider.cluster localnet
```

A passing workflow is required before any devnet program update or token migration.

## Devnet preparation

Copy the ignored configuration template:

```text
config/reserve-vault-migration.devnet.local.example.json
```

Create:

```text
config/reserve-vault-migration.devnet.local.json
```

The local file must contain the rotated allocation signer paths and approved release destinations. Never commit keypairs or the populated local configuration.

## Required activation order

1. Rotate every keypair previously exposed in repository history.
2. Confirm repository history and current branches no longer expose key material.
3. Obtain a passing Anchor build, Rust test, TypeScript check and local-validator transaction test.
4. Update the devnet program using the new secure upgrade authority.
5. Initialize only the community vault with its full approved cap.
6. Deposit exactly 1,000 PEX from its configured legacy source account.
7. Execute a valid 100 PEX release to its approved ordinary-wallet destination.
8. Confirm all negative and rollback tests.
9. Initialize all 13 vaults.
10. Migrate the remaining authorized devnet balances.
11. Run `verify-reserve-vaults-devnet.js`.
12. Commit only the safe public vault registry and update the public deployment record.

Do not migrate full balances before the limited community trial passes.
