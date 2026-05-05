#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== 01-tour.sql ==="
duckdb data/tpch.duckdb -f sql/01-tour.sql

echo ""
echo "=== 02-files-as-tables.sql ==="
duckdb -f sql/02-files-as-tables.sql

echo ""
echo "=== 03-tpch-q1.sql ==="
time duckdb data/tpch.duckdb -f sql/03-tpch-q1.sql
