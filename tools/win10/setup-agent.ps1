$ErrorActionPreference = 'Stop'
$d = "$env:LOCALAPPDATA\iiw-agent"
New-Item -Force -ItemType Directory $d | Out-Null

Copy-Item -Force "$PSScriptRoot\key", "$PSScriptRoot\key.pub" $d
icacls "$d\key" /inheritance:r /grant:r "${env:USERNAME}:F" | Out-Null

& "$PSScriptRoot\bootstrap.ps1"
