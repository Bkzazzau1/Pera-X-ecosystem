# Concrete Anchor Settlement Client

The market engine now contains a concrete Anchor client for the policy-driven settlement router.

## What the client enforces

`src/anchor-client.ts`:

- Validates the generated settlement IDL and deployed program address.
- Derives the Pera-X state, settlement policy, product policy, APC, observation, settlement, custody and custody-authority PDAs from the contract seeds.
- Derives the isolated settlement PEX associated token account.
- Rejects an existing settlement ID that conflicts with the original immutable plan.
- Resumes `planned`, `funding`, `ready` and `finalized` settlements without repeating completed financial stages.
- Uses the on-chain settlement record to choose the destination and market mode.
- Confirms every submitted transaction.
- Validates the policy-vault authority PDA before requesting vault funding.
- Requires explicit authority, token-account and signer bindings for direct PEX and quote-token sources.
- Passes adapter accounts with their exact writable and signer metadata.
- Rejects adapter signer requirements that are not available to the transaction.
- Creates a customer's PEX associated token account idempotently in the same finalization transaction when the standard ATA is used.

The client does not allow the executor to choose `MarketPurchase`, `PolicyVault`, `Hybrid` or `DirectPex`. Those modes come from the contract's settlement record.

## Executable service

Build the package before starting the service:

```bash
npm ci
npm run typecheck
npm test
npm run build
npm run start:executor
```

The service requires:

```text
SOLANA_RPC_URL
PERAX_PROGRAM_ID
PERAX_STATE_PDA
PEX_MINT_ADDRESS
PERAX_SETTLEMENT_IDL_PATH
PERAX_SETTLEMENT_SIGNER_PATH
PERAX_SETTLEMENT_RUNTIME_MODULE
PERAX_SETTLEMENT_EXECUTOR_TOKEN
```

Optional settings:

```text
PERAX_SETTLEMENT_COMMITMENT=confirmed
PERAX_SETTLEMENT_EXECUTOR_HOST=127.0.0.1
PERAX_SETTLEMENT_EXECUTOR_PORT=8788
```

The signer file must be a local 64-byte Solana keypair JSON array. It must never be committed to the repository.

## Runtime module boundary

The file configured by `PERAX_SETTLEMENT_RUNTIME_MODULE` must export:

```ts
export function createSettlementRuntime(context) {
  return {
    venue,
    observations,
    resolveQuoteSource,
    resolveDirectPexSource,       // optional when direct PEX is handled by users
    resolveCustomerDestination,  // optional; standard ATA is the default
    isTerminalError,              // optional
  };
}
```

The runtime must provide:

- A fresh APC observation provider.
- An atomic market adapter that returns the exact instruction data and account metadata for the approved on-chain market program and pool.
- A quote-token source authority and token account capable of signing the outer transaction.

The runtime is intentionally injected. A Meteora or other adapter must not be guessed from a generic pool address. Its exact instruction and account specification must be approved and tested first.

## Production activation block

Do not start the executor in production until:

1. The latest Anchor build has generated the validated settlement IDL.
2. The configured program ID and derived state PDA match that IDL and deployment.
3. The approved market program and pool are initialized in the settlement policy.
4. The adapter runtime uses that exact program and pool.
5. Settlement numerical policy and product policies are approved.
6. Contract, market-engine and local-validator tests pass.
7. The signer and quote-source accounts contain only the policy-approved operational balances.

No deployment or on-chain initialization is performed by this source implementation.
