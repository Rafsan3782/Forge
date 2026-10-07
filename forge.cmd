@echo off
if exist "%~dp0.tools\cargo\bin\forge.exe" (
    "%~dp0.tools\cargo\bin\forge.exe" %*
) else if exist "%~dp0target\debug\forge.exe" (
    "%~dp0target\debug\forge.exe" %*
) else (
    echo [ERROR] forge binary not found. Please run '. .\tools\dev-env.ps1' and 'cargo build -p forge_cli' first.
)
