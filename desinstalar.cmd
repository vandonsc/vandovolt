@echo off
setlocal
echo Feche o VandoVolt pelo menu Sair antes de continuar.
pause
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v VandoVolt /f >nul 2>&1
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PeripheralBattery /f >nul 2>&1
if exist "%LOCALAPPDATA%\VandoVolt\VandoVolt.exe" del "%LOCALAPPDATA%\VandoVolt\VandoVolt.exe"
echo VandoVolt removido. As configuracoes locais foram preservadas.
pause
