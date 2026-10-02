#!/usr/bin/env bash
# Host side of the Windows test VM channel. The VM connects to the host (never the other way
# around): its agent (tools/vm-agent.ps1) logs in with a key that is pinned to this script as a
# forced command, so that key can only do what the "agent commands" below allow.
#
#   agent commands (VM key, via SSH_ORIGINAL_COMMAND):
#     agent            print the current agent body (vm-agent.ps1)
#     poll             wait up to 20 s for a queued job; print "<id>" then the PowerShell script
#     fetch <name>     tar stream of dist/<name>
#     result <id> <rc> store the job's output (stdin) and exit code
#
#   setup commands (the user, logged in with their own password from inside the VM):
#     bootstrap        print the PowerShell installer (vm-bootstrap.ps1)
#     authorize        read the VM's public key on stdin and pin it to this script
#
# Installed as ~/.local/bin/iiw-vm, so the user only has to type a short line in the VM.
set -euo pipefail

self=$(readlink -f "${BASH_SOURCE[0]}")
IIW=$(cd "$(dirname "$self")/.." && pwd)
Q="$IIW/build/vmq"
mkdir -p "$Q/pending" "$Q/running" "$Q/done"

valid_name() { [[ "$1" =~ ^[A-Za-z0-9._-]+$ ]]; }

# How the VM reaches this host: this user at the libvirt bridge address, unless IIW_GW_HOST says
# otherwise. Filled into the PowerShell served below in place of @IIW_GW_HOST@.
gw_host() {
	if [ -n "${IIW_GW_HOST:-}" ]; then echo "$IIW_GW_HOST"; return; fi
	local ip
	ip=$(ip -4 -o addr show "${IIW_GW_BRIDGE:-virbr0}" 2>/dev/null | awk '{print $4}' | cut -d/ -f1 | head -1)
	echo "$(id -un)@${ip:-192.168.122.1}"
}
serve() { sed "s/@IIW_GW_HOST@/$(gw_host)/g" "$1"; }

if [ -n "${SSH_ORIGINAL_COMMAND+x}" ]; then
	# Forced command: only agent commands.
	read -r cmd a1 a2 _ <<<"$SSH_ORIGINAL_COMMAND"
	date +%s > "$Q/agent.seen"
	case "$cmd" in
	agent) serve "$IIW/tools/vm-agent.ps1" ;;
	poll)
		for _ in $(seq 40); do
			job=$(find "$Q/pending" -maxdepth 1 -type f ! -name '.*' -printf '%f\n' | sort | head -1)
			if [ -n "$job" ] && mv "$Q/pending/$job" "$Q/running/$job" 2>/dev/null; then
				printf '%s\n' "$job"
				cat "$Q/running/$job"
				exit 0
			fi
			sleep 0.5
			date +%s > "$Q/agent.seen"
		done ;;
	fetch)
		valid_name "${a1:-}" && [ -d "$IIW/dist/$a1" ] || { echo "no such dist: ${a1:-}" >&2; exit 2; }
		tar -C "$IIW/dist/$a1" -cf - . ;;
	result)
		valid_name "${a1:-}" && [[ "${a2:-}" =~ ^-?[0-9]+$ ]] || exit 2
		cat > "$Q/done/$a1.out.tmp"
		mv "$Q/done/$a1.out.tmp" "$Q/done/$a1.out"
		echo "$a2" > "$Q/done/$a1.rc"
		rm -f "$Q/running/$a1" ;;
	*) echo "unknown agent command" >&2; exit 2 ;;
	esac
	exit 0
fi

case "${1:-}" in
bootstrap) serve "$IIW/tools/vm-bootstrap.ps1" ;;
authorize)
	key=$(head -c 1024 | tr -d '\r' | head -1)
	[[ "$key" =~ ^ssh-ed25519\ [A-Za-z0-9+/=]+(\ .*)?$ ]] || { echo "not an ed25519 public key" >&2; exit 2; }
	mkdir -p ~/.ssh && chmod 700 ~/.ssh
	touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys
	blob=$(awk '{print $2}' <<<"$key")
	# Replace an older key of the agent instead of piling them up.
	sed -i '/ iiw-agent$/d' ~/.ssh/authorized_keys
	printf 'restrict,from="192.168.122.0/24",command="%s" ssh-ed25519 %s iiw-agent\n' \
		"$self" "$blob" >> ~/.ssh/authorized_keys
	echo "authorized" ;;
*) echo "usage: iiw-vm bootstrap|authorize" >&2; exit 2 ;;
esac
