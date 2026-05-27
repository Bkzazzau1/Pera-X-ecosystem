# Pera-X (PEX) Deployment Flow

This document defines the safe deployment flow for Pera-X (PEX) token setup, allocation planning, and future smart contract deployment.

The current scripts are intentionally safe. They validate and print plans only. They do not mint, transfer, burn, or create liquidity.

## 1. Required Safety Rule

Never commit local environment files, private deployment material, Solana wallet/keypair files, or production-only allocation configs.

The committed wallet file must remain only:

```text
perax-contracts/config/pex-allocation-wallets.example.json
```

## 2. Install Dependencies

From the repository root:

```bash
cd perax-contracts
npm install
```

## 3. Validate Tokenomics

Run:

```bash
npm run validate:tokenomics
```

This checks:

1. Total supply is exactly 1,000,000,000 PEX.
2. Initial price is $0.000012.
3. All allocation percentages equal 100%.
4. All allocation amounts equal the total supply.
5. Team child wallets equal 6%.
6. Advisor child wallets equal 3%.
7. Initial liquidity values are correct.
8. Unlocking policy matches the approved model.
9. Wallet template matches the tokenomics config.
10. Production wallet config is not committed.

Deployment must stop if this command fails.

## 4. Print Allocation Plan

Run:

```bash
npm run plan:allocation
```

This prints:

1. Token setup.
2. Wallet allocation plan.
3. Initial liquidity guidance.
4. Unlocking policy summary.
5. Whether the script is using the example template or production wallet config.

If it says `DRY RUN / TEMPLATE ONLY`, no real wallet addresses are loaded.

## 5. Prepare Local Environment

Copy the local environment example file and update it locally only.

Required values include:

```text
SOLANA_CLUSTER
SOLANA_RPC_URL
SOLANA_KEYPAIR_PATH
PERAX_CORE_PROGRAM_ID
PEX_MINT_ADDRESS
PEX_MINT_AUTHORITY
PEX_FREEZE_AUTHORITY
TRADING_COMPANY_TOKEN_ACCOUNT
MAX_PAYMENT_AMOUNT
```

Do not commit local environment files.

## 6. Prepare Local Production Wallet Config

Copy the example allocation wallet template to a local production wallet config, then replace all placeholder wallet addresses with real Solana public wallet addresses.

Do not commit the production wallet config. It is ignored by Git.

## 7. Run Checks Again

After preparing local files, run:

```bash
npm run validate:tokenomics
npm run plan:allocation
npm run typecheck
anchor test
```

If any command fails, deployment must stop.

## 8. Deployment Order

The recommended order is:

1. Confirm wallet addresses and authorities.
2. Confirm RPC and Solana cluster.
3. Build and test Anchor program.
4. Deploy `perax_core` program.
5. Update `Anchor.toml` with the real program ID.
6. Create PEX SPL token mint.
7. Mint total supply according to approved supply.
8. Create token accounts for allocation wallets.
9. Initialize the Pera-X core program using the PEX mint and trading company token account.
10. Allocate tokens to approved wallets.
11. Create initial liquidity on Meteora.
12. Save deployment addresses in a secure internal deployment record.
13. Update public docs only with safe public addresses.

## 9. Initial Liquidity Reference

Approved initial price:

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

## 10. Unlocking Operations

Unlocking must follow the market-conditional unlocking policy.

Before any unlock:

1. Confirm trigger price has been reached.
2. Confirm TWAP for 30–60 minutes.
3. Confirm liquidity depth.
4. Confirm real trading volume.
5. Confirm cooldown is complete.
6. Confirm daily cap is not exceeded.
7. Confirm the unlock has a business or ecosystem purpose.
8. Confirm manual or multisig approval.
9. Confirm emergency pause is not active.

## 11. Commands Summary

```bash
cd perax-contracts
npm install
npm run validate:tokenomics
npm run plan:allocation
npm run typecheck
anchor test
```

## 12. Current Status

Current implementation status:

1. Tokenomics documentation is complete.
2. Machine-readable tokenomics config is complete.
3. Wallet allocation template is complete.
4. Validation script is complete.
5. Dry-run allocation plan script is complete.
6. Environment example is complete.
7. Deployment checklist is complete.

Pending future work:

1. Real wallet addresses.
2. Mint creation script.
3. Allocation transfer script.
4. Anchor deployment script.
5. Meteora liquidity setup process.
6. Market-condition unlock bot/service.
