#!/usr/bin/env bash
# demo-3-tpch.sh — all 22 official TPC-H queries against sf=1 (~6M rows).
#
# Each query is the canonical TPC-H benchmark query from the spec (sql/tpch/qNN.sql).
# DuckDB has the parameter values pre-substituted so every query is runnable
# verbatim. The .timer pragma prints wall-clock per query.
#
# Usage:
#   scripts/demo-3-tpch.sh             # run all 22
#   scripts/demo-3-tpch.sh 1 6 9       # run only Q1, Q6, Q9
#
set -euo pipefail
cd "$(dirname "$0")/.."

bar() { printf '\n\033[1;36m=== %s ===\033[0m\n\n' "$*"; }

[ -f data/tpch.duckdb ] || {
    bar "generating TPC-H sf=1 (one-time, ~30–90 s)"
    bash scripts/fetch-data.sh
}

if [ $# -gt 0 ]; then
    QUERIES=("$@")
else
    QUERIES=(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22)
fi

for n in "${QUERIES[@]}"; do
    q=$(printf 'q%02d' "$n")
    bar "TPC-H ${q^^}"
    duckdb data/tpch.duckdb -cmd '.timer on' -f "sql/tpch/${q}.sql"
done

bar "TPC-H benchmark complete (${#QUERIES[@]} queries)"
