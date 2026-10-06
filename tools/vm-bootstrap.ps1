& {
$ErrorActionPreference = 'Stop'
$d = "$env:LOCALAPPDATA\iiw-agent"
New-Item -Force -ItemType Directory $d | Out-Null

if (-not (Test-Path "$d\key")) {
	ssh-keygen -q -t ed25519 -N '""' -C iiw-agent -f "$d\key"
}

$ErrorActionPreference = 'Continue'
ssh -i "$d\key" -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o ConnectTimeout=5 @IIW_GW_HOST@ agent 2>&1 | Out-Null
$keyWorks = $LASTEXITCODE -eq 0
$ErrorActionPreference = 'Stop'
if (-not $keyWorks) {
	Write-Host 'Authorizing the agent key on the host (Linux password again):'
	Get-Content "$d\key.pub" | ssh @IIW_GW_HOST@ iiw-vm authorize
}

Set-Content -Encoding UTF8 "$d\boot.ps1" @'
$d = "$env:LOCALAPPDATA\iiw-agent"
# Single instance: the logon shortcut and the watchdog task both start this script.
$created = $false
$mutex = New-Object System.Threading.Mutex($true, 'Local\iiw-agent', [ref]$created)
if (-not $created) { exit }
while ($true) {
	$a = ssh -n -i "$d\key" -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o ConnectTimeout=5 @IIW_GW_HOST@ agent 2>$null
	if ($LASTEXITCODE -eq 0 -and $a) { Invoke-Expression ($a -join "`n") }
	Start-Sleep 10
}
'@

$bootArgs = "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$d\boot.ps1`""
$lnk = (New-Object -ComObject WScript.Shell).CreateShortcut("$([Environment]::GetFolderPath('Startup'))\iiw-agent.lnk")
$lnk.TargetPath = 'powershell.exe'
$lnk.Arguments = $bootArgs
$lnk.Save()

schtasks /create /f /tn iiw-agent /sc minute /mo 5 /rl limited `
	/tr "conhost.exe --headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$d\boot.ps1`"" | Out-Null

Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
	Where-Object { $_.CommandLine -like '*iiw-agent\boot.ps1*' } |
	ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Process powershell.exe $bootArgs
Write-Host 'ii-windows agent installed and running.'
}
