param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$Settings
)

# Navigate to theorem_prover directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
$theoremProverDir = Join-Path $projectRoot "theorem_prover"

if (-not (Test-Path $theoremProverDir)) {
    Write-Host "Error: theorem_prover directory not found at $theoremProverDir" -ForegroundColor Red
    exit 1
}

Push-Location $theoremProverDir

$engines = @("naive", "id", "priority-id")

foreach ($engine in $engines) {
    Write-Host "Running prove with engine: $engine" -ForegroundColor Green
    Write-Host "Command: cargo run prove $($Settings -join ' ') --engine $engine" -ForegroundColor Gray

    if ($Settings) {
        & cargo run prove @Settings --engine $engine
    } else {
        & cargo run prove --engine $engine
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Engine $engine failed with exit code $LASTEXITCODE" -ForegroundColor Red
        Pop-Location
        exit $LASTEXITCODE
    }

    Write-Host ""
}

Pop-Location
Write-Host "All engines completed successfully" -ForegroundColor Green
