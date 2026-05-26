# Pera-X (PEX) Deployment Checklist

This checklist must be completed before any PEX mainnet deployment, allocation transfer, liquidity creation, or market-conditional unlock operation.

## 1. Safety Rules

- Do not commit private keys, seed phrases, keypair files, `.env`, or production wallet configs.
- Production wallet config must remain outside Git.
- Use multisig or controlled authority wallets where possible.
- Confirm all addresses twice before deployment.
- Run validation scripts before deployment.

## 2. Required Local Files

Create these locally only:

```text
perax-contracts/.env
perax-contracts/config/pex-allocation-wallets.json
```

Do not commit them.

## 3. Required Commands Before Deployment

From `perax-contracts/`:

```bash
npm install
npm run validate:tokenomics
npm run typecheck
anchor test
```

Deployment should not continue if any command fails.

## 4. Token Configuration

| Item | Approved Value |
|---|---:|
| Token Name | Pera-X |
| Symbol | PEX |
| Network | Solana |
| Decimals | 6 |
| Total Supply | 1,000,000,000 PEX |
| Initial Price | $0.000012 |
| Initial Valuation | $12,000 |

## 5. Allocation Wallets Required

The following wallet addresses must be prepared before token allocation:

1. Liquidity Pool wallet.
2. Community / Utility Rewards wallet.
3. Treasury wallet.
4. Ecosystem / Marketing wallet.
5. Trading Company Operations wallet.
6. Development Team wallet.
7. Founder wallet.
8. Future Team Incentives wallet.
9. Team Emergency Reserve wallet.
10. Private / Strategic Investors wallet.
11. Advisor Wallet 1.
12. Advisor Wallet 2.
13. Advisor Wallet 3.

## 6. Allocation Confirmation

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

## 7. Team Split

| Wallet | Allocation | Token Amount |
|---|---:|---:|
| Development Team | 2% | 20,000,000 PEX |
| Founder | 2% | 20,000,000 PEX |
| Future Team Incentives | 1% | 10,000,000 PEX |
| Team Emergency Reserve | 1% | 10,000,000 PEX |

## 8. Advisor Split

| Wallet | Allocation | Token Amount |
|---|---:|---:|
| Advisor Wallet 1 | 1% | 10,000,000 PEX |
| Advisor Wallet 2 | 1% | 10,000,000 PEX |
| Advisor Wallet 3 | 1% | 10,000,000 PEX |

## 9. Initial Liquidity Guidance

Approved launch price:

```text
1 PEX = $0.000012
```

Recommended initial Meteora liquidity:

```text
250,000,000 PEX + $3,000 USDC
```

Remaining liquidity reserve:

```text
130,000,000 PEX
```

## 10. Anchor Program Initialization

Before initializing the Anchor program, confirm:

1. PEX mint address is created.
2. Trading company token account is created.
3. Program authority is correct.
4. `max_payment_amount` is set correctly.
5. Emergency pause authority is controlled.
6. Program ID is not the placeholder value.

## 11. Unlocking Policy Confirmation

Before any unlock operation, confirm:

1. Price trigger is reached.
2. TWAP confirmation is satisfied for 30–60 minutes.
3. Liquidity depth is healthy.
4. Trading volume is real.
5. Daily unlock cap is not exceeded.
6. Cooldown period is complete.
7. Unlock has a clear business or ecosystem purpose.
8. Manual or multisig approval is completed.
9. Emergency pause is not active.

## 12. Emergency Pause Conditions

Pause unlocking immediately if:

1. Market manipulation is detected.
2. Liquidity becomes weak.
3. Price crashes suddenly.
4. Abnormal sell pressure appears.
5. DEX or liquidity pool issue occurs.
6. Bot/oracle data becomes unreliable.
7. Security issue is detected.
8. Community confidence is at risk.

## 13. Final Approval

No deployment, allocation, liquidity creation, or unlock should proceed without final approval from the project authority or designated multisig process.
