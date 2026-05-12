#!/usr/bin/env bash
# demo-1-tour.sh — DuckDB feature tour.
#
# Three short SQL files that build intuition for what DuckDB is and what's
# different about it. Mirrors the three lab examples in the Coursera videos.
#
#   01-tour.sql           DuckDB-the-database: .timer, duckdb_tables(), TPC-H
#   02-files-as-tables    Parquet files ARE tables — no CREATE TABLE
#   03-tpch-q1.sql        TPC-H Q1 against ~6M lineitem rows in under a second
#
# Usage:
#   scripts/demo-1-tour.sh
#
set -euo pipefail
cd "$(dirname "$0")/.."

bar() { printf '\n\033[1;36m=== %s ===\033[0m\n\n' "$*"; }

[ -f data/tpch.duckdb ] && [ -f data/pagila/film.parquet ] || {
    bar "fetching fixtures (one-time, ~60–120 s)"
    bash scripts/fetch-data.sh
}

bar "01-tour.sql — DuckDB-the-database (TPC-H sf=1, 6M lineitem rows)"
duckdb data/tpch.duckdb -f sql/01-tour.sql

bar "02-files-as-tables.sql — Parquet files ARE tables"
duckdb -f sql/02-files-as-tables.sql

bar "03-tpch-q1.sql — TPC-H Q1 over 6M rows"
duckdb data/tpch.duckdb -f sql/03-tpch-q1.sql

bar "tour complete"
