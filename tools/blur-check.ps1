# blur-check.ps1: read-only report of the blur backdrop windows of a running ii (branch `blur`).
# Run while ii is up:  tools/vm.sh job < tools/blur-check.ps1
# For every QuickshellBlurBackdrop window: visible, rect, topmost, and the window directly above
# it in the z-order, which must be its panel (a Qt window of qs/qsw.exe with the same rect).
# Nothing is started, stopped or changed.

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class BlurCheck {
	public delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
	[DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
	[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder name, int size);
	[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
	[DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
	[DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr hwnd, uint cmd);
	[DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtr(IntPtr hwnd, int index);
	[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
	public struct RECT { public int Left, Top, Right, Bottom; }

	public static string ClassOf(IntPtr hwnd) {
		var name = new StringBuilder(256);
		GetClassName(hwnd, name, 256);
		return name.ToString();
	}

	public static string Describe(IntPtr hwnd) {
		RECT r;
		GetWindowRect(hwnd, out r);
		uint pid;
		GetWindowThreadProcessId(hwnd, out pid);
		long ex = GetWindowLongPtr(hwnd, -20).ToInt64();
		return String.Format("0x{0:x} {1} pid={2} visible={3} topmost={4} rect={5},{6} {7}x{8} exstyle=0x{9:x}",
			hwnd.ToInt64(), ClassOf(hwnd), pid, IsWindowVisible(hwnd), (ex & 0x8) != 0,
			r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top, ex);
	}
}
"@

$transparency = (Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize' -ErrorAction SilentlyContinue).EnableTransparency
"EnableTransparency = $transparency"
Get-Process qs, qsw -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Name).exe pid $($_.Id), $([math]::Round($_.CPU, 1)) s CPU, $([math]::Round($_.WorkingSet64 / 1MB)) MB" }

$backdrops = New-Object System.Collections.ArrayList
[void][BlurCheck]::EnumWindows({
	param($hwnd, $lParam)
	if ([BlurCheck]::ClassOf($hwnd) -eq 'QuickshellBlurBackdrop') { [void]$backdrops.Add($hwnd) }
	$true
}, [IntPtr]::Zero)

"$($backdrops.Count) backdrop window(s)"
foreach ($hwnd in $backdrops) {
	"backdrop: " + [BlurCheck]::Describe($hwnd)
	# GW_HWNDPREV = 3: the window right above
	$above = [BlurCheck]::GetWindow($hwnd, 3)
	if ($above -ne [IntPtr]::Zero) { "  above:  " + [BlurCheck]::Describe($above) } else { "  above:  (none, top of the z-order)" }
}
