@echo off
setlocal ENABLEEXTENSIONS

REM === Parse arguments ===
REM Default build mode is debug
set MODE=debug
set CARGO_FLAG=
REM Check if the first argument is "--release" and set mode accordingly
if "%1"=="--release" (
    set MODE=release
    set CARGO_FLAG=--release
)
REM Set the target directory based on the build mode
set TARGET=target\%MODE%

REM Display the build mode and output path
echo [BUILD] Mode set to %MODE%
echo [BUILD] Output path: %TARGET%

REM Build the Rust project using Cargo
echo Building projects...
cargo build --manifest-path server\Cargo.toml %CARGO_FLAG%
REM Check if the build failed and exit with an error code if it did
if errorlevel 1 (
    echo Failed to build!
    exit /b 1
)

REM Copy the web project files to the target directory
echo Copying web project to target/debug/web...
xcopy /E /I /Y "server\web" "%TARGET%\web"
REM Check if the copy operation failed and exit with an error code if it did
if errorlevel 1 (
    echo Failed to copy plugin_status web folder.
    exit /b 1
)