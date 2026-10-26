# PowerShell tool to generate procedural assets for the platformer template
param(
    [string]$OutputDir = "$PSScriptRoot\..\templates\platformer\assets"
)

$targetDir = [System.IO.Path]::GetFullPath($OutputDir)
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

Write-Host "Generating platformer assets in: $targetDir"

# Run forge_runtime procedural asset generation via cargo or headless mode
& cargo run -p forge_runtime -- --scene templates/platformer/scene.json --headless --frames 1
Write-Host "Platformer procedural assets generated successfully."
