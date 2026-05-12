#!/usr/bin/env bash
# demo-all.sh — run every demo in sequence.
#
#   1. SQL feature tour       (scripts/demo-1-tour.sh)
#   2. Pagila analytics       (scripts/demo-2-pagila.sh)
#   3. TPC-H 22-query suite   (scripts/demo-3-tpch.sh)
#   4. Rust capstone          (scripts/demo-4-rust.sh)
#
# Usage:
#   scripts/demo-all.sh
#
set -euo pipefail
cd "$(dirname "$0")/.."

bar() { printf '\n\033[1;33m################ %s ################\033[0m\n' "$*"; }

bar "DEMO 1/4 — SQL feature tour"
scripts/demo-1-tour.sh

bar "DEMO 2/4 — Pagila analytics"
scripts/demo-2-pagila.sh

bar "DEMO 3/4 — TPC-H benchmark (22 queries)"
scripts/demo-3-tpch.sh

bar "DEMO 4/4 — Rust capstone"
scripts/demo-4-rust.sh

bar "ALL DEMOS COMPLETE"
