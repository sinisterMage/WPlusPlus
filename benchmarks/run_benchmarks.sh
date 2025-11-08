#!/bin/bash
# Automated benchmark runner for Raython vs ASP.NET Core vs Phoenix
# Requires: wrk, servers running on ports 3000, 5000, 4000

set -e

DURATION="30s"
THREADS=4
CONNECTIONS=100
RESULTS_FILE="BENCHMARK_RESULTS.txt"

echo "========================================" > $RESULTS_FILE
echo "Web Framework Benchmark Results" >> $RESULTS_FILE
echo "========================================" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "Test Configuration:" >> $RESULTS_FILE
echo "  Duration: $DURATION" >> $RESULTS_FILE
echo "  Threads: $THREADS" >> $RESULTS_FILE
echo "  Connections: $CONNECTIONS" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "System Info:" >> $RESULTS_FILE
sysctl -n machdep.cpu.brand_string >> $RESULTS_FILE 2>/dev/null || lscpu | grep "Model name" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
date >> $RESULTS_FILE
echo "" >> $RESULTS_FILE
echo "========================================" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE

run_benchmark() {
    local name=$1
    local url=$2

    echo ""
    echo "Testing: $name - $url"
    echo "----------------------------------------"
    echo "" >> $RESULTS_FILE
    echo "### $name - $url" >> $RESULTS_FILE
    echo "" >> $RESULTS_FILE

    wrk -t$THREADS -c$CONNECTIONS -d$DURATION "$url" | tee -a $RESULTS_FILE

    echo "" >> $RESULTS_FILE
    sleep 2
}

echo "Starting benchmarks..."
echo ""

# Verify servers are running
echo "Checking if servers are running..."
curl -s http://localhost:3000/ > /dev/null && echo "✓ Raython server (port 3000) is up" || echo "✗ Raython server NOT running"
curl -s http://localhost:5000/ > /dev/null && echo "✓ ASP.NET server (port 5000) is up" || echo "✗ ASP.NET server NOT running"
curl -s http://localhost:4000/ > /dev/null && echo "✓ Phoenix server (port 4000) is up" || echo "✗ Phoenix server NOT running"
echo ""

read -p "Press Enter to start benchmarks (or Ctrl+C to cancel)..."

# Root endpoint benchmarks
echo ""
echo "================================================"
echo "Benchmarking: GET / (Root Endpoint)"
echo "================================================"

run_benchmark "Raython - GET /" "http://localhost:3000/"
run_benchmark "ASP.NET - GET /" "http://localhost:5000/"
run_benchmark "Phoenix - GET /" "http://localhost:4000/"

# API Posts list endpoint
echo ""
echo "================================================"
echo "Benchmarking: GET /api/posts (JSON List)"
echo "================================================"

run_benchmark "Raython - GET /api/posts" "http://localhost:3000/api/posts"
run_benchmark "ASP.NET - GET /api/posts" "http://localhost:5000/api/posts"
run_benchmark "Phoenix - GET /api/posts" "http://localhost:4000/api/posts"

# API Single post endpoint
echo ""
echo "================================================"
echo "Benchmarking: GET /api/posts/1 (JSON Object)"
echo "================================================"

run_benchmark "Raython - GET /api/posts/1" "http://localhost:3000/api/posts/1"
run_benchmark "ASP.NET - GET /api/posts/1" "http://localhost:5000/api/posts/1"
run_benchmark "Phoenix - GET /api/posts/1" "http://localhost:4000/api/posts/1"

echo ""
echo "========================================" >> $RESULTS_FILE
echo "Benchmarks Complete!" >> $RESULTS_FILE
echo "========================================" >> $RESULTS_FILE

echo ""
echo "Benchmarks complete! Results saved to: $RESULTS_FILE"
echo ""
