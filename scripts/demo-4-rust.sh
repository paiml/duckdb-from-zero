#!/usr/bin/env bash
# demo-4-rust.sh — Rust capstone: duckdb-reports binary, three reports.
#
# Same SQL as scripts/demo-2-pagila.sh, but wrapped by a Rust binary that
# enforces five named runtime contracts (C1–C5) on every result:
#
#   C1 row_count_exact          rows.len() == limit
#   C2 top_record_has_count     rows[0].count >= 1
#   C3 id_positive              all rows have id > 0
#   C4 string_field_well_formed text fields satisfy per-report invariant
#   C5 count_descending         count column is monotonically non-increasing
#
# Spec: contracts/duckdb-rust-v1.yaml.
# Output: pretty-printed JSON to stdout AND to out/ as files.
#
# Usage:
#   scripts/demo-4-rust.sh             # default --limit 10
#   scripts/demo-4-rust.sh 5           # --limit 5
#
set -euo pipefail
cd "$(dirname "$0")/.."

LIMIT="${1:-10}"
OUT_DIR="out"

bar() { printf '\n\033[1;36m=== %s ===\033[0m\n\n' "$*"; }

[ -f data/pagila/film.parquet ] || {
    bar "fetching Pagila fixtures (one-time)"
    bash scripts/fetch-data.sh
}

bar "building duckdb-reports (release)"
cargo build --release --bin duckdb-reports

mkdir -p "$OUT_DIR"

for r in customers films actors; do
    bar "duckdb-reports --report ${r} --limit ${LIMIT}"
    cargo run --release --quiet --bin duckdb-reports -- \
        --report "$r" --limit "$LIMIT" --out "${OUT_DIR}/${r}.json"
    echo
    echo "wrote ${OUT_DIR}/${r}.json"
done

bar "capstone complete — JSON written to ${OUT_DIR}/"
ls -lh "$OUT_DIR"/*.json
