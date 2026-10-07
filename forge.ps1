param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs
)

$scriptDir = $PSScriptRoot
$toolsExe = Join-Path $scriptDir ".tools\cargo\bin\forge.exe"
$targetExe = Join-Path $scriptDir "target\debug\forge.exe"

if (Test-Path $toolsExe) {
    & $toolsExe @RemainingArgs
} elseif (Test-Path $targetExe) {
    & $targetExe @RemainingArgs
} else {
    Write-Error "forge binary not found. Please activate local environment with '. .\tools\dev-env.ps1' and build forge_cli."
}
