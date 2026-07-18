# Pera-X Settlement Executor

## Role

The market-engine package now provides an authenticated HTTP settlement service at:

```text
POST /execute/settlement
```

The service validates the configured Solana RPC, Pera-X program, core-state PDA, and PEX mint. It obtains a fresh APC observation, passes factual order data to `SettlementCoordinator`, and returns only a finalized settlement result containing the settlement-record address and transaction signature.

The HTTP request does not contain a market mode, release percentage, reserve amount, or burn decision. Those values come from the on-chain settlement record.

## Required dependency modules

The service intentionally uses dependency injection for three deployment-specific components:

### `SettlementProgramClient`

This client must be generated against the current Anchor IDL and must implement:

- `planSettlement`
- `fundDirectPex`
- `executeMarketPurchase`
- `executePolicyVaultFunding`
- `finalizeSettlement`

Each method must read the resulting on-chain settlement record and return the contract-recorded mode, required amounts, status, record address, and confirmed transaction signature.

The IDL must be regenerated from the current source before implementing this client. An older IDL must not be used because the settlement instructions and `SettlementCustody` accounts did not exist previously.

### `SettlementExecutionVenue`

This module builds the instruction for the exact executable adapter program and market pool stored in `SettlementPolicy`.

It must return:

- maximum quote amount
- minimum PEX output at least equal to the contract requirement
- adapter instruction data
- required remaining accounts

The venue cannot choose the settlement mode. It receives the contract-recorded settlement and the exact remaining market PEX requirement.

A direct Meteora or other DEX instruction layout must not be guessed. The approved canonical adapter must document its accounts and instruction encoding, and that specification must match the program configured in APC and SettlementPolicy.

### `SettlementObservationProvider`

This module supplies a fresh, already-submitted APC observation ID. The executor rejects IDs that are not exactly 32 bytes, and the contract independently enforces observation freshness and the approved oracle feed.

## Authentication

The executor requires a bearer token of at least 24 characters and compares it with `timingSafeEqual`.

The backend should configure:

```env
PERAX_SETTLEMENT_EXECUTOR_URL=http://127.0.0.1:8790
PERAX_SETTLEMENT_EXECUTOR_TOKEN=replace-with-a-private-service-token
PERAX_SETTLEMENT_INTERVAL_SECONDS=30
```

The executor service must use the same token and should be reachable only over a private network or authenticated service mesh.

## Error classification

By default, executor errors are retryable. A deployment may provide `isTerminalError`, but it should return `true` only for permanent failures such as an inactive product policy that will not be repaired for the order.

Transport errors, unavailable RPC, stale observation, temporary liquidity shortage, adapter downtime, and unconfirmed transactions must remain retryable. Incorrectly classifying them as terminal can trigger a Credits refund.

## Production activation block

Do not enable the backend settlement worker until all of the following exist:

1. A successful Anchor build has generated the current IDL.
2. A tested `SettlementProgramClient` uses that IDL.
3. The approved canonical atomic adapter program is deployed.
4. The approved pool and adapter account specification are fixed.
5. The `SettlementExecutionVenue` has transaction tests against that adapter.
6. The observation provider submits fresh approved-pool observations.
7. Product settlement policies are initialized from the SHA-256 service identifiers used by the backend.
8. Contract, market-engine, and backend CI pass.
9. Local-validator end-to-end tests cover direct PEX, market, vault, hybrid, burn, lock, retry, and terminal refund paths.
10. Production numerical policies and custody accounts are approved.

The generic executor service is source-complete, but it cannot safely sign real settlements until the generated IDL and exact adapter specification are supplied.
