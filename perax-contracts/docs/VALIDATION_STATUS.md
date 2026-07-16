# Reserve Vault Validation Status

## Source hardening completed

- Exact authorized source owner and legacy PEX token account are stored per allocation.
- Migration deposits require an exact source match and signer.
- Authorized deposits, unsolicited balance and released totals are accounted separately.
- Unsolicited direct transfers do not increase releasable allocation inventory.
- Market releases require the configured ordinary-wallet owner and exact PEX token account.
- PDA-owned destinations, including another reserve vault, are rejected.
- The legacy approval-only release route remains disabled.
- Transaction tests cover all 13 allocation IDs, source enforcement, reconciliation, destination enforcement, pause controls, class restrictions, replay and transaction rollback.
- Root and contract ignore rules block `.local/` directories.
- The tracked `.local` directory has been removed from the current `main` tree.
- JavaScript migration and verification scripts pass `node --check` in the available offline environment.
- Rust and TypeScript sources pass structural delimiter checks in the available offline environment.
- The Anchor transaction test harness uses explicit helper types and a narrow account namespace adapter instead of recursive `Parameters<...>` inference.
- The dynamically generated Anchor workspace client is kept at an explicit `any` boundary so TypeScript does not recursively expand the full IDL. A guarded GitHub Actions repair run completed `npm run typecheck` before committing this change; full CI confirmation remains required.
- The contract workspace was formatted with the CI-pinned Rust 1.79.0 toolchain, and the guarded formatter run completed `cargo fmt --all -- --check` before committing; unit-test and build confirmation remain required.

## CI validation required

The GitHub workflow now installs Node, Rust, Solana CLI and Anchor CLI and runs:

```bash
npm install
npm run validate:tokenomics
anchor build
cargo test --manifest-path programs/perax-core/Cargo.toml
npm run typecheck
anchor test --provider.cluster localnet
```

Correction number 1 must not be marked complete until this workflow passes. A configured workflow is not evidence that the program compiled or that transactions succeeded.

## Security work still required

Keypairs that were committed publicly must be considered compromised. Removing them from the current tree is not sufficient. Before devnet activation:

1. Generate replacement authorities and allocation-owner keypairs in a secure environment.
2. Transfer every affected on-chain authority and balance to the replacements.
3. Purge the exposed paths from reachable Git history and contact GitHub support where necessary for cached or pull-request references.
4. Verify that no branch, tag, artifact, log or fork still exposes the old material.

The repository does not contain or request replacement private keys.

## Devnet work not performed

- No devnet program update has been performed for the hardened vault code.
- No reserve vault has been initialized by this correction.
- No PEX has been migrated into a new vault.
- The 1,000 PEX deposit and 100 PEX release trial has not been executed on devnet.
- `config/reserve-vaults.devnet.public.json` is not yet a verified activation registry.
- The public deployment record has not yet been updated with hardened vault activation evidence.

Do not update devnet or move reserve balances until key rotation is complete and all CI and local-validator checks pass.
