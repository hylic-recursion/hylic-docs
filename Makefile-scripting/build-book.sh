#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo insta test --accept --lib
cd book && mdbook build
echo "Book: $(cd .. && pwd)/target/book/index.html"
