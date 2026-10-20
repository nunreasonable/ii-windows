param(
    [Parameter(Mandatory = $true)][string]$PackageDir,
    [Parameter(Mandatory = $true)][string]$Arch
)

$ErrorActionPreference = 'Stop'
$machineByArch = @{ x64 = 0x8664; arm64 = 0xAA64 }
if (-not $machineByArch.ContainsKey($Arch)) { "unknown arch $Arch"; exit 1 }
$wantMachine = $machineByArch[$Arch]
$fails = 0

function Get-PEMachine([string]$Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
    [BitConverter]::ToUInt16($bytes, $peOffset + 4)
}

$root = (Resolve-Path $PackageDir).Path
$files = Get-ChildItem -LiteralPath $root -Recurse -File | Where-Object { $_.Extension -in '.exe', '.dll', '.node' }

$wrongArch = @()
foreach ($f in $files) {
    $m = Get-PEMachine $f.FullName
    if ($m -ne $wantMachine) { $wrongArch += ("{0} (0x{1:X4})" -f $f.FullName.Substring($root.Length + 1), $m) }
}
if ($wrongArch.Count) {
    $fails++
    "FAIL  package has non-$Arch PE files:"
    $wrongArch | ForEach-Object { "      $_" }
} else {
    "PASS  every .exe/.dll/.node under $PackageDir is $Arch"
}

$packageNames = New-Object Collections.Generic.HashSet[string]
Get-ChildItem -LiteralPath $root -Recurse -File | ForEach-Object { [void]$packageNames.Add($_.Name.ToLowerInvariant()) }

$system32 = Join-Path $env:SystemRoot 'System32'
$downlevel = Join-Path $system32 'downlevel'
$missing = @{}
foreach ($f in $files) {
    $out = & dumpbin /nologo /dependents $f.FullName 2>&1
    $deps = New-Object Collections.Generic.List[string]
    $inList = $false
    foreach ($line in $out) {
        $t = "$line".Trim()
        if ($t -match '^Image has the following dependencies') { $inList = $true; continue }
        if (-not $inList) { continue }
        if ($t -eq '') { if ($deps.Count -gt 0) { $inList = $false }; continue }
        if ($t -match '^Summary') { $inList = $false; continue }
        if ($t -match '(?i)\.dll$') { $deps.Add($t) }
    }
    foreach ($dep in $deps) {
        $depLower = $dep.ToLowerInvariant()
        if ($depLower -match '^(api-ms-win-|ext-ms-win-)') { continue }
        if ($packageNames.Contains($depLower)) { continue }
        if (Test-Path (Join-Path $system32 $dep)) { continue }
        if (Test-Path (Join-Path $downlevel $dep)) { continue }
        if (-not $missing.ContainsKey($depLower)) { $missing[$depLower] = New-Object Collections.Generic.List[string] }
        $missing[$depLower].Add($f.FullName.Substring($root.Length + 1))
    }
}
if ($missing.Count) {
    $fails++
    "FAIL  binaries import DLLs that are neither in the package nor in System32:"
    foreach ($dep in $missing.Keys) { "      $dep  (imported by: $($missing[$dep] -join ', '))" }
} else {
    "PASS  every imported DLL resolves inside the package or System32"
}

if ($fails) { exit 1 }
exit 0
