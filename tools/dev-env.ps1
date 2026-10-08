# tools/dev-env.ps1
# Dot-source this script in a PowerShell session to activate the Forge local development environment:
# . .\tools\dev-env.ps1

$scriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path
$toolsDir = Join-Path $repoRoot ".tools"

if (-not (Test-Path $toolsDir)) {
    Write-Error "Project-local tools folder not found at: $toolsDir"
    return
}

# Session-only environment variables
$env:CARGO_HOME = Join-Path $toolsDir "cargo"
$env:RUSTUP_HOME = Join-Path $toolsDir "rustup"
$env:COREPACK_HOME = Join-Path $toolsDir "corepack"
$env:npm_config_store_dir = Join-Path $toolsDir "pnpm-store"
$env:PNPM_HOME = Join-Path $toolsDir "pnpm-store"

$rcExe = Join-Path $toolsDir "cargo\bin\rc.exe"
$rcSrc = Join-Path $repoRoot "tools\rc-stub.rs"
if ((-not (Test-Path $rcExe)) -and (Test-Path $rcSrc)) {
    $splatDir = Join-Path $repoRoot ".xwin-cache\splat"
    $rustcExe = Join-Path $toolsDir "cargo\bin\rustc.exe"
    if ((Test-Path $splatDir) -and (Test-Path $rustcExe)) {
        & $rustcExe $rcSrc -O -C linker=rust-lld -C linker-flavor=lld-link `
            -L ("native=" + (Join-Path $splatDir "crt\lib\x64")) `
            -L ("native=" + (Join-Path $splatDir "sdk\lib\ucrt\x64")) `
            -L ("native=" + (Join-Path $splatDir "sdk\lib\um\x64")) `
            -o $rcExe 2>$null
    }
}
$env:RC = $rcExe

# Bin directories to prepend to PATH for this session only
$localBinDirs = @(
    (Join-Path $toolsDir "cargo\bin"),
    (Join-Path $toolsDir "npm-global"),
    (Join-Path $toolsDir "node"),
    (Join-Path $toolsDir "git\cmd"),
    (Join-Path $toolsDir "gh\bin")
)

$validBinDirs = @()
foreach ($dir in $localBinDirs) {
    if (Test-Path $dir) {
        $validBinDirs += (Resolve-Path $dir).Path
    }
}

# Prepend to PATH, ensuring project tools take precedence over system PATH
$currentPaths = ($env:PATH -split ';') | Where-Object { $_ -and ($_ -notin $validBinDirs) }
$env:PATH = ($validBinDirs + $currentPaths) -join ';'

Write-Host "Forge development environment loaded (.tools activated)." -ForegroundColor Green
