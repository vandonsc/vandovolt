@echo off
setlocal
cd /d "%~dp0"
set "SRC=target\x86_64-pc-windows-msvc\release\VandoVolt.exe"
set "DEST=%LOCALAPPDATA%\VandoVolt"
set "LEGACY=%LOCALAPPDATA%\PeripheralBattery"
if not exist "%SRC%" (
 echo Compile primeiro usando compilar.cmd.
 pause
 exit /b 1
)
if not exist "%DEST%" mkdir "%DEST%"
if errorlevel 1 goto fail
if not exist "%DEST%\config.ini" if exist "%LEGACY%\config.ini" (
 copy /y "%LEGACY%\config.ini" "%DEST%\config.ini.migrating" >nul
 if errorlevel 1 goto fail
 fc /b "%LEGACY%\config.ini" "%DEST%\config.ini.migrating" >nul
 if errorlevel 1 goto fail
 move /y "%DEST%\config.ini.migrating" "%DEST%\config.ini" >nul
 if errorlevel 1 goto fail
)
copy /y "%SRC%" "%DEST%\VandoVolt.exe" >nul
if errorlevel 1 goto fail
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v VandoVolt /t REG_SZ /d "\"%DEST%\VandoVolt.exe\"" /f
if errorlevel 1 goto fail
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PeripheralBattery /f >nul 2>&1
start "" "%DEST%\VandoVolt.exe"
echo VandoVolt instalado para seu usuario e configurado para iniciar com o Windows.
pause
exit /b 0
:fail
if exist "%DEST%\config.ini.migrating" del "%DEST%\config.ini.migrating"
echo Falha na instalacao. As configuracoes antigas foram preservadas.
pause
exit /b 1
