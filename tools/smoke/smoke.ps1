param([string]$Root = 'C:\ii-windows\smoketest')

$ErrorActionPreference = 'Continue'
$inst = Join-Path $env:LOCALAPPDATA 'ii-windows'
$dev = Join-Path $env:IIW_ROOT 'ii-windows'
$configDir = Join-Path $env:LOCALAPPDATA 'quickshell\ii'
$runRoot = Join-Path $env:LOCALAPPDATA 'quickshell\run\by-id'
$results = New-Object Collections.Generic.List[string]
$script:fails = 0
$errorPattern = 'TypeError|ReferenceError|is not defined|Cannot read property|Failed to create FreeType'

function Check([string]$name, [bool]$ok, [string]$detail = '') {
    if (-not $ok) { $script:fails++ }
    $mark = if ($ok) { 'PASS' } else { 'FAIL' }
    $results.Add("$mark  $name" + $(if ($detail) { "  ($detail)" } else { '' }))
}
function Note([string]$name, [string]$detail) { $results.Add("INFO  $name  $detail") }

function IpcT {
    $p = Start-Process (Join-Path $inst 'qs.exe') -ArgumentList (@('-c', 'ii', 'ipc', 'call') + $args) -WindowStyle Hidden -PassThru
    if (-not $p.WaitForExit(5000)) { try { $p.Kill() } catch {} }
}

Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class SmokeKeys { [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra); }'
function PressEscape { [SmokeKeys]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 50; [SmokeKeys]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero) }

function ShellLog {
    Get-ChildItem $runRoot -Directory -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | ForEach-Object {
        $log = Join-Path $_.FullName 'log.log'
        if ((Test-Path $log) -and ((Get-Content $log -TotalCount 1) -match 'quickshell/ii/shell\.qml')) { return $log }
    } | Select-Object -First 1
}
function LogErrors([string]$log) { if (-not $log) { return @() }; @(Get-Content $log | Select-String -Pattern $errorPattern | ForEach-Object { $_.Line }) }
function NewestLogOf([datetime]$since) {
    Get-ChildItem $runRoot -Directory -ErrorAction SilentlyContinue | Where-Object { $_.CreationTime -ge $since } |
        Sort-Object CreationTime -Descending | ForEach-Object { Join-Path $_.FullName 'log.log' } | Select-Object -First 1
}
function GuiSwitches { (Get-CimInstance Win32_PerfRawData_PerfProc_Thread -Filter "Name='qsw/0'" -ErrorAction SilentlyContinue | Select-Object -First 1).ContextSwitchesPersec }
function ShellProcess { Get-Process qsw -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Join-Path $inst 'qsw.exe') } | Sort-Object StartTime | Select-Object -First 1 }
function StartShell {
    $p = Start-Process -PassThru -FilePath (Join-Path $inst 'qsw.exe') -ArgumentList '-n', '-c', 'ii' -WorkingDirectory $inst
    Start-Sleep 25
    $p
}
function TryRename([string]$dir) {
    $leaf = Split-Path $dir -Leaf
    try {
        Rename-Item -LiteralPath $dir "$leaf.smoke" -ErrorAction Stop
        Rename-Item -LiteralPath "$dir.smoke" $leaf -ErrorAction Stop
        return ''
    } catch { return $_.Exception.Message }
}
function RunHarness([string]$qml, [hashtable]$envs, [int]$seconds) {
    foreach ($k in $envs.Keys) { Set-Item "Env:$k" $envs[$k] }
    $p = Start-Process -PassThru -FilePath (Join-Path $inst 'qsw.exe') -ArgumentList '-n', '-p', $qml -WorkingDirectory $inst
    Start-Sleep $seconds
    foreach ($k in $envs.Keys) { Remove-Item "Env:$k" -ErrorAction SilentlyContinue }
    $p
}

Get-Process qsw, qs, ii-windows-setup, PING -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
robocopy $dev $inst /E /XD config /XF ii-windows-setup.exe install-manifest.json VERSION /NFL /NDL /NJH /NJS /NP | Out-Null
robocopy (Join-Path $dev 'config\ii') (Join-Path $inst 'config\ii') /MIR /NFL /NDL /NJH /NJS /NP | Out-Null
robocopy (Join-Path $inst 'config\ii') $configDir /MIR /XF config.json /NFL /NDL /NJH /NJS /NP | Out-Null
$rev = Select-String -LiteralPath (Join-Path $inst 'qsw.exe') -Pattern 'revision [0-9a-f]{7}' -List | Select-Object -First 1
Note 'build' ($(if ($rev) { $rev.Matches[0].Value } else { 'unknown revision' }) + ', ' + [Environment]::OSVersion.VersionString)

$compile = Join-Path $Root 'compilecheck.qml'
if (Test-Path $compile) {
    $target = Join-Path $configDir 'compilecheck.qml'
    Copy-Item -Force $compile $target
    $since = Get-Date
    $c = Start-Process -PassThru -FilePath (Join-Path $inst 'qsw.exe') -ArgumentList '-n', '-p', $target -WorkingDirectory $inst
    $done = $null
    for ($i = 0; $i -lt 120 -and -not $done; $i++) {
        Start-Sleep 1
        $clog = NewestLogOf $since
        if ($clog) { $done = Get-Content $clog | Select-String -SimpleMatch 'COMPILE-DONE' | Select-Object -First 1 }
    }
    $failed = if ($clog) { @(Get-Content $clog | Select-String -SimpleMatch 'COMPILE-FAIL' | ForEach-Object { $_.Line }) } else { @() }
    Check 'every QML file compiles' (($null -ne $done) -and $failed.Count -eq 0) $(if ($done) { $done.Line -replace '^.*COMPILE-DONE ', '' } else { 'no result' })
    $failed | Select-Object -First 5 | ForEach-Object { $results.Add('      ' + ($_ -replace '^.*COMPILE-FAIL ', '')) }
    if (-not $c.HasExited) { Stop-Process -Id $c.Id -Force -ErrorAction SilentlyContinue }
    Start-Sleep 1
    Remove-Item -Force $target -ErrorAction SilentlyContinue
    if ($clog) { Remove-Item -Recurse -Force (Split-Path $clog) -ErrorAction SilentlyContinue }
}

$shell = StartShell
Check 'shell starts' (-not $shell.HasExited)
$log = ShellLog

$panels = @(
    @('sidebarLeft', 'open', 'sidebarLeft', 'close'),
    @('sidebarRight', 'open', 'sidebarRight', 'close'),
    @('search', 'workspacesToggle', 'search', 'close'),
    @('search', 'toggle', 'search', 'close'),
    @('cheatsheet', 'open', 'cheatsheet', 'close'),
    @('mediaControls', 'open', 'mediaControls', 'close'),
    @('wallpaperSelector', 'toggle', 'wallpaperSelector', 'toggle'),
    @('session', 'open', 'session', 'close'),
    @('settings', 'open', 'settings', 'close'),
    @('overlay', 'toggle', 'overlay', 'toggle')
)
foreach ($panel in $panels) {
    IpcT $panel[0] $panel[1]; Start-Sleep 2
    IpcT $panel[2] $panel[3]; Start-Sleep 1
}
$shotDir = Join-Path $env:TEMP 'quickshell\media\screenshot'
IpcT region screenshot; Start-Sleep 4
$shot = Get-ChildItem $shotDir -Filter 'image-*' -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt $shell.StartTime -and $_.Name -notlike '*.tmp-*' } | Select-Object -First 1
Check 'region screenshot captured (async)' ($null -ne $shot)
PressEscape; Start-Sleep 2
$left = @(Get-ChildItem $shotDir -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt $shell.StartTime -and ($_.Name -like 'image-*' -or $_.Name -like '*.tmp-*') })
Check 'region screenshot cleaned up after closing' ($left.Count -eq 0) "$($left.Count) files"
Start-Sleep 10
Check 'shell alive after opening every panel' (-not $shell.HasExited)
$errors = LogErrors $log
Check 'no QML/font errors in the log' ($errors.Count -eq 0) "$($errors.Count) lines"
$errors | Select-Object -Unique -First 5 | ForEach-Object { $results.Add('      ' + $_.Substring([Math]::Min(24, $_.Length))) }

Start-Sleep 20
$p = ShellProcess
$cpu0 = $p.TotalProcessorTime.TotalMilliseconds
$sw0 = GuiSwitches
Start-Sleep 30
$p.Refresh()
$cpu = [int]($p.TotalProcessorTime.TotalMilliseconds - $cpu0)
$sw = [Math]::Round(((GuiSwitches) - $sw0) / 30.0, 1)
Note 'idle' "cpu $cpu ms / 30 s, gui thread $sw switches/s, private $([int]($p.PrivateMemorySize64 / 1MB)) MB"
Check 'idle cpu under 3 s per 30 s' ($cpu -lt 3000) "$cpu ms"
Check 'idle gui thread under 60 switches/s' ($sw -lt 60) "$sw/s"

$since = Get-Date
$h = RunHarness (Join-Path $Root 'fonttest.qml') @{ IIW_FONTTEST_FAMILY = 'Bahnschrift'; IIW_FONTTEST_COUNT = '600' } 14
$hlog = NewestLogOf $since
Stop-Process -Id $h.Id -Force -ErrorAction SilentlyContinue
$fontFails = if ($hlog) { @(Get-Content $hlog | Select-String -SimpleMatch 'Failed to create FreeType').Count } else { -1 }
Check 'font stress (600 variations of a system font)' ($fontFails -eq 0) "$fontFails failures"
if ($hlog) { Remove-Item -Recurse -Force (Split-Path $hlog) -ErrorAction SilentlyContinue }

$fonts = Join-Path $Root 'inherit\fonts'
New-Item -ItemType Directory -Force $fonts | Out-Null
Copy-Item -Force (Join-Path $inst 'fonts\SpaceGrotesk-Regular.otf') (Join-Path $fonts 'test.ttf')
$h = RunHarness (Join-Path $Root 'inherit.qml') @{ IIW_INHERIT_CMD = '["ping","-n","30","127.0.0.1"]' } 7
Stop-Process -Id $h.Id -Force -ErrorAction SilentlyContinue; Start-Sleep 2
$err = TryRename $fonts
Check 'children of ii do not keep its files open' ($err -eq '') $err
Get-Process PING -ErrorAction SilentlyContinue | Stop-Process -Force

$setup = Join-Path $Root 'ii-windows-setup.exe'
if (Test-Path $setup) {
    $cmd = ConvertTo-Json -Compress @($setup, '--update', '--no-self-update')
    $h = RunHarness (Join-Path $Root 'inherit.qml') @{ IIW_INHERIT_CMD = $cmd } 9
    Stop-Process -Id $h.Id -Force -ErrorAction SilentlyContinue; Start-Sleep 2
    $err = TryRename $fonts
    $open = @(Get-Process ii-windows-setup -ErrorAction SilentlyContinue).Count
    Check 'setup opened for an update holds nothing of ii' (($err -eq '') -and ($open -gt 0)) $(if ($open -eq 0) { 'setup not running' } else { $err })
    Get-Process ii-windows-setup -ErrorAction SilentlyContinue | Stop-Process -Force
}

$shell = ShellProcess
if ($shell) { Stop-Process -Id $shell.Id -Force; Start-Sleep 2 }
$err = TryRename (Join-Path $inst 'fonts')
Check 'install folder is free once ii stops' ($err -eq '') $err
$shell = StartShell
Check 'shell restarts' (-not $shell.HasExited)

$report = Join-Path $configDir 'scripts\report\collect-report.ps1'
$out = & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $report -Version smoke -Screens smoke -Style smoke -Family smoke 2>&1
$zip = ($out | Select-Object -Last 1) -as [string]
$zipOk = $zip -and $zip.Trim().EndsWith('.zip') -and (Test-Path $zip.Trim())
$entries = @()
if ($zipOk) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip.Trim())
    $entries = @($archive.Entries | ForEach-Object Name)
    $latestEntry = $archive.Entries | Where-Object Name -eq 'log-latest.txt' | Select-Object -First 1
    $latestHead = ''
    if ($latestEntry) {
        $reader = New-Object IO.StreamReader($latestEntry.Open())
        $latestHead = $reader.ReadLine()
        $reader.Dispose()
    }
    $archive.Dispose()
    Remove-Item -Force $zip.Trim()
}
Check 'problem report zip' ($zipOk -and ($entries -contains 'system-info.txt') -and ($entries -contains 'log-latest.txt')) ($entries -join ', ')
Check 'problem report has the running shell log' ($latestHead -match 'Launching config: .*shell\.qml') $latestHead
Check 'problem report leaves crash dumps out unless asked' (@($entries | Where-Object { $_ -like '*.dmp' }).Count -eq 0)

$clip = Join-Path $env:TEMP 'iiw-smoke-clip.ps1'
[IO.File]::WriteAllText($clip, "Add-Type -AssemblyName System.Windows.Forms; [Windows.Forms.Clipboard]::SetText(('smoke ' * 20000)); Start-Sleep -Milliseconds 300; [Windows.Forms.Clipboard]::SetText('smoke small')")
$before = Get-Date
Start-Process powershell.exe -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $clip -Wait
Start-Sleep 3
$spilled = @(Get-ChildItem (Join-Path $env:LOCALAPPDATA 'cache\quickshell\clipboard') -Recurse -Filter '*.txt' -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -ge $before })
Check 'large clipboard text kept on disk' ($spilled.Count -eq 1) "$($spilled.Count) files"
[IO.File]::WriteAllText($clip, 'Add-Type -AssemblyName System.Windows.Forms; [Windows.Forms.Clipboard]::Clear()')
Start-Process powershell.exe -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $clip -Wait
Remove-Item -Force $clip -ErrorAction SilentlyContinue

$log = ShellLog
$errors = LogErrors $log
Check 'no errors after the second start' ($errors.Count -eq 0) "$($errors.Count) lines"
Check 'shell alive at the end' (-not $shell.HasExited)
Remove-Item -Recurse -Force (Join-Path $Root 'inherit') -ErrorAction SilentlyContinue

$results
"RESULT: $(if ($script:fails -eq 0) { 'all passed' } else { "$($script:fails) failed" })"
