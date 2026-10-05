[CmdletBinding()]
param(
	[switch]$Autostart
)

$ErrorActionPreference = 'Stop'

function Invoke-Mirror {
	param(
		[Parameter(Mandatory)][string]$Source,
		[Parameter(Mandatory)][string]$Destination,
		[string[]]$ExcludeFiles = @()
	)

	New-Item -Force -ItemType Directory $Destination | Out-Null
	$roboArgs = @($Source, $Destination, '/MIR', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/R:2', '/W:1')
	if ($ExcludeFiles.Count -gt 0) { $roboArgs += '/XF'; $roboArgs += $ExcludeFiles }

	robocopy.exe @roboArgs | Out-Null
	if ($LASTEXITCODE -ge 8) {
		throw "robocopy failed (exit $LASTEXITCODE) mirroring `"$Source`" to `"$Destination`""
	}
}

$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$InstallDir = Join-Path $env:LOCALAPPDATA 'ii-windows'
$QuickshellIiDir = Join-Path $env:LOCALAPPDATA 'quickshell\ii'
$ColorsPath = Join-Path $env:LOCALAPPDATA 'quickshell\State\user\generated\colors.json'
$StartMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$RunKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'

Write-Host "Installing ii-windows to $InstallDir ..."

Get-Process -Name 'qsw', 'qs' -ErrorAction SilentlyContinue |
	Where-Object { $_.Path -like "$InstallDir\*" } |
	Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

Invoke-Mirror -Source $ScriptRoot -Destination $InstallDir -ExcludeFiles @('install.ps1', 'uninstall.ps1')

foreach ($name in 'install.ps1', 'uninstall.ps1') {
	$src = Join-Path $ScriptRoot $name
	$dst = Join-Path $InstallDir $name
	if ((Test-Path $src) -and ($src -ne $dst)) {
		Copy-Item -Force $src $dst
	}
}

$IiSource = Join-Path $InstallDir 'config\ii'
if (Test-Path $IiSource) {
	Invoke-Mirror -Source $IiSource -Destination $QuickshellIiDir -ExcludeFiles @('config.json')
} else {
	Write-Warning "No config\ii in this package; skipping the ii config mirror."
}

if (-not (Test-Path $ColorsPath)) {
	$defaultColors = Join-Path $QuickshellIiDir 'defaults\windows\colors.json'
	if (Test-Path $defaultColors) {
		New-Item -Force -ItemType Directory (Split-Path $ColorsPath) | Out-Null
		Copy-Item -Force $defaultColors $ColorsPath
	}
}

New-Item -Force -ItemType Directory $StartMenuDir | Out-Null
$shell = New-Object -ComObject WScript.Shell
$qsw = Join-Path $InstallDir 'qsw.exe'

$mainShortcut = $shell.CreateShortcut((Join-Path $StartMenuDir 'illogical-impulse.lnk'))
$mainShortcut.TargetPath = $qsw
$mainShortcut.Arguments = '-n -c ii'
$mainShortcut.WorkingDirectory = $InstallDir
$mainShortcut.IconLocation = $qsw
$mainShortcut.Description = 'illogical-impulse shell'
$mainShortcut.Save()

$settingsShortcut = $shell.CreateShortcut((Join-Path $StartMenuDir 'ii Settings.lnk'))
$settingsShortcut.TargetPath = $qsw
$settingsShortcut.Arguments = '-p "' + (Join-Path $QuickshellIiDir 'settings.qml') + '"'
$settingsShortcut.WorkingDirectory = $InstallDir
$settingsShortcut.IconLocation = $qsw
$settingsShortcut.Description = 'illogical-impulse settings'
$settingsShortcut.Save()

if ($Autostart) {
	$runCommand = '"' + $qsw + '" -n -c ii'
	New-ItemProperty -Path $RunKey -Name 'illogical-impulse' -Value $runCommand -PropertyType String -Force | Out-Null
	Write-Host "Autostart enabled (HKCU Run: illogical-impulse)."
} else {
	Remove-ItemProperty -Path $RunKey -Name 'illogical-impulse' -ErrorAction SilentlyContinue
}

Write-Host "Installed. Launch 'illogical-impulse' from the Start menu, or run:"
Write-Host "  `"$qsw`" -c ii"
