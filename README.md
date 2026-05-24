# Perax Ecosystem

This workspace keeps the Web2 gateway and Web3 contracts in separate project roots.

```text
perax-ecosystem/
├── perax-gateway/
└── perax-contracts/
```

`perax-gateway` is reserved for the Axum backend. `perax-contracts` is an Anchor workspace for Solana programs.

## WSL Rust Workstation

Use WSL2 Ubuntu as the standard development environment for Rust, Anchor, Solana, Redis, and backend services.

From an elevated PowerShell window:

```powershell
.\scripts\setup-wsl.ps1
```

If Ubuntu fails during VM creation with an HCS error, run the repair script from an elevated PowerShell window and reboot:

```powershell
.\scripts\repair-wsl-hcs.ps1
```

After Ubuntu opens and your Linux user is created:

```bash
cd /mnt/c/PROJECTS/"smartcontract PEX"/perax-ecosystem
bash scripts/bootstrap-ubuntu.sh
```

Optional: copy `.wslconfig.example` to `%UserProfile%\.wslconfig`, then run `wsl --shutdown` to apply memory/CPU limits.
