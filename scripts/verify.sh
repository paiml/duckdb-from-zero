#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

assert_count() {
    local label="$1" expected="$2" actual="$3"
    if [ "$actual" = "$expected" ]; then
        echo "[verify] $label = $actual ✓"
    else
        echo "[verify] $label expected $expected, got $actual ✗"
        exit 1
    fi
}

LINEITEM=$(duckdb data/tpch.duckdb -noheader -list -c "SELECT COUNT(*) FROM lineitem;")
assert_count "TPC-H lineitem" "6001215" "$LINEITEM"

FILMS=$(duckdb -noheader -list -c "SELECT COUNT(*) FROM 'data/pagila/film.parquet';")
assert_count "Pagila film"     "1000"    "$FILMS"

ACTORS=$(duckdb -noheader -list -c "SELECT COUNT(*) FROM 'data/pagila/actor.parquet';")
assert_count "Pagila actor"    "200"     "$ACTORS"

echo "[verify] all assertions passed"
