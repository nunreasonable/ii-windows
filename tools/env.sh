IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export IIW
export ARCH="${ARCH:-x64}"
case "$ARCH" in
x64) QT_ARCH_DIR=msvc2022_64 ;;
arm64) QT_ARCH_DIR=msvc2022_arm64 ;;
*)
	echo "env.sh: unknown ARCH '$ARCH' (want x64 or arm64)" >&2
	return 1 2>/dev/null || exit 1
	;;
esac
export QT_VERSION=6.11.2
export QT_WIN="${QT_WIN:-$IIW/toolchain/qt/$QT_VERSION/$QT_ARCH_DIR}"
export QT_HOST="${QT_HOST:-$IIW/toolchain/qt/$QT_VERSION/gcc_64}"
export XWIN="$IIW/toolchain/xwin"

export PATH="$PATH:$IIW/toolchain/bin"

export VM_NAME=win11
export VM_URI=qemu:///system
export VM_KEY="$HOME/.ssh/ii-windows-vm"
export VM_USER="${VM_USER:-}"
export VM_DIR='C:/ii-windows'
if [ -f "$IIW/tools/vm.local.sh" ]; then . "$IIW/tools/vm.local.sh"; fi
