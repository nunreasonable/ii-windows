#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"

VM="${IIW_VM:-WIN10-II}"
DOMAIN="${SMOKE_DOMAIN:-win10}"
START=0
[ "${1:-}" = "--start" ] && START=1

virsh_() { virsh -c "$VM_URI" "$@"; }
seen() { echo $(($(date +%s) - $(cat "$IIW/build/vmq/seen/$VM" 2>/dev/null || echo 0))); }

if [ "$(virsh_ domstate "$DOMAIN" 2>/dev/null)" != "running" ]; then
	if [ "$START" != 1 ]; then
		echo "smoke-test: $DOMAIN is not running (pass --start to boot it and shut it down afterwards)" >&2
		exit 2
	fi
	virsh_ start "$DOMAIN" >/dev/null
	STARTED=1
fi
for _ in $(seq 60); do [ "$(seen)" -lt 10 ] && break; sleep 5; done
if [ "$(seen)" -ge 10 ]; then
	echo "smoke-test: the agent in $VM isn't answering" >&2
	exit 2
fi

[ -d "$IIW/dist/ii-windows" ] || { echo "no dist/ii-windows - run tools/deploy-ii.sh first" >&2; exit 1; }
untracked=$(git -C "$IIW/ii" ls-files --others --exclude-standard | grep -E '\.(qml|js|ps1|json)$' || true)
if [ -n "$untracked" ]; then
	echo "WARN: untracked files in ii are not packaged by deploy-ii.sh:" >&2
	sed 's/^/  /' <<<"$untracked" >&2
fi
STAGE="$IIW/dist/smoketest"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$IIW/tools/smoke/smoke.ps1" "$IIW/tools/smoke/fonttest.qml" "$IIW/tools/smoke/inherit.qml" "$STAGE/"
python3 "$IIW/tools/qml-compile-check.py" "$IIW/ii" > "$STAGE/compilecheck.qml"
SETUP="$IIW/installer/target/x86_64-pc-windows-msvc/release/ii-windows-setup.exe"
[ -f "$SETUP" ] && cp "$SETUP" "$STAGE/"

IIW_VM="$VM" "$IIW/tools/vm.sh" push ii-windows >/dev/null
IIW_VM="$VM" "$IIW/tools/vm.sh" push smoketest >/dev/null
rm -rf "$STAGE"

set +e
out=$(echo "& (Join-Path \$env:IIW_ROOT 'smoketest\\smoke.ps1')" | IIW_VM="$VM" "$IIW/tools/vm.sh" job 480 | tr -d '\r')
set -e
echo "$out"

if [ "${STARTED:-0}" = 1 ]; then
	virsh_ shutdown "$DOMAIN" >/dev/null
fi

grep -q "^RESULT: all passed" <<<"$out"
