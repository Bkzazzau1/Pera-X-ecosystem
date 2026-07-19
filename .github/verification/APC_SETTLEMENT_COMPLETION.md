# APC settlement completion verification

This temporary verification change runs the exact current `main` source through:

- Rust formatting, unit tests, and compilation.
- Contract tokenomics, settlement source, hardened market, and TypeScript validators.
- Market-engine typecheck, tests, build, and production dependency audit.
- Meteora runtime reconstruction, typecheck, tests, build, and production dependency audit.

The workflow uploads the reconstructed reviewed Meteora TypeScript source and deterministic lockfile so they can replace the temporary generated-artifact mechanism on `main`.

No deployment, initialization, reserve movement, authority change, or numerical-policy approval is performed by this verification.
