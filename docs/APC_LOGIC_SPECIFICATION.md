# APC Logic Specification

## Accounts and seeds

| Account | Seed |
|---|---|
| `ApcConfig` | `apc-config`, core state |
| `ApcState` | `apc-state`, APC config |
| `ApcObservation` | `apc-observation`, observation ID |
| `ApcBandRecord` | `apc-band`, APC state, band index |
| `ApcReleaseRecord` | `apc-release`, release ID |
| `CounterweightConfig` | `counterweight-config`, APC config |
| `CounterweightDepositRecord` | `counterweight-deposit`, deposit ID |
| `DeferredBurnRecord` | `deferred-burn`, decision ID |
| `ApcRecoveryRecord` | `apc-recovery`, recovery ID |
| `RecoveryPoolConfig` | `recovery-pool`, pool ID |

## State machine

```text
Inactive → Armed → Active
                    ├─ PumpControl
                    ├─ AwaitingAbsorption
                    └─ Recovery
Any live state → Paused → exact previous state
```

`confirm_apc_absorption` uses a distinct fresh observation at or above the current reference price. It clears unconfirmed release exposure and returns Pump Control, Awaiting Absorption, or Recovery to Active.

## Deterministic band calculation

Band 1 trigger is exactly three times launch price. Every later trigger is the previous immutable trigger plus the contract-calculated interval. The effective price is the lower of spot and TWAP.

Risk tier is the maximum tier reached by velocity, volatility, or estimated impact. The contract requires both the interval and release arrays to be non-increasing as risk rises, selects the response from those immutable arrays, and applies the monotonic cascade reduction. All multiplication and division uses checked `u128` arithmetic and explicit rounding.

## Observation lifecycle

An observation PDA is permanent. Sequence increases globally. Multi-band activation may reuse one unconsumed observation. Release, absorption confirmation, recovery entry, and recovery purchase consume observations for their respective safety purpose; a consumed observation cannot authorize a later release or recovery settlement.

## APC release transaction

1. Validate APC, vault, destination, mint, pool, signer, observation, and band.
2. Require the effective price to support both the selected band and the highest crossed APC reference, then reset hourly, pump, daily, and monthly windows from `Clock::get()`.
3. Calculate all caps and counterweight coverage.
4. Transfer PEX atomically from the Correction 1 reserve PDA.
5. Update core, APC, band, and vault accounting with checked arithmetic.
6. Consume the observation and create the permanent release record.
7. Enter Awaiting Absorption or remain in Pump Control.

## Counterweight and recovery

USDC credit follows a real SPL transfer. Recovery invokes only the immutable approved executable adapter. Before and after balances are reloaded; a recovery record is created only when USDC actually decreased and the locked PEX vault actually increased within the permitted limits. Every recovery purchase is also constrained by an immutable percentage cap, protected reserve floor, trusted-clock spending window, cooldown, and the cumulative recovery cap. Deferred burns share the global daily burn cap and additionally use an immutable execution window and cooldown.

The built-in adapter is a constant-product exact-input swap with a bounded fee and minimum-output check. Its PEX pool vault is controlled by a program PDA.

## Legacy compatibility

`ReleaseType::Growth` always returns `UseApcRelease`. Emergency release remains unchanged. `current_stepped_floor` cannot be updated and is never read by APC.

<!-- APC_POLICY_V1_SYNC -->
## APC Numerical Policy Version 1

Correction 2 uses immutable APC Policy V1, policy hash `17f93bacb0cfa5346a466258117908068f1f0cd67054f8b61c7d40818dfe84bb`. The canonical machine-readable source is `perax-contracts/config/apc-policy-v1.json`; contract initialization rejects every numerical or hash difference. The deterministic selection evaluated 2,916 candidates, 1,920 market scenarios and 25,000 randomized invariant cases. The selected policy produced a 498-bps worst modeled APC-added impact and 1.365× minimum counterweight coverage.

Key controls: 750–2,000 bps adaptive bands; 2,000,000 PEX base band cap; 2,500,000 PEX hourly cap; 6,000,000 PEX six-hour pump cap; 70% counterweight allocation; 10% deferred-burn resumption; 30% protected recovery reserve; and four drawdown support bands at 10%, 25%, 50% and 75%.

No deployment, initialization, reserve movement or migration is authorized by this policy approval. Runtime freeze still requires the complete validation pipeline and independent security approval.
