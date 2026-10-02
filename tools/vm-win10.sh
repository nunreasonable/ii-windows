#!/usr/bin/env bash
# Creates the Windows 10 22H2 test VM ("win10") from Microsoft's ISO, unattended:
#
#   tools/vm-win10.sh create <Win10_22H2_*.iso>
#
# The install takes a while and needs no clicks: autounattend.xml (tools/win10/) installs Windows
# 10 Pro with the generic key (not activated), a local administrator that logs on by itself, and
# the test agent. The agent's key is pinned on the host first (iiw-vm authorize win10), so
# nothing asks for the Linux password. Afterwards tools/vm.sh drives it like the win11 VMs; run
# only one test VM at a time, since they all pull from the same job queue.
#
# The VM's local user and password live in build/win10-vm/credentials (not in git).
set -euo pipefail
. "$(dirname "$0")/env.sh"

NAME=win10
DISK=/var/lib/libvirt/images/$NAME.qcow2
WORK="$IIW/build/win10-vm"
virsh_() { virsh -c "$VM_URI" "$@"; }

case "${1:-}" in
create)
	iso="${2:?usage: vm-win10.sh create <iso>}"
	[ -f "$iso" ] || { echo "no $iso" >&2; exit 1; }
	if virsh_ dominfo "$NAME" >/dev/null 2>&1; then
		echo "$NAME already exists" >&2
		exit 1
	fi

	mkdir -p "$WORK/disc"
	if [ ! -f "$WORK/credentials" ]; then
		printf 'user=ii\npassword=%s\n' "$(head -c 12 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 12)" \
			> "$WORK/credentials"
		chmod 600 "$WORK/credentials"
	fi
	. "$WORK/credentials"

	[ -f "$WORK/disc/key" ] || ssh-keygen -q -t ed25519 -N '' -C iiw-agent-win10 -f "$WORK/disc/key"
	"$IIW/tools/vm-gateway.sh" authorize win10 < "$WORK/disc/key.pub"

	sed -e "s/@USER@/$user/g" -e "s/@PASSWORD@/$password/g" \
		"$IIW/tools/win10/autounattend.xml" > "$WORK/disc/autounattend.xml"
	cp -f "$IIW/tools/win10/setup-agent.ps1" "$IIW/tools/win10/setup-agent.cmd" "$WORK/disc/"
	# The bootstrap as the gateway would serve it, with this host's address filled in.
	"$IIW/tools/vm-gateway.sh" bootstrap > "$WORK/disc/bootstrap.ps1"
	xorriso -as mkisofs -quiet -J -r -V UNATTEND -o "$WORK/unattend.iso" "$WORK/disc"

	# Into libvirt's default pool through libvirt itself (the directory is root's).
	vol="$NAME-unattend.iso"
	virsh_ vol-delete --pool default "$vol" >/dev/null 2>&1 || true
	virsh_ vol-create-as default "$vol" "$(stat -c %s "$WORK/unattend.iso")" --format raw >/dev/null
	virsh_ vol-upload --pool default "$vol" "$WORK/unattend.iso"

	virt-install --connect "$VM_URI" \
		--name "$NAME" \
		--osinfo win10 \
		--memory 8192 --vcpus 6 --cpu host-passthrough \
		--machine q35 \
		--disk path="$DISK",size=64,format=qcow2,bus=sata \
		--disk path="$iso",device=cdrom,bus=sata,readonly=on \
		--disk path="/var/lib/libvirt/images/$NAME-unattend.iso",device=cdrom,bus=sata,readonly=on \
		--network network=default,model=e1000e \
		--graphics spice,listen=127.0.0.1 \
		--video vga \
		--controller usb,model=qemu-xhci \
		--input tablet,bus=usb \
		--boot hd,cdrom \
		--noautoconsole
	echo "installing; watch it with: virt-manager --connect $VM_URI --show-domain-console $NAME"
	;;
*)
	echo "usage: vm-win10.sh create <iso>" >&2
	exit 2
	;;
esac
