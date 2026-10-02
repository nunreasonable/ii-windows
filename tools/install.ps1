<#
.SYNOPSIS
    Installs ii-windows (the Quickshell runtime + the illogical-impulse config) for the current
    user. No admin rights needed; everything lands under %LOCALAPPDATA%.
.PARAMETER Autostart
    Also register "qsw.exe -c ii" to start with Windows, via HKCU\...\CurrentVersion\Run.
    Off by default - run install.ps1 again with -Autostart to turn it on, or without it to turn
    it back off.
.NOTES
    PowerShell 5.1 compatible (the version that ships with Windows 11), run from the folder this
    script was extracted into alongside qs.exe/qsw.exe/config/ii/...
#>
[CmdletBinding()]
param(
	[switch]$Autostart
)

$ErrorActionPreference = 'Stop'

function Invoke-Mirror {
	# Wraps robocopy /MIR: exit codes 0-7 are success (0 means "nothing to do"), >=8 is a real
	# failure - robocopy's normal behavior trips $ErrorActionPreference='Stop' wrapping if you
	# just let a nonzero $LASTEXITCODE flow through, so it's checked explicitly instead.
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

# A running instance locks qs.exe/qsw.exe and the Qt DLLs next to them; stop it before copying
# over it. Only processes actually running from this install (not some other quickshell config).
Get-Process -Name 'qsw', 'qs' -ErrorAction SilentlyContinue |
	Where-Object { $_.Path -like "$InstallDir\*" } |
	Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# Runtime: qs.exe, qsw.exe, Qt DLLs/plugins/qml, fonts, config/ii (the template, mirrored into
# its live location below), VirtualDesktopAccessor.dll, matugen.exe, qt.conf - everything the zip
# ships except this installer and the uninstaller, which get their own copy right after so a
# re-run never has to overwrite its own currently-executing file via the mirror.
Invoke-Mirror -Source $ScriptRoot -Destination $InstallDir -ExcludeFiles @('install.ps1', 'uninstall.ps1')

foreach ($name in 'install.ps1', 'uninstall.ps1') {
	$src = Join-Path $ScriptRoot $name
	$dst = Join-Path $InstallDir $name
	if ((Test-Path $src) -and ($src -ne $dst)) {
		Copy-Item -Force $src $dst
	}
}

# ii config: mirror the shipped copy into %LOCALAPPDATA%\quickshell\ii, which is where
# "qsw.exe -c ii" looks for it (same path tools/vm.sh's `ii start` job uses on the test VM).
# The user's actual settings (%LOCALAPPDATA%\illogical-impulse\config.json) live in a completely
# separate directory and are never touched by this mirror; the /XF below is just a defensive
# backstop in case that ever changes.
$IiSource = Join-Path $InstallDir 'config\ii'
if (Test-Path $IiSource) {
	Invoke-Mirror -Source $IiSource -Destination $QuickshellIiDir -ExcludeFiles @('config.json')
} else {
	Write-Warning "No config\ii in this package; skipping the ii config mirror."
}

# Seed the Material palette only if nothing is there yet. MaterialThemeLoader falls back to
# Appearance's built-in palette quietly when colors.json is missing, but a first run should see
# *some* theming rather than plain defaults, and a reinstall shouldn't clobber a palette matugen
# (or a previous run) already generated.
if (-not (Test-Path $ColorsPath)) {
	$defaultColors = Join-Path $QuickshellIiDir 'defaults\windows\colors.json'
	if (Test-Path $defaultColors) {
		New-Item -Force -ItemType Directory (Split-Path $ColorsPath) | Out-Null
		Copy-Item -Force $defaultColors $ColorsPath
	}
}

# Start menu shortcuts (per-user Start Menu, no admin needed).
New-Item -Force -ItemType Directory $StartMenuDir | Out-Null
$shell = New-Object -ComObject WScript.Shell
$qsw = Join-Path $InstallDir 'qsw.exe'

$mainShortcut = $shell.CreateShortcut((Join-Path $StartMenuDir 'illogical-impulse.lnk'))
$mainShortcut.TargetPath = $qsw
$mainShortcut.Arguments = '-c ii'
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

# Autostart is opt-in only: re-running install.ps1 without -Autostart turns it back off, so
# there's one obvious way to toggle it rather than a separate switch to remove it.
if ($Autostart) {
	$runCommand = '"' + $qsw + '" -c ii'
	New-ItemProperty -Path $RunKey -Name 'illogical-impulse' -Value $runCommand -PropertyType String -Force | Out-Null
	Write-Host "Autostart enabled (HKCU Run: illogical-impulse)."
} else {
	Remove-ItemProperty -Path $RunKey -Name 'illogical-impulse' -ErrorAction SilentlyContinue
}

Write-Host "Installed. Launch 'illogical-impulse' from the Start menu, or run:"
Write-Host "  `"$qsw`" -c ii"
