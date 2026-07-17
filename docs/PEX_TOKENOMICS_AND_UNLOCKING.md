# Pera-X Tokenomics and Adaptive Price Control

## Token parameters

PEX is a Solana token with a fixed supply of 1,000,000,000 PEX, six decimals, and an approved launch price of `$0.000012`. The full 380,000,000 PEX liquidity allocation is documented as deployed at launch; APC never draws inventory from the liquidity allocation.

## Reserve custody

Growth inventory remains in program-controlled reserve vaults created under Correction 1. Only Market Reserve, Operations, and Community Rewards vault classes may use APC. Emergency Reserve remains on the emergency route. Liquidity and vesting vaults cannot use APC.

Every release preserves the approved allocation ID, exact authorized inventory, approved ordinary destination, permanent release record, allocation ceiling, unsolicited-token exclusion, and atomic SPL transfer.

## Adaptive Price Control

The first activation is exactly:

```text
$0.000012 × 3 = $0.000036
```

The fixed multiplier ends after the first activation. Later bands are derived by the program from the previous reference price, immutable risk policy, and a fresh signed market observation.

APC uses separate PDAs for configuration, mutable progression, observations, bands, releases, counterweight deposits, deferred burns, and recovery settlements. The legacy `current_stepped_floor` field remains only for account-layout compatibility and cannot change APC progression.

## Sequential multi-band pumps

A single fresh observation may cross several bands. Each band is activated by one bounded instruction and receives a unique immutable PDA. Bands cannot be skipped, recreated, or restored after a price decline.

`AwaitingAbsorption` does not block band activation. It preserves unconfirmed-release exposure, reduces the safe release surface through the band and cascade caps, and prevents time-window resets from restoring unabsorbed capacity. A distinct fresh observation must confirm support before unconfirmed exposure is cleared.

## Observations

The oracle signer supplies market inputs, not final policy decisions. The program verifies the approved pool, nonzero observation ID, strictly increasing sequence, trusted Solana clock freshness, future-clock skew, prices, liquidity, volume, buy pressure, and bounded risk metrics.

The program derives the effective price, risk tier, band interval, next trigger, release ceiling, cascade reduction, and counterweight requirement with checked `u128` arithmetic.

## Release ceilings

Every APC release is limited simultaneously by:

- Permanent per-band capacity.
- Unconfirmed absorption exposure.
- Hourly capacity.
- Pump-window capacity.
- Existing global daily and monthly caps.
- Authorized reserve-vault inventory.
- Approved destination and vault class.
- Actual counterweight coverage.

The smallest applicable capacity wins. Checked arithmetic is mandatory.

## Counterweight custody

Counterweight credit is created only after a real `transfer_checked` moves USDC from the approved proceeds account into the PDA-controlled Counterweight Vault. A bot-reported balance is never accepted.

After a PEX release, later releases stop unless actual credited USDC satisfies the configured coverage requirement.

## Burn deferral

Immediate market burns are blocked in Pump Control, Awaiting Absorption, and Recovery. The obligated PEX is transferred into a PDA-controlled Deferred Burn Vault and recorded permanently. It may burn gradually only after APC returns to Armed or Active.

## Recovery

Recovery starts only from a fresh approved-pool observation below the current APC reference price. Counterweight USDC can be spent only through the immutable approved adapter. The core program verifies an actual USDC decrease and actual PEX increase in the locked Recovery Vault in the same transaction.

The repository also contains a constant-product atomic recovery adapter suitable for deterministic local validation or an approved custom pool. A production Meteora adapter may be approved later without weakening the core settlement checks.

Recovered PEX remains locked. It is not automatically burned or returned to circulation.

## Authority model

Routine APC observation, band activation, and release execution are autonomous. No manual or multisig approval is added. Safety authorities may pause the system but cannot authorize reserve releases or rewrite APC reference prices.

## Numerical approval gate

The following remain pending formal approval and are deliberately `null` in the machine-readable production policy:

1. Risk-tier thresholds.
2. Risk-to-interval formula values.
3. Normal tranche values.
4. Cascade reductions.
5. Pump-window duration.
6. Pump-window release cap.
7. Counterweight proceeds allocation.
8. Minimum counterweight coverage.
9. Deferred-burn resumption rate.
10. Recovery spending bands and limits.

APC initialization and deployment remain blocked until these values are formally approved and inserted through the separate initializer.
