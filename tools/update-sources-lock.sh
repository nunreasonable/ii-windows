#!/usr/bin/env bash
set -euo pipefail
IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for repo in quickshell ii; do
	if [ -n "$(git -C "$IIW/$repo" status --porcelain --untracked-files=no)" ]; then
		echo "$repo has uncommitted changes" >&2
		exit 1
	fi
done

{
	echo "quickshell https://github.com/nunreasonable/quickshell $(git -C "$IIW/quickshell" rev-parse HEAD)"
	echo "ii https://github.com/nunreasonable/dots-hyprland $(git -C "$IIW/ii" rev-parse HEAD)"
} > "$IIW/sources.lock"
cat "$IIW/sources.lock"
