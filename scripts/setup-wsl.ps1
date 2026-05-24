$ErrorActionPreference = "Stop"

function Assert-Elevated {
    $principal = [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Run this script from an elevated PowerShell window."
    }
}

Assert-Elevated

Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux -NoRestart | Out-Null
Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform -NoRestart | Out-Null

wsl --update
wsl --set-default-version 2

Restart-Service vmcompute -ErrorAction SilentlyContinue
Restart-Service hns -ErrorAction SilentlyContinue
Start-Service LxssManager -ErrorAction SilentlyContinue
wsl --shutdown

wsl --install -d Ubuntu-24.04

Write-Host ""
Write-Host "After Ubuntu opens, create your Linux user, then run:"
Write-Host "  cd /mnt/c/PROJECTS/'smartcontract PEX'/perax-ecosystem"
Write-Host "  bash scripts/bootstrap-ubuntu.sh"
