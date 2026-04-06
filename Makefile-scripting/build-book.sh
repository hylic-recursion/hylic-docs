#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Copy latest bench results into book source (if they exist)
BENCH_DIR="../hylic/target/bench-report"
BOOK_BENCH="book/src/bench-results"
if [ -d "$BENCH_DIR" ]; then
    mkdir -p "$BOOK_BENCH"
    cp -f "$BENCH_DIR"/*.html "$BOOK_BENCH/" 2>/dev/null || true
    cp -f "$BENCH_DIR"/*.txt "$BOOK_BENCH/" 2>/dev/null || true
    cp -f "$BENCH_DIR"/*.csv "$BOOK_BENCH/" 2>/dev/null || true
    cp -f "$BENCH_DIR"/*.css "$BOOK_BENCH/" 2>/dev/null || true
fi

cargo insta test --accept --lib
cd book && mdbook build
echo "Book: $(cd .. && pwd)/target/book/index.html"
