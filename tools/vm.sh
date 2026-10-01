#!/usr/bin/env bash
# Drive the Windows test VM through its agent. The VM connects to the host (tools/vm-gateway.sh,
# installed in the VM once with tools/vm-bootstrap.ps1); this script queues PowerShell jobs for
# the agent and waits for their output. Works with whichever of win11 / win11-gpu is running
# (they share one disk), and never starts win11-gpu itself.
set -euo pipefail
. "$(dirname "$0")/env.sh"

Q="$IIW/build/vmq"
mkdir -p "$Q/pending" "$Q/running" "$Q/done"
virsh_() { virsh -c "$VM_URI" "$@"; }
running() { virsh_ list --state-running --name 2>/dev/null | grep -qx "$1"; }

agent_age() {
	local seen
	seen=$(cat "$Q/agent.seen" 2>/dev/null || echo 0)
	echo $(($(date +%s) - seen))
}

# job [timeout]: PowerShell script on stdin -> output on stdout, exit code of the job
job() {
	local timeout="${1:-120}" id
	id="$(date +%s%N)-$RANDOM"
	{ echo "#timeout=$((timeout - 5))"; cat; } > "$Q/pending/.$id"
	mv "$Q/pending/.$id" "$Q/pending/$id"
	for _ in $(seq $((timeout * 2))); do
		if [ -f "$Q/done/$id.rc" ]; then
			cat "$Q/done/$id.out"
			local rc
			rc=$(cat "$Q/done/$id.rc")
			rm -f "$Q/done/$id.out" "$Q/done/$id.rc"
			return "$rc"
		fi
		sleep 0.5
	done
	rm -f "$Q/pending/$id"
	echo "vm.sh: job $id timed out (agent last seen $(agent_age)s ago)" >&2
	return 124
}

ps_quote() { printf "'%s'" "${1//\'/\'\'}"; }

cmd="${1:-}"; shift || true
case "$cmd" in
up)
	if running win11-gpu || running win11; then
		:
	else
		virsh_ start win11 >/dev/null
		echo "started win11"
	fi
	echo "waiting for the agent..."
	for _ in $(seq 120); do
		[ "$(agent_age)" -lt 30 ] && { echo "agent up"; exit 0; }
		sleep 2
	done
	echo "agent not seen; is it installed? (tools/vm-bootstrap.ps1)" >&2; exit 1 ;;
down)
	running win11 && virsh_ shutdown win11 || echo "win11 is not running (win11-gpu is never stopped from here)" ;;
status)
	for d in win11 win11-gpu; do printf '%-10s %s\n' "$d" "$(virsh_ domstate $d)"; done
	echo "agent last seen $(agent_age)s ago" ;;
job) job "${1:-120}" ;;
push)
	# push <dist name>: mirror dist/<name> to C:\ii-windows\<name>
	name="$1"
	[ -d "$IIW/dist/$name" ] || { echo "no dist/$name" >&2; exit 1; }
	job 600 <<EOF
New-Item -Force -ItemType Directory "\$env:IIW_ROOT\\$name" | Out-Null
cmd /c "%IIW_GW% fetch $name | tar -xf - -C %IIW_ROOT%\\$name"
if (\$LASTEXITCODE -ne 0) { exit \$LASTEXITCODE }
"pushed $name: " + (Get-ChildItem -Recurse "\$env:IIW_ROOT\\$name" | Measure-Object -Sum Length).Sum + " bytes"
EOF
	;;
run)
	# run <exe under C:\ii-windows> [args...]: start detached in the user session, logs in logs\
	exe="$1"; shift
	base=$(basename "${exe//\\//}" .exe)
	argl=""
	for a in "$@"; do argl+="$(ps_quote "$a"),"; done
	job <<EOF
\$exe = Join-Path \$env:IIW_ROOT $(ps_quote "$exe")
\$p = @{ FilePath = \$exe; WorkingDirectory = (Split-Path \$exe); PassThru = \$true
	RedirectStandardOutput = "\$env:IIW_ROOT\\logs\\$base.out"; RedirectStandardError = "\$env:IIW_ROOT\\logs\\$base.err" }
\$argv = @(${argl%,})
if (\$argv.Count) { \$p.ArgumentList = \$argv }
\$proc = Start-Process @p
"started \$(\$proc.Id) \$exe"
EOF
	;;
kill)
	names=""
	for n in ${*:-qsw qs iiw_hello}; do names+="$(ps_quote "$n"),"; done
	job <<EOF
Get-Process -Name @(${names%,}) -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
"remaining: " + @(Get-Process -Name @(${names%,}) -ErrorAction SilentlyContinue).Count
EOF
	;;
log)
	f="${1:-qsw}"
	job <<EOF
Get-Content -Tail ${2:-200} "\$env:IIW_ROOT\\logs\\$f.out", "\$env:IIW_ROOT\\logs\\$f.err" -ErrorAction SilentlyContinue
EOF
	;;
ipc)
	argl=""
	for a in "$@"; do argl+="$(ps_quote "$a"),"; done
	job <<EOF
& "\$env:IIW_ROOT\\ii-windows\\qs.exe" ipc call @(${argl%,}) 2>&1
EOF
	;;
shot)
	# Captured inside Windows, so it also works with the RTX passed through (win11-gpu).
	out="${1:-$IIW/build/shots/$(date +%H%M%S).png}"
	mkdir -p "$(dirname "$out")"
	job <<'EOF' | tr -d '\r\n' | base64 -d > "$out"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public static class Dpi { [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(System.IntPtr v); }'
[void][Dpi]::SetProcessDpiAwarenessContext([IntPtr]-4)
$b = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
[Convert]::ToBase64String($ms.ToArray())
EOF
	echo "$out" ;;
ii)
	# ii start|stop: install dist/ii-windows/config/ii as %LOCALAPPDATA%\quickshell\ii, seed the
	# colors on first run, and run it like `qs -c ii` on Linux.
	case "${1:-start}" in
	start) job 60 <<'EOF'
$dir = "$env:IIW_ROOT\ii-windows"
Get-Process qsw -ErrorAction SilentlyContinue | Stop-Process -Force
robocopy "$dir\config\ii" "$env:LOCALAPPDATA\quickshell\ii" /MIR /NFL /NDL /NJH /NJS /NP | Out-Null
$colors = "$env:LOCALAPPDATA\quickshell\user\generated\colors.json"
if (-not (Test-Path $colors)) {
	New-Item -Force -ItemType Directory (Split-Path $colors) | Out-Null
	Copy-Item "$dir\config\ii\defaults\windows\colors.json" $colors
}
$p = Start-Process -PassThru -FilePath "$dir\qsw.exe" -ArgumentList '-c', 'ii' -WorkingDirectory $dir `
	-RedirectStandardError "$env:IIW_ROOT\logs\ii.err" -RedirectStandardOutput "$env:IIW_ROOT\logs\ii.out"
Start-Sleep 8
"alive: " + (-not $p.HasExited)
Get-Content "$env:IIW_ROOT\logs\ii.out", "$env:IIW_ROOT\logs\ii.err" -Tail 60
EOF
	;;
	stop) job 30 <<'EOF'
Get-Process qsw -ErrorAction SilentlyContinue | Stop-Process -Force
"stopped"
EOF
	;;
	esac ;;
reload-agent)
	: > "$Q/pending/reload-agent"
	echo "agent will reload on its next poll" ;;
restart-agent)
	# For an agent too old to understand reload-agent: start a fresh boot.ps1, then kill the
	# old one. The job's own result is lost with the old agent, so don't wait for it.
	cat > "$Q/pending/$(date +%s%N)-restart" <<'EOF'
$old = Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
	Where-Object { $_.CommandLine -like '*iiw-agent\boot.ps1*' }
Start-Process powershell.exe "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$env:LOCALAPPDATA\iiw-agent\boot.ps1`""
$old | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
EOF
	echo "restart queued" ;;
*)
	cat >&2 <<EOF
usage: vm.sh up|down|status|push <dist>|run <exe> [args]|kill [names]|log [name] [lines]|ipc <args>|shot [out.png]|job [timeout] < script.ps1|ii start|stop|reload-agent|restart-agent
EOF
	exit 1 ;;
esac
