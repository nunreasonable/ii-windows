# One-time setup of the ii-windows test agent inside the Windows VM. Run from a normal (not
# elevated) PowerShell in the VM:
#
#   iex (ssh <linux user>@192.168.122.1 iiw-vm bootstrap | Out-String)
#
# It asks for the Linux password twice (this download, then authorizing the agent's key).
# After that the agent starts with every Windows logon and needs no password.
& {
$ErrorActionPreference = 'Stop'
$d = "$env:LOCALAPPDATA\iiw-agent"
New-Item -Force -ItemType Directory $d | Out-Null

if (-not (Test-Path "$d\key")) {
	# '""' is how Windows PowerShell 5.1 passes an empty passphrase to a native program.
	ssh-keygen -q -t ed25519 -N '""' -C iiw-agent -f "$d\key"
}

Write-Host 'Authorizing the agent key on the host (Linux password again):'
Get-Content "$d\key.pub" | ssh <linux user>@192.168.122.1 iiw-vm authorize

# boot.ps1 never changes: it fetches the real agent from the host on every start, so the agent
# can be updated from Linux without touching the VM again.
Set-Content -Encoding UTF8 "$d\boot.ps1" @'
$d = "$env:LOCALAPPDATA\iiw-agent"
# Single instance: the logon shortcut and the watchdog task both start this script.
$created = $false
$mutex = New-Object System.Threading.Mutex($true, 'Local\iiw-agent', [ref]$created)
if (-not $created) { exit }
while ($true) {
	$a = ssh -i "$d\key" -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o ConnectTimeout=5 <linux user>@192.168.122.1 agent 2>$null
	if ($LASTEXITCODE -eq 0 -and $a) { Invoke-Expression ($a -join "`n") }
	Start-Sleep 10
}
'@

$bootArgs = "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$d\boot.ps1`""
$lnk = (New-Object -ComObject WScript.Shell).CreateShortcut("$([Environment]::GetFolderPath('Startup'))\iiw-agent.lnk")
$lnk.TargetPath = 'powershell.exe'
$lnk.Arguments = $bootArgs
$lnk.Save()

# Watchdog: restarts the agent within 5 minutes if it dies (no admin rights needed). conhost
# --headless keeps the console from flashing every time the task fires.
schtasks /create /f /tn iiw-agent /sc minute /mo 5 /rl limited `
	/tr "conhost.exe --headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$d\boot.ps1`"" | Out-Null

Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
	Where-Object { $_.CommandLine -like '*iiw-agent\boot.ps1*' } |
	ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Start-Process powershell.exe $bootArgs
Write-Host 'ii-windows agent installed and running.'
}
