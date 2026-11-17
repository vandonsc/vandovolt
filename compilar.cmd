@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>&1
if errorlevel 1 (
 echo Instale Rust em https://rustup.rs/ e as ferramentas C++ do Visual Studio.
 echo Depois abra este arquivo novamente.
 pause
 exit /b 1
)
cargo test --target x86_64-pc-windows-msvc
if errorlevel 1 goto fail
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 goto fail
echo Executavel: target\x86_64-pc-windows-msvc\release\VandoVolt.exe
pause
exit /b 0
:fail
echo Falha na compilacao. Copie as mensagens acima para diagnostico.
pause
exit /b 1
