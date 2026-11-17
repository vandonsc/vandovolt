@echo off
setlocal
echo Feche o Peripheral Battery pelo menu Sair antes de continuar.
pause
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PeripheralBattery /f
if exist "%LOCALAPPDATA%\PeripheralBattery\peripheral-battery.exe" del "%LOCALAPPDATA%\PeripheralBattery\peripheral-battery.exe"
if exist "%LOCALAPPDATA%\PeripheralBattery" rd "%LOCALAPPDATA%\PeripheralBattery"
pause
