@echo off
echo Setting up MSVC environment...
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64
if errorlevel 1 (
    echo ERROR: vcvarsall.bat failed
    exit /b 1
)
echo MSVC environment ready.
echo Checking link.exe...
where link.exe
if errorlevel 1 (
    echo ERROR: link.exe not found
    exit /b 1
)
set PATH=%PATH%;C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64
cd /d D:\lisitra_mpandray\EgliseManager
echo Starting cargo tauri dev...
cargo tauri dev
