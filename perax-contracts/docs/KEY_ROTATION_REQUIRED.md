# Security Blocker: Historical Keypair Rotation Required

## Status

Private Solana keypair material was previously committed under `perax-contracts/.local/`. The directory has been removed from the current `main` tree and both repository ignore files now block every `.local/` directory.

This does not make the exposed identities safe. Every keypair ever committed under the historical path must be treated as compromised, even when the corresponding file is no longer visible in the latest revision.

## Do not proceed with vault activation

Do not update the devnet program, initialize reserve vaults or migrate PEX until all affected identities have been replaced.

At minimum, verify and rotate:

- Solana program upgrade authority.
- Pera-X program state authority.
- Safety administrator.
- Oracle bot signer.
- Every legacy allocation owner whose keypair appeared in repository history.
- Any fee-payer, deployment or operational signer stored under the historical path.

## Secure rotation procedure

1. Generate replacement keypairs on an offline or otherwise secured machine.
2. Back them up securely outside the repository and outside CI logs or artifacts.
3. Fund only the replacement public keys with the minimum devnet SOL required.
4. Transfer the Solana program upgrade authority to the replacement authority.
5. Use the Pera-X two-step authority transfer to nominate and accept the replacement state authority.
6. Update the safety administrator and oracle bot to replacement public keys.
7. For each exposed allocation owner, transfer the full PEX balance to a newly approved token account controlled by a replacement owner, or securely change the token-account authority where that is the intended policy.
8. Update the public deployment record with replacement public keys and transaction signatures only.
9. Verify the old public keys no longer control the program, state, oracle, safety role or allocation balances.
10. Only then purge the exposed paths from reachable Git history.

## History remediation

After rotation, rewrite reachable Git history to remove `perax-contracts/.local/`, force-update the approved branch, remove affected tags and stale branches, and have every collaborator discard old clones. GitHub support may also be required to remove cached views, pull-request references or other retained objects.

History rewriting is disruptive and is not a substitute for key rotation. Rotation is the security control; history cleanup reduces continued accidental exposure.

## Evidence required before this blocker is closed

- Replacement public keys recorded.
- On-chain authority-transfer signatures recorded.
- Allocation-balance transfer or authority-change signatures recorded.
- Verification that old keys have no remaining authority.
- Repository history scan showing no reachable keypair files.
- Confirmation that no workflow artifact or log contains private key material.

Never commit replacement private keys, seed phrases, local signer maps or populated migration configuration files.
