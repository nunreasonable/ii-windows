# First logon of the Windows 10 test VM (autounattend.xml): puts the agent key that
# tools/vm-win10.sh already pinned on the host in place and runs the usual bootstrap, which then
# skips the password prompts.
$ErrorActionPreference = 'Stop'
$d = "$env:LOCALAPPDATA\iiw-agent"
New-Item -Force -ItemType Directory $d | Out-Null

Copy-Item -Force "$PSScriptRoot\key", "$PSScriptRoot\key.pub" $d
# OpenSSH refuses a private key other users can read.
icacls "$d\key" /inheritance:r /grant:r "${env:USERNAME}:F" | Out-Null

& "$PSScriptRoot\bootstrap.ps1"
