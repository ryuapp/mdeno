# Benchmark script for comparing startup time (Windows)
# Measures time to start runtime and execute empty script

$ITERATIONS = 100
$SCRIPT = "benchmarks/empty.js"

Write-Host "Benchmarking: Startup time (empty script)"
Write-Host "Iterations: $ITERATIONS"
Write-Host ""

function Run-Bench {
    param(
        [string]$Runtime,
        [string]$Command
    )

    Write-Host "Testing $Runtime..."

    # Warmup
    for ($i = 0; $i -lt 5; $i++) {
        Invoke-Expression "$Command" > $null 2>&1
    }

    # Actual benchmark
    $times = @()
    for ($i = 0; $i -lt $ITERATIONS; $i++) {
        $start = Get-Date
        Invoke-Expression "$Command" > $null 2>&1
        $end = Get-Date
        $duration = ($end - $start).TotalMilliseconds
        $times += $duration
    }

    $avg = ($times | Measure-Object -Average).Average
    Write-Host "$Runtime`: $([math]::Round($avg, 2))ms (average)"
    Write-Host ""
}

# Build mdeno if needed
if (-not (Test-Path "target/release/mdeno.exe")) {
    Write-Host "Building mdeno..."
    cargo build --release
    Write-Host ""
}

# Run benchmarks
Run-Bench "Deno" "deno run $SCRIPT"
Run-Bench "Node.js" "node $SCRIPT"
Run-Bench "Bun" "bun run $SCRIPT"
Run-Bench "mDeno" "target/release/mdeno.exe $SCRIPT"

Write-Host "Benchmark complete!"
