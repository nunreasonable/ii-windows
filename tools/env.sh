IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export IIW
export QT_VERSION=6.11.2
export QT_WIN="$IIW/toolchain/qt/$QT_VERSION/msvc2022_64"
export QT_HOST="$IIW/toolchain/qt/$QT_VERSION/gcc_64"
export XWIN="$IIW/toolchain/xwin"

export PATH="$PATH:$IIW/toolchain/bin"

export VM_NAME=win11
export VM_URI=qemu:///system
export VM_KEY="$HOME/.ssh/ii-windows-vm"
export VM_USER="${VM_USER:-}"
export VM_DIR='C:/ii-windows'
if [ -f "$IIW/tools/vm.local.sh" ]; then . "$IIW/tools/vm.local.sh"; fi
