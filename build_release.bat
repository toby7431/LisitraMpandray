@echo off
echo Setting up MSVC environment...
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64
set PATH=%PATH%;C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64
cd /d D:\lisitra_mpandray\EgliseManager

echo Cleaning dist...
rmdir /s /q dist 2>nul
ping -n 5 127.0.0.1 >nul
mkdir dist

echo Building frontend RELEASE explicitement...
trunk build --release
if errorlevel 1 (
    echo ERROR: trunk build failed
    exit /b 1
)

echo Verification que c'est bien un build release...
findstr /c:"__TRUNK_ADDRESS__" dist\index.html >nul 2>&1
if not errorlevel 1 (
    echo ERROR: index.html contient du code dev - build incorrect
    exit /b 1
)
echo Frontend OK - pas de code dev detecte

echo Building Tauri release...
cargo tauri build
echo Done. Check target\release\ for the .exe
