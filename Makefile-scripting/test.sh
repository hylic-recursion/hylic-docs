#!/usr/bin/env bash
set -euo pipefail
cargo test --lib ${HYLIC_DOCS_TEST_FILTER:+-- --nocapture $HYLIC_DOCS_TEST_FILTER}
