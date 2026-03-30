#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Running cookbook tests + updating snapshots..."
cargo insta test --accept --lib 2>&1 | tail -5

echo "Building mdBook..."
cd book && mdbook build
echo "Book built at: $(cd .. && pwd)/target/book/index.html"
