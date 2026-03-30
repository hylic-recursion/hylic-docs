#!/usr/bin/env bash
set -euo pipefail
# Accept new snapshots during dev; CI should run without --accept to catch regressions.
cargo insta test --accept --lib ${HYLIC_DOCS_TEST_FILTER:+-- $HYLIC_DOCS_TEST_FILTER}
