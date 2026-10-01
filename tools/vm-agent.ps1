# ii-windows VM agent. boot.ps1 (installed by vm-bootstrap.ps1) downloads this file from the host
# on every start and runs it in the logged-on user's session, so jobs can start GUI programs.
# $d is set by boot.ps1.
$ErrorActionPreference = 'Continue'
$utf8 = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = $utf8
[Console]::OutputEncoding = $utf8

$gwHost = '<linux user>@192.168.122.1'
$sshArgs = @('-i', "$d\key", '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=accept-new',
	'-o', 'ConnectTimeout=5', '-o', 'ServerAliveInterval=15')
# For jobs: cmd.exe pipes bytes untouched (PowerShell 5.1 pipelines would mangle a tar stream).
$env:IIW_GW = "ssh -i `"$d\key`" -o BatchMode=yes -o StrictHostKeyChecking=accept-new $gwHost"
$env:IIW_ROOT = 'C:\ii-windows'
New-Item -Force -ItemType Directory "$env:IIW_ROOT\logs", "$d\jobs" | Out-Null

while ($true) {
	$resp = @(& ssh.exe @sshArgs $gwHost poll 2>$null)
	if ($LASTEXITCODE -ne 0) { Start-Sleep 5; continue }
	if ($resp.Count -eq 0 -or -not $resp[0]) { continue }

	$id = $resp[0].Trim()
	if ($id -eq 'reload-agent') { return }  # boot.ps1 fetches the new agent body

	$file = "$d\jobs\$id.ps1"
	$outFile = "$d\jobs\$id.out"
	$body = "[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding `$false`r`n" +
		(($resp | Select-Object -Skip 1) -join "`r`n")
	[IO.File]::WriteAllText($file, $body, $utf8)
	# Output goes to a file through cmd.exe, not to a pipe: programs a job starts in the
	# background inherit the job's handles, and a pipe would stay open until they exit.
	cmd /c "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$file`" > `"$outFile`" 2>&1"
	$rc = $LASTEXITCODE
	if ($null -eq $rc) { $rc = 0 }
	cmd /c "type `"$outFile`" | $env:IIW_GW result $id $rc" 2>$null
	Remove-Item $file, $outFile -ErrorAction SilentlyContinue
}
