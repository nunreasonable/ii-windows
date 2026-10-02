<#
.SYNOPSIS
    Removes ii-windows for the current user: stops it, deletes the Start menu shortcuts and
    autostart entry, then the installed copy under %LOCALAPPDATA%\ii-windows.
.PARAMETER RemoveConfig
    Also delete the ii config mirror (%LOCALAPPDATA%\quickshell\ii) and the user's settings
    (%LOCALAPPDATA%\illogical-impulse). Off by default, so a reinstall keeps the user's settings,
    keybinds and theme choices. Shared quickshell state (colors.json, logs, crash dumps) is left
    alone either way - it isn't specific to the ii config and nothing here created it.
.NOTES
    PowerShell 5.1 compatible, no admin rights needed.
#>
[CmdletBinding()]
param(
	[switch]$RemoveConfig
)

$ErrorActionPreference = 'Stop'

$InstallDir = Join-Path $env:LOCALAPPDATA 'ii-windows'
$QuickshellIiDir = Join-Path $env:LOCALAPPDATA 'quickshell\ii'
$IllogicalImpulseDir = Join-Path $env:LOCALAPPDATA 'illogical-impulse'
$StartMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$RunKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'

Write-Host "Uninstalling ii-windows ..."

Get-Process -Name 'qsw', 'qs' -ErrorAction SilentlyContinue |
	Where-Object { $_.Path -like "$InstallDir\*" } |
	Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

Remove-ItemProperty -Path $RunKey -Name 'illogical-impulse' -ErrorAction SilentlyContinue

foreach ($name in 'illogical-impulse.lnk', 'ii Settings.lnk') {
	$path = Join-Path $StartMenuDir $name
	if (Test-Path $path) { Remove-Item -Force $path }
}

if (Test-Path $InstallDir) {
	try {
		# This script was copied into $InstallDir by install.ps1 and may be the one currently
		# running; PowerShell -File execution doesn't hold the script file open after parsing it,
		# so this normally succeeds anyway, but fall back to a short-delayed cmd.exe cleanup
		# rather than fail the uninstall if it doesn't.
		Remove-Item -Recurse -Force $InstallDir -ErrorAction Stop
	} catch {
		Write-Warning "Could not fully remove $InstallDir right now ($($_.Exception.Message)); scheduling cleanup in the background."
		Start-Process -WindowStyle Hidden cmd.exe -ArgumentList "/c timeout /t 2 /nobreak >nul & rmdir /s /q `"$InstallDir`""
	}
}

if ($RemoveConfig) {
	Write-Host "Removing ii config and settings (-RemoveConfig) ..."
	foreach ($dir in $QuickshellIiDir, $IllogicalImpulseDir) {
		if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
	}
} else {
	Write-Host "Keeping config: $QuickshellIiDir, $IllogicalImpulseDir"
	Write-Host "Re-run with -RemoveConfig to also delete those."
}

Write-Host "Done."
