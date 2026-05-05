#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

DATA=data
mkdir -p "$DATA/pagila" "$DATA/nyc-taxi"

# 1. TPC-H sf=1
if [ ! -f "$DATA/tpch.duckdb" ]; then
    echo "[fetch] generating TPC-H sf=1 (30–90s)…"
    duckdb "$DATA/tpch.duckdb" -c "INSTALL tpch; LOAD tpch; CALL dbgen(sf=1);"
fi
COUNT=$(duckdb "$DATA/tpch.duckdb" -noheader -list -c "SELECT COUNT(*) FROM lineitem;")
[ "$COUNT" = "6001215" ] || (echo "TPC-H lineitem count wrong: $COUNT" && exit 1)
echo "[fetch] TPC-H lineitem count = 6001215 ✓"

# 2. Pagila as Parquet
#
# Pagila ships as a Postgres pg_dump that DuckDB can't parse directly (SET
# statement_timeout, schema-qualified names, multi-line CREATE TYPE blocks).
# We sidestep the schema entirely: extract each `COPY public.<table> ... FROM
# stdin;` block out of pagila-data.sql into a TSV, then let DuckDB's
# read_csv_auto infer types when writing each Parquet file.
if [ ! -f "$DATA/pagila/film.parquet" ]; then
    echo "[fetch] downloading Pagila…"
    TMPDIR=$(mktemp -d)
    mkdir -p "$TMPDIR/tsv"
    curl -sL https://raw.githubusercontent.com/devrimgunduz/pagila/master/pagila-data.sql -o "$TMPDIR/data.sql"

    # awk: split each COPY block into its own TSV (header = column list, rows = tab-separated data, NULL = \N)
    awk -v outdir="$TMPDIR/tsv" '
        /^COPY public\.[a-z_0-9]+ \(.*\) FROM stdin;$/ {
            tbl = $0
            sub(/^COPY public\./, "", tbl)
            sub(/ \(.*$/, "", tbl)
            cols = $0
            sub(/^COPY public\.[a-z_0-9]+ \(/, "", cols)
            sub(/\) FROM stdin;$/, "", cols)
            gsub(/, /, "\t", cols)
            cur = outdir "/" tbl ".tsv"
            print cols > cur
            in_copy = 1
            next
        }
        in_copy && $0 == "\\." {
            close(cur)
            in_copy = 0
            next
        }
        in_copy {
            print > cur
        }
    ' "$TMPDIR/data.sql"

    # Sanity: actor.tsv exists and has 200 data rows.
    ACTOR_ROWS=$(($(wc -l < "$TMPDIR/tsv/actor.tsv") - 1))
    [ "$ACTOR_ROWS" = "200" ] || (echo "Pagila actor TSV row count wrong: $ACTOR_ROWS" && exit 1)

    # One Parquet per non-partitioned table.
    for tbl in film customer rental inventory actor film_actor category film_category language staff store address city country; do
        duckdb -c "COPY (SELECT * FROM read_csv_auto('$TMPDIR/tsv/$tbl.tsv', delim='\t', header=true, nullstr='\N')) TO '$DATA/pagila/$tbl.parquet' (FORMAT PARQUET);"
    done

    # payment is range-partitioned in Pagila (payment_p2022_01..12). Union them.
    duckdb -c "COPY (SELECT * FROM read_csv_auto('$TMPDIR/tsv/payment_p*.tsv', delim='\t', header=true, nullstr='\N', union_by_name=true)) TO '$DATA/pagila/payment.parquet' (FORMAT PARQUET);"

    rm -rf "$TMPDIR"
fi
PFILMS=$(duckdb -noheader -list -c "SELECT COUNT(*) FROM '$DATA/pagila/film.parquet';")
[ "$PFILMS" = "1000" ] || (echo "Pagila film count wrong: $PFILMS" && exit 1)
echo "[fetch] Pagila film count = 1000 ✓"

# 3. NYC Taxi
if [ ! -f "$DATA/nyc-taxi/yellow_tripdata_2024-01.parquet" ]; then
    echo "[fetch] downloading NYC Taxi 2024-01…"
    curl -fsSL "https://d37ci6vzurychx.cloudfront.net/trip-data/yellow_tripdata_2024-01.parquet" \
        -o "$DATA/nyc-taxi/yellow_tripdata_2024-01.parquet"
fi
NYC=$(duckdb -noheader -list -c "SELECT COUNT(*) FROM '$DATA/nyc-taxi/yellow_tripdata_2024-01.parquet';")
echo "[fetch] NYC Taxi 2024-01 row count = $NYC ✓"

echo "[fetch] All fixtures ready in $DATA/"
