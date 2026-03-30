@echo off
REM Build script for cross-platform compilation on Windows

echo Building Aporia Loader for all platforms...

REM Create builds directory
if not exist builds mkdir builds

REM Windows x64 (native)
echo Building for Windows x64...
cargo build --release
if %errorlevel% neq 0 (
    echo Failed to build for Windows x64
    pause
    exit /b 1
)
copy target\release\aporia-loader.exe builds\aporia-loader-windows-x64.exe

REM Linux x64
echo.
echo Building for Linux x64...
echo Make sure Docker Desktop is running...
cross build --release --target x86_64-unknown-linux-gnu
if %errorlevel% equ 0 (
    copy target\x86_64-unknown-linux-gnu\release\aporia-loader builds\aporia-loader-linux-x64
    echo Linux x64 build successful!
) else (
    echo Warning: Linux x64 build failed. Make sure Docker Desktop is running.
)

echo.
echo Build complete! Binaries are in builds\ directory
echo.
echo Note: macOS builds require macOS host or GitHub Actions
echo Use GitHub Actions workflow for full cross-platform builds
pause
