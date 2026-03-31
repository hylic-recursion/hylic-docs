#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Optionally run benchmarks first
if [ "${HYLIC_BENCH:-}" = "1" ]; then
    echo "Running benchmarks..."
    (cd ../hylic && make bench && make bench-report)
fi

# Copy latest bench results into book source (if they exist)
BENCH_DIR="../hylic/target/bench-report"
BOOK_BENCH="book/src/bench-results"
if [ -d "$BENCH_DIR" ]; then
    mkdir -p "$BOOK_BENCH"
    cp -f "$BENCH_DIR/bench-table.txt" "$BOOK_BENCH/" 2>/dev/null || true
    cp -f "$BENCH_DIR/bench-chart.svg" "$BOOK_BENCH/" 2>/dev/null || true
    cp -f "$BENCH_DIR/bench.csv" "$BOOK_BENCH/" 2>/dev/null || true
fi

cargo insta test --accept --lib
cd book && mdbook build
echo "Book: $(cd .. && pwd)/target/book/index.html"
