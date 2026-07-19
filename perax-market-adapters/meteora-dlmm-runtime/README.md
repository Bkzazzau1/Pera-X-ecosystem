# Pera-X Meteora DLMM Runtime

This isolated runtime executes policy-driven Pera-X market purchases on the APC-approved Meteora DLMM pool. It cannot choose settlement mode, PEX obligation, burn disposition, vault share, recovery entry, or policy limits; those decisions are enforced by the Pera-X smart contract.

## Install and verify

```bash
npm ci --ignore-scripts --no-audit --no-fund
npm run verify
```

The reviewed TypeScript source, tests, local `bigint-buffer` replacement, and deterministic `package-lock.json` are committed directly. No generated or compressed source bootstrap is required.

## Runtime responsibilities

- Load only the market program, pool, quote mint, and oracle signer approved on-chain.
- Submit fresh APC observations from the approved pool and durable reserve-flow history.
- Refuse to fabricate TWAP, volume, buy-pressure, volatility, or price-impact information during warmup.
- Build only Meteora `swapExactOut2` instructions.
- Preserve ordered market accounts and signer/writable metadata for independent contract validation.
- Use the contract-calculated exact PEX output and bounded quote maximum.
- Resolve only a configured quote account whose mint, owner, and balance are valid.

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

The settlement executor variables are documented in `perax-market-engine/SETTLEMENT_EXECUTOR.md`.

## Security boundary

The smart contract independently verifies the approved program and pool, exact-out discriminator and amounts, source and destination accounts, token mints and programs, signer privileges, Meteora event authority, absence of host fees and transfer-hook slices, actual quote spent, and actual PEX received.

Production activation remains blocked until the APC numerical policy, settlement policy, pool, quote source, and product policies are formally approved and initialized.
