# tools/dev-env-check.ps1
# Verifies that development tools resolve inside the project-local .tools folder.

$scriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
$repoRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path
$toolsDir = Join-Path $repoRoot ".tools"

$toolsToCheck = @(
    @{ Name = "rustc"; Command = "rustc"; VersionArgs = @("--version") },
    @{ Name = "cargo"; Command = "cargo"; VersionArgs = @("--version") },
    @{ Name = "cargo-nextest"; Command = "cargo-nextest"; VersionArgs = @("-V") },
    @{ Name = "node"; Command = "node"; VersionArgs = @("-v") },
    @{ Name = "pnpm"; Command = "pnpm"; VersionArgs = @("-v") },
    @{ Name = "git"; Command = "git"; VersionArgs = @("--version") },
    @{ Name = "gh"; Command = "gh"; VersionArgs = @("--version") }
)

Write-Host "=== Forge Project-Local Tool Resolution Check ===" -ForegroundColor Cyan
Write-Host "Repository root : $repoRoot"
Write-Host "Tools directory : $toolsDir`n"

$results = @()
$hasFailure = $false

foreach ($t in $toolsToCheck) {
    $cmdInfo = Get-Command $t.Command -ErrorAction SilentlyContinue
    if (-not $cmdInfo) {
        $results += [PSCustomObject]@{
            Tool = $t.Name
            ResolvedPath = "NOT FOUND"
            Version = "N/A"
            Status = "FAIL (Not in PATH)"
        }
        $hasFailure = $true
        continue
    }

    $path = $cmdInfo.Source
    $versionOutput = ""
    try {
        $rawVersion = & $t.Command $t.VersionArgs 2>&1
        $versionOutput = ($rawVersion | Select-Object -First 1).ToString().Trim()
    } catch {
        $versionOutput = "Error querying version"
    }

    $isInsideForge = $path.ToLower().StartsWith($repoRoot.ToLower())
    $status = if ($isInsideForge) { "PASS (Local)" } else { "FAIL (Outside repo)" }
    if (-not $isInsideForge) {
        $hasFailure = $true
    }

    $results += [PSCustomObject]@{
        Tool = $t.Name
        ResolvedPath = $path
        Version = $versionOutput
        Status = $status
    }
}

$results | Format-Table -AutoSize -Property Tool, Status, Version, ResolvedPath

Write-Host "=== System Dependencies (Host-Level Components) ===" -ForegroundColor Cyan
$systemComponents = @(
    [PSCustomObject]@{
        Component = "Windows SDK / CRT"
        Location = (Join-Path $repoRoot ".xwin-cache\splat")
        Status = "Local CRT & SDK via rust-lld"
    },
    [PSCustomObject]@{
        Component = "WebView2 Runtime"
        Location = "C:\Program Files (x86)\Microsoft\EdgeWebView\Application\*\msedgewebview2.exe"
        Status = "System-managed runtime"
    },
    [PSCustomObject]@{
        Component = "GPU Display Driver"
        Location = "OS Kernel / DirectX 12 / Vulkan"
        Status = "System driver"
    }
)
$systemComponents | Format-Table -AutoSize

if ($hasFailure) {
    Write-Host "`n[FAIL] One or more tools resolve outside the project root or are missing." -ForegroundColor Red
    exit 1
} else {
    Write-Host "`n[PASS] All project tools resolve strictly inside the forge folder." -ForegroundColor Green
    exit 0
}
