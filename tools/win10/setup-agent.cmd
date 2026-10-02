@echo off
rem Re-runs the first-logon agent setup (double-click it from the UNATTEND disc).
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup-agent.ps1"
pause
