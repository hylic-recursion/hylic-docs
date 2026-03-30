#!/usr/bin/env bash
set -euo pipefail
# Strict: snapshot mismatches are failures. No auto-accept.
cargo insta test --lib
