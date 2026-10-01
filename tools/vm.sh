#!/usr/bin/env bash
# Drive the win11 test VM: start, push builds, run GUI programs in the interactive session,
# take screenshots, read logs. Never touches the win11-gpu domain.
set -euo pipefail
. "$(dirname "$0")/env.sh"

virsh_() { virsh -c "$VM_URI" "$@"; }

vm_ip() {
	local mac
	mac=$(virsh_ domiflist "$VM_NAME" | awk '$2 == "network" { print $5 }')
	virsh_ net-dhcp-leases default | awk -v mac="$mac" '$0 ~ mac { split($5, a, "/"); print a[1] }' | tail -1
}

ssh_() {
	[ -n "$VM_USER" ] || { echo "set VM_USER in tools/vm.local.sh" >&2; exit 1; }
	ssh -i "$VM_KEY" -o StrictHostKeyChecking=accept-new -o ConnectTimeout=5 -o BatchMode=yes \
		"$VM_USER@$(vm_ip)" "$@"
}

# Windows paths for cmd.exe
win() { echo "${1//\//\\}"; }

cmd="${1:-}"; shift || true
case "$cmd" in
up)
	[ "$VM_NAME" = win11 ] || { echo "refusing to start $VM_NAME" >&2; exit 1; }
	if [ "$(virsh_ domstate "$VM_NAME")" != running ]; then virsh_ start "$VM_NAME"; fi
	echo "waiting for ssh..."
	for _ in $(seq 90); do
		if [ -n "$(vm_ip)" ] && ssh_ "echo ok" >/dev/null 2>&1; then echo "up: $(vm_ip)"; exit 0; fi
		sleep 2
	done
	echo "ssh not reachable" >&2; exit 1 ;;
down) virsh_ shutdown "$VM_NAME" ;;
ip) vm_ip ;;
ssh) ssh_ "$@" ;;
push)
	# push <local dir> [remote subdir]: mirror a dist folder into $VM_DIR/<subdir>
	src="$1"; dst="$VM_DIR/${2:-$(basename "$src")}"
	ssh_ "if not exist \"$(win "$dst")\" mkdir \"$(win "$dst")\""
	tar -C "$src" -cf - . | ssh_ "tar -xf - -C \"$dst\""
	echo "pushed $src -> $dst" ;;
run)
	# run <command line...>: start a program in the logged-on (interactive) session.
	# Output goes to $VM_DIR/logs/run.log.
	line="$*"
	ssh_ "if not exist \"$(win "$VM_DIR")\\logs\" mkdir \"$(win "$VM_DIR")\\logs\""
	printf '@echo off\r\ncd /d %s\r\n%s > %s\\logs\\run.log 2>&1\r\n' \
		"$(win "$VM_DIR")" "$line" "$(win "$VM_DIR")" | ssh_ "more > \"$(win "$VM_DIR")\\run.cmd\""
	ssh_ "schtasks /create /f /tn iiw-run /sc once /st 00:00 /it /tr \"$(win "$VM_DIR")\\run.cmd\" >nul && schtasks /run /tn iiw-run >nul"
	echo "started: $line" ;;
kill)
	for exe in "${@:-qsw.exe qs.exe iiw_hello.exe}"; do ssh_ "taskkill /f /im $exe" 2>/dev/null || true; done ;;
log) ssh_ "type \"$(win "$VM_DIR")\\logs\\${1:-run.log}\"" ;;
shot)
	out="${1:-$IIW/build/shots/$(date +%H%M%S).png}"
	mkdir -p "$(dirname "$out")"
	tmp=$(mktemp --suffix=.ppm)
	virsh_ screenshot "$VM_NAME" "$tmp" >/dev/null
	magick "$tmp" "$out"; rm -f "$tmp"
	echo "$out" ;;
ipc) ssh_ "\"$(win "$VM_DIR")\\ii-windows\\qs.exe\" ipc call $*" ;;
*)
	echo "usage: vm.sh up|down|ip|ssh|push <dir> [sub]|run <cmd>|kill [exe..]|log [file]|shot [out.png]|ipc <args>" >&2
	exit 1 ;;
esac
