$ErrorActionPreference = 'Continue'
$utf8 = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = $utf8
[Console]::OutputEncoding = $utf8

$gwHost = '@IIW_GW_HOST@'
$sshArgs = @('-n', '-i', "$d\key", '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=accept-new',
	'-o', 'ConnectTimeout=5', '-o', 'ServerAliveInterval=15')
$env:IIW_GW = "ssh -i `"$d\key`" -o BatchMode=yes -o StrictHostKeyChecking=accept-new $gwHost"
$env:IIW_ROOT = 'C:\ii-windows'
New-Item -Force -ItemType Directory "$env:IIW_ROOT\logs", "$d\jobs" | Out-Null
$bootFile = "$d\boot.ps1"
if (Test-Path $bootFile) {
	$bootText = [IO.File]::ReadAllText($bootFile)
	if ($bootText.Contains('$a = ssh -i')) { [IO.File]::WriteAllText($bootFile, $bootText.Replace('$a = ssh -i', '$a = ssh -n -i'), $utf8) }
}

while ($true) {
	$resp = @(& ssh.exe @sshArgs $gwHost poll 2>$null)
	if ($LASTEXITCODE -ne 0) { Start-Sleep 5; continue }
	if ($resp.Count -eq 0 -or -not $resp[0]) { continue }

	$id = $resp[0].Trim()
	if ($id -eq 'reload-agent') { break }

	$file = "$d\jobs\$id.ps1"
	$outFile = "$d\jobs\$id.out"
	$body = "[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding `$false`r`n" +
		(($resp | Select-Object -Skip 1) -join "`r`n")
	[IO.File]::WriteAllText($file, $body, $utf8)
	$timeout = 120
	if ($resp.Count -gt 1 -and $resp[1] -match '^#timeout=(\d+)') { $timeout = [int]$Matches[1] }
	$psi = New-Object System.Diagnostics.ProcessStartInfo 'cmd.exe',
		"/c powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$file`" > `"$outFile`" 2>&1"
	$psi.UseShellExecute = $false
	$psi.CreateNoWindow = $true
	$proc = [System.Diagnostics.Process]::Start($psi)
	if ($proc.WaitForExit($timeout * 1000)) {
		$rc = $proc.ExitCode
	} else {
		taskkill.exe /T /F /PID $proc.Id | Out-Null
		Add-Content $outFile "`r`n[agent] job killed after ${timeout}s"
		$rc = 124
	}
	cmd /c "type `"$outFile`" | $env:IIW_GW result $id $rc" 2>$null
	Remove-Item $file, $outFile -ErrorAction SilentlyContinue
}
