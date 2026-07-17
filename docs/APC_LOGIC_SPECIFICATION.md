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

Risk tier is the maximum tier reached by velocity, volatility, or estimated impact. The contract selects the interval and base release percentage from immutable arrays and applies the monotonic cascade reduction. All multiplication and division uses checked `u128` arithmetic and explicit rounding.

## Observation lifecycle

An observation PDA is permanent. Sequence increases globally. Multi-band activation may reuse one unconsumed observation. Release, absorption confirmation, recovery entry, and recovery purchase consume observations for their respective safety purpose; a consumed observation cannot authorize a later release or recovery settlement.

## APC release transaction

1. Validate APC, vault, destination, mint, pool, signer, observation, and band.
2. Reset hourly, pump, daily, and monthly windows from `Clock::get()`.
3. Calculate all caps and counterweight coverage.
4. Transfer PEX atomically from the Correction 1 reserve PDA.
5. Update core, APC, band, and vault accounting with checked arithmetic.
6. Consume the observation and create the permanent release record.
7. Enter Awaiting Absorption or remain in Pump Control.

## Counterweight and recovery

USDC credit follows a real SPL transfer. Recovery invokes only the immutable approved executable adapter. Before and after balances are reloaded; a recovery record is created only when USDC actually decreased and the locked PEX vault actually increased within the permitted limits.

The built-in adapter is a constant-product exact-input swap with a bounded fee and minimum-output check. Its PEX pool vault is controlled by a program PDA.

## Legacy compatibility

`ReleaseType::Growth` always returns `UseApcRelease`. Emergency release remains unchanged. `current_stepped_floor` cannot be updated and is never read by APC.
