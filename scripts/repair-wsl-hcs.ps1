$ErrorActionPreference = "Stop"

function Assert-Elevated {
    $principal = [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Run this script from an elevated PowerShell window."
    }
}

Assert-Elevated

Write-Host "Enabling WSL and Virtual Machine Platform..."
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

Write-Host "Making sure the Windows hypervisor starts at boot..."
bcdedit.exe /set hypervisorlaunchtype auto

Write-Host "Updating WSL kernel/package..."
wsl --update
wsl --set-default-version 2
wsl --shutdown

Write-Host "Restarting VM services..."
Restart-Service vmcompute -Force -ErrorAction SilentlyContinue
Restart-Service hns -Force -ErrorAction SilentlyContinue
Start-Service LxssManager -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Repair commands completed. Reboot Windows now, then run:"
Write-Host "  wsl --install -d Ubuntu-24.04"

