# PR Summary: Program-Controlled Reserve Vaults

This development change moves reserve custody from ordinary wallet-controlled token accounts toward separate Pera-X program-controlled vaults.

## Main changes

- Separate configuration PDA, authority PDA, and PEX token account for every approved allocation.
- On-chain allocation ID, vault class, and maximum-cap enforcement.
- Signed migration deposits from existing allocation owners.
- Atomic market validation, PDA-signed token transfer, accounting update, event emission, and permanent replay-protected release record.
- Legacy approval-only release disabled.
- Liquidity and vesting allocations excluded from ordinary market releases.
- Per-vault pause and safe reconciliation instructions.
- Local-validator transaction tests for the 1,000 PEX / 100 PEX trial and required failure cases.
- Dry-run-first devnet creation, migration, and verification scripts.

## Safety status

No program deployment, minting, or PEX transfer was performed. Rust/Anchor compilation and local-validator execution remain required in a prepared toolchain environment before deployment.
