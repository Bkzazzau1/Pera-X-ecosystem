# Pera-X Core V2: Program-Controlled Reserve Vaults

## Scope

This upgrade changes reserve custody and release execution only. It does not alter the approved staged-price thresholds.

The existing initialized `PeraxState` account is not resized. Every allocation receives a separate `ReserveVaultConfig` PDA, a separate authority PDA, and a separate PEX associated token account owned by that authority PDA.

## PDA layout

```text
Reserve configuration: ["reserve-config", allocation_id]
Reserve authority:     ["reserve-authority", allocation_id]
Reserve token account: canonical PEX ATA owned by reserve authority PDA
Release record:        ["vault-release", release_id]
```

`allocation_id` is the allocation key encoded as UTF-8 and zero-padded to 32 bytes. The contract accepts only the 13 allocations already recorded in the public devnet deployment record and enforces their approved maximum caps.

## Vault classes

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

Liquidity and vesting vaults cannot use `execute_market_conditional_release`. They remain program-controlled but require their own future liquidity-deployment and vesting instructions.

## Instructions

### `initialize_reserve_vault`

Creates one configuration PDA, one authority PDA and one canonical PEX token account. It rejects unknown allocation IDs, incorrect classes, zero caps, caps above the approved allocation, wrong mint and duplicate initialization.

### `deposit_into_reserve_vault`

Moves already-minted PEX from a legacy allocation token account into its PDA-controlled vault. The source owner must sign. Deposits cannot exceed the approved lifetime allocation cap.

### `execute_market_conditional_release`

Performs market validation, vault validation, replay protection, accounting updates, PDA-signed PEX transfer and permanent release recording in one transaction. The destination token account is included in the oracle bot's signed instruction data and must match the provided account. The vault itself cannot be used as the destination.

Growth releases are allowed only from `MarketReserve`, `Operations` and `CommunityRewards` vaults. Emergency releases are allowed only from `EmergencyReserve` vaults.

### `set_reserve_vault_pause`

The program authority or safety admin can pause or unpause an individual vault. A pause blocks releases but does not transfer tokens.

### `reconcile_reserve_vault`

Records unsolicited direct PEX deposits by reconciling the actual token balance with lifetime deposits. It never permits reconciliation above the approved allocation cap and never hides a missing-balance discrepancy.

### Legacy route

`record_market_conditional_release` now returns `UseVaultControlledRelease`. It cannot increase release counters without moving tokens.

## Devnet migration scripts

```bash
npm run plan:reserve-vaults:devnet
npm run create:reserve-vaults:devnet
npm run plan:migrate-reserves:devnet
npm run migrate:reserves:devnet
npm run verify:reserve-vaults:devnet
```

The creation script never transfers PEX. The migration script never mints PEX and requires a local ignored signer configuration copied from:

```text
config/reserve-vault-migration.devnet.local.example.json
```

Create the real file as:

```text
config/reserve-vault-migration.devnet.local.json
```

That file is already protected by `.gitignore`. Never commit allocation keypairs.

## Required migration order

1. Build and test the upgraded program locally.
2. Upgrade the devnet program.
3. Create only the community vault configuration, but move no full allocation yet.
4. Deposit only 1,000 PEX from the existing community allocation account.
5. Execute a valid 100 PEX release.
6. Confirm replay, ordinary-wallet withdrawal, wrong destination, wrong mint, paused release and over-balance release all fail.
7. Create the final vault configurations with the approved caps.
8. Migrate each allocation with its current owner signer.
9. Run the verification script.
10. Commit only the resulting public vault-address registry.

Use these commands for the limited first trial:

```bash
node scripts/create-reserve-vaults-devnet.js --only community_utility_rewards --execute
node scripts/migrate-reserves-to-vaults-devnet.js --only community_utility_rewards --amount-pex 1000 --execute
```

The community configuration may use its final approved cap while only 1,000 PEX is deposited for the trial. After the tests pass, run the normal creation and migration commands for all remaining balances.

Do not move the full devnet balances until the trial passes.
