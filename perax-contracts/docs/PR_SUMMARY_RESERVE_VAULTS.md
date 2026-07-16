# Development Summary: Program-Controlled Reserve Vaults

This ongoing development change hardens reserve custody before devnet activation.

## Source changes

- Separate configuration PDA, authority PDA and PEX token account for each approved allocation.
- Exact configured migration source owner and legacy PEX token account.
- Separate `authorized_deposited`, `unsolicited_balance` and `total_released` accounting.
- Atomic market validation, approved-destination enforcement, PDA-signed transfer, accounting and permanent replay record.
- PDA-owned and cross-vault destinations rejected.
- Liquidity and vesting allocations excluded from ordinary market releases.
- Expanded local-validator tests covering all 13 allocation IDs and rollback behavior.
- Dry-run-first creation, migration and verification scripts.
- `.local/` secret directories ignored and removed from the current `main` tree.

## Status

No production release is represented by these changes. No devnet program update, reserve migration, PEX minting or vault trial has been performed. Publicly exposed historical keypairs still require secure rotation and history remediation. The GitHub CI build and transaction tests must pass before any on-chain activation.
