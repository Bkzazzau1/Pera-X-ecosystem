# Pera-X Meteora DLMM Runtime

This package is the approved market adapter for policy-driven Pera-X utility settlement. It does not choose the settlement mode, PEX amount, burn action, vault share, or recovery action. Those outcomes are calculated and enforced by the Pera-X smart contract.

## Reproduce and verify

From this directory:

```bash
node scripts/bootstrap.mjs
npm ci --ignore-scripts --no-audit --no-fund
npm run verify
```

`bootstrap.mjs` reconstructs the reviewed TypeScript source and deterministic dependency lock from checked-in artifacts and verifies both SHA-256 checksums before installation.

## Runtime responsibilities

The runtime:

- Loads the APC-approved Meteora DLMM pool and program from the Pera-X contract.
- Refuses a configured pool, quote mint, signer, or program that differs from the on-chain policy.
- Submits a fresh APC observation before settlement planning.
- Builds only Meteora `swapExactOut2` instructions.
- Uses the contract-calculated exact PEX output and a bounded quote-token maximum.
- Replaces the SDK destination with the isolated per-settlement PEX custody account.
- Preserves the SDK account order and signer/writable metadata for contract verification.
- Uses the configured quote-token account only after checking its mint, owner, and balance.
- Refuses to fabricate TWAP, volume, or flow information while observation history is warming up.

## Required environment

```env
METEORA_DLMM_POOL=
PERAX_QUOTE_MINT_ADDRESS=
PERAX_SETTLEMENT_QUOTE_TOKEN_ACCOUNT=
METEORA_MAX_SLIPPAGE_BPS=
PERAX_OBSERVATION_TWAP_SECONDS=
PERAX_OBSERVATION_FLOW_WINDOW_SECONDS=
PERAX_OBSERVATION_PROBE_PEX_AMOUNT=
PERAX_OBSERVATION_STATE_PATH=
```

The executor also requires the variables documented in `perax-market-engine/SETTLEMENT_EXECUTOR.md`, including the Pera-X program, state PDA, PEX mint, validated IDL, signer file, bearer token, and Solana RPC endpoint.

Set the executor runtime module to the built adapter:

```env
PERAX_SETTLEMENT_RUNTIME_MODULE=/absolute/path/to/perax-market-adapters/meteora-dlmm-runtime/dist/src/index.js
```

## Observation warmup

The first observation cannot be submitted until the runtime has enough persisted reserve samples and the Meteora oracle covers the configured TWAP window. Warmup failures are retryable and do not create a settlement record.

`PERAX_OBSERVATION_STATE_PATH` must point to durable storage. Removing or replacing this file resets local reserve-flow history and forces a new warmup period.

## Security boundary

The runtime is an executor, not a policy authority. The contract independently checks:

- Approved market program and pool.
- Exact-out instruction discriminator and encoded amounts.
- Source and destination token accounts.
- PEX and quote mints.
- Signer and writable privileges.
- Token programs and Meteora event authority.
- Absence of host-fee and transfer-hook routes.
- Actual quote spent and actual PEX received after CPI.

No production activation is permitted until the APC numerical policy, recovery slippage, settlement policy, pool address, quote source, and product policies have been formally approved and initialized.
