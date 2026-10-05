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
