@echo off
setlocal
cd /d "%~dp0"
set "SRC=target\x86_64-pc-windows-msvc\release\peripheral-battery.exe"
set "DEST=%LOCALAPPDATA%\PeripheralBattery"
if not exist "%SRC%" (
 echo Compile primeiro usando compilar.cmd.
 pause
 exit /b 1
)
if not exist "%DEST%" mkdir "%DEST%"
copy /y "%SRC%" "%DEST%\peripheral-battery.exe" >nul
if errorlevel 1 goto fail
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PeripheralBattery /t REG_SZ /d "\"%DEST%\peripheral-battery.exe\"" /f
if errorlevel 1 goto fail
start "" "%DEST%\peripheral-battery.exe"
echo Instalado para seu usuario e configurado para iniciar com o Windows.
pause
exit /b 0
:fail
echo Falha na instalacao. Feche o aplicativo caso esteja aberto e tente novamente.
pause
exit /b 1
