param(
    [Parameter(Mandatory = $true)][string]$Package,
    [Parameter(Mandatory = $true)][string]$Setup,
    [Parameter(Mandatory = $true)][string]$CompileCheck,
    [Parameter(Mandatory = $true)][string]$Work,
    [string]$Output = ''
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

New-Item -ItemType Directory -Force $Work | Out-Null
$dev = Join-Path $Work 'ii-windows'
$stage = Join-Path $Work 'smoketest'
Remove-Item -Recurse -Force $dev, $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $dev, $stage, (Join-Path $Work 'logs') | Out-Null

Add-Type -AssemblyName System.IO.Compression.FileSystem
[IO.Compression.ZipFile]::ExtractToDirectory((Resolve-Path $Package).Path, $dev)
foreach ($f in 'smoke.ps1', 'fonttest.qml', 'inherit.qml') { Copy-Item -Force (Join-Path $here $f) $stage }
Copy-Item -Force $CompileCheck (Join-Path $stage 'compilecheck.qml')
Copy-Item -Force $Setup (Join-Path $stage 'ii-windows-setup.exe')

$colors = Join-Path $env:LOCALAPPDATA 'quickshell\State\user\generated\colors.json'
if (-not (Test-Path $colors)) {
    New-Item -ItemType Directory -Force (Split-Path $colors) | Out-Null
    Copy-Item (Join-Path $dev 'config\ii\defaults\windows\colors.json') $colors
}

$arch = $env:PROCESSOR_ARCHITECTURE
$machine = (Get-CimInstance Win32_ComputerSystem).SystemType
$bytes = [IO.File]::ReadAllBytes((Join-Path $dev 'qsw.exe'))
$peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
$exeMachine = [BitConverter]::ToUInt16($bytes, $peOffset + 4)
$exeArch = switch ($exeMachine) { 0x8664 { 'x64' } 0xAA64 { 'arm64' } default { ('0x{0:X4}' -f $exeMachine) } }

$env:IIW_ROOT = $Work
$ErrorActionPreference = 'Continue'
$out = & (Join-Path $stage 'smoke.ps1') -Root $stage 2>&1 | ForEach-Object { "$_" }
$lines = @("INFO  machine  $machine (PowerShell process $arch), qsw.exe $exeArch") + $out
$lines | ForEach-Object { $_ }
if ($Output) { [IO.File]::WriteAllLines($Output, $lines) }

Get-ChildItem (Join-Path $env:LOCALAPPDATA 'quickshell\run\by-id') -Recurse -Filter log.log -ErrorAction SilentlyContinue |
    ForEach-Object { Copy-Item $_.FullName (Join-Path $Work ("logs\" + $_.Directory.Name + '.log')) -ErrorAction SilentlyContinue }

$runnerLimits = @('region screenshot captured (async)', 'install folder is free once ii stops')
$failed = @($out | Where-Object { $_ -like 'FAIL  *' } | ForEach-Object { ($_ -replace '^FAIL  ', '') -replace '  \(.*$', '' })
$known = @($failed | Where-Object { $runnerLimits -contains $_ })
$real = @($failed | Where-Object { $runnerLimits -notcontains $_ })
if ($known.Count) { "NOTE  failing because of the CI runner, not ii (elevated session, virtual display): " + ($known -join '; ') }
if (-not ($out -match '^RESULT: ')) { "no smoke result"; exit 1 }
if ($real.Count) { exit 1 }
exit 0
