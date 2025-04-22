@echo off
setlocal ENABLEEXTENSIONS

REM === Parse arguments ===
set MODE=debug
set CARGO_FLAG=
if "%1"=="--release" (
    set MODE=release
    set CARGO_FLAG=--release
)
set TARGET=target\%MODE%

echo [BUILD] Mode set to %MODE%
echo [BUILD] Output path: %TARGET%

echo Building projects...
cargo build --manifest-path server\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build!
    exit /b 1
)

echo Copying web project to target/debug/web...
xcopy /E /I /Y "server\web" "%TARGET%\web"
if errorlevel 1 (
    echo Failed to copy plugin_status web folder.
    exit /b 1
)