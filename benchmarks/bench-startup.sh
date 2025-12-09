#!/bin/bash

# Benchmark script for comparing startup time
# Measures time to start runtime and execute empty script

ITERATIONS=100
SCRIPT="benchmarks/empty.js"

echo "Benchmarking: Startup time (empty script)"
echo "Iterations: $ITERATIONS"
echo ""

# Function to run benchmark
run_bench() {
    local runtime=$1
    local command=$2

    echo "Testing $runtime..."

    # Warmup
    for i in {1..5}; do
        $command > /dev/null 2>&1
    done

    # Actual benchmark
    local total=0
    for i in $(seq 1 $ITERATIONS); do
        local start=$(date +%s%N)
        $command > /dev/null 2>&1
        local end=$(date +%s%N)
        local duration=$((end - start))
        total=$((total + duration))
    done

    local avg=$((total / ITERATIONS))
    local avg_ms=$(awk "BEGIN {printf \"%.2f\", $avg / 1000000}")

    echo "$runtime: ${avg_ms}ms (average)"
    echo ""
}

# Build mdeno if needed
if [ ! -f "target/release/mdeno" ]; then
    echo "Building mdeno..."
    cargo build --release
    echo ""
fi

# Run benchmarks
run_bench "Deno" "deno run $SCRIPT"
run_bench "Node.js" "node $SCRIPT"
run_bench "Bun" "bun run $SCRIPT"
run_bench "mDeno" "./target/release/mdeno $SCRIPT"

echo "Benchmark complete!"
