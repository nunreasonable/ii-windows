# VM job: windowless test of the native Quickshell.Services.Notifications.
#   tools/vm.sh job 150 < tools/notif-test.ps1
# Needs a qs.exe with the module plus tools/testconfigs/notifications next to it (tools/deploy-ii.sh
# stages both into dist/ii-windows) and the listener probe (tools/notif-probe, pushed as
# notif-probe). Runs qs.exe (console, no windows) on the test config, shows three test toasts
# (two as Windows PowerShell, one as Calculator) and removes one from the Windows side, then
# prints the qs log. Each toast shows a Windows banner for a few seconds; if ii is running it
# mirrors them too.
#
# Expected: "access Allowed", "mirror active true", the notifySend notifications with parsed
# urgency/app name/actions/hints, "actionInvoked", mirrored toasts with appIcon set and an
# "icon ... ready" line without a "Could not load icon" warning, "closed N Dismissed" for the
# "ii-windows probe" toasts (and they are gone from "toasts after"), "closed N CloseRequested" for
# "keep B", "qs exit code: 0".
$dir = if ($env:NOTIF_QS_DIR) { $env:NOTIF_QS_DIR } else { "$env:IIW_ROOT\ii-windows" }
$cfg = if ($env:NOTIF_TEST_CONFIG) { $env:NOTIF_TEST_CONFIG } else { "$dir\testconfigs\notifications" }
$probe = "$env:IIW_ROOT\notif-probe\notif-probe.exe"
$out = "$env:IIW_ROOT\logs\notif-test.out"; $err = "$env:IIW_ROOT\logs\notif-test.err"

"--- listener"
& $probe status 2>&1
& $probe events 1 2>&1
"--- toasts before"
& $probe list 2>&1 | Select-String '^- id'

# A PNG app icon (what mirrored toasts get from the logo cache) for the "PNG icon" notification.
Add-Type -AssemblyName System.Drawing
$png = "$env:IIW_ROOT\logs\notif-test-icon.png"
$bmp = New-Object System.Drawing.Bitmap 32, 32
[System.Drawing.Graphics]::FromImage($bmp).Clear([System.Drawing.Color]::Orange)
$bmp.Save($png, [System.Drawing.Imaging.ImageFormat]::Png)
$env:QS_NOTIF_TEST_PNG = $png -replace '\\', '/'
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\quickshell\cache\notification-icons" -ErrorAction SilentlyContinue

$env:QS_NOTIF_TEST_SECONDS = "26"
$qs = Start-Process -PassThru -WindowStyle Hidden -FilePath "$dir\qs.exe" -WorkingDirectory $dir `
	-ArgumentList '-p', $cfg, '--no-color', '--log-rules', 'quickshell.windows.notifications*.debug=true' `
	-RedirectStandardOutput $out -RedirectStandardError $err
$null = $qs.Handle # keep the handle so ExitCode is readable after exit
"started qs $($qs.Id)"

[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
function Show-Toast($aumid, $title, $body) {
	try {
		$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
		$xml.LoadXml("<toast><visual><binding template=`"ToastGeneric`"><text>$title</text><text>$body</text></binding></visual></toast>")
		[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($aumid).Show([Windows.UI.Notifications.ToastNotification]::new($xml))
		"$(Get-Date -Format HH:mm:ss.fff) toast shown: $title"
	} catch { "toast '$title' failed: $($_.Exception.Message)" }
}
$ps = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'

Start-Sleep 7
Show-Toast $ps 'ii-windows probe A' 'dismissed by the test config, line &lt;2&gt;'
Start-Sleep 2
Show-Toast 'Microsoft.WindowsCalculator_8wekyb3d8bbwe!App' 'ii-windows probe C' 'packaged sender'
Start-Sleep 4
Show-Toast $ps 'ii-windows keep B' 'removed from the Windows side by the probe'
Start-Sleep 4

"--- toasts mid-test (A and C should be gone already)"
$list = & $probe list 2>&1
$list | Select-String '^- id|text'
$id = ($list | Select-String -Pattern '^- id=(\d+).*powershell' | ForEach-Object { $_.Matches[0].Groups[1].Value }) | Select-Object -Last 1
if ($id) { "removing B ($id) from the Windows side"; & $probe remove $id 2>&1 }

$qs.WaitForExit(30000) | Out-Null
if (-not $qs.HasExited) { "qs still running, stopping it"; Stop-Process -Id $qs.Id -Force }
"qs exit code: $($qs.ExitCode)"
"--- logo cache"
Get-ChildItem "$env:LOCALAPPDATA\quickshell\cache\notification-icons" -ErrorAction SilentlyContinue | ForEach-Object { "$($_.FullName) $($_.Length)" }
"--- toasts after"
& $probe list 2>&1 | Select-String '^- id|text'
"--- qs stdout"
Get-Content $out
"--- qs stderr"
Get-Content $err
