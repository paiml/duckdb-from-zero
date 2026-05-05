# duckdb-reports

Three Sakila/Pagila analytics reports built on DuckDB's Parquet support.
The crate's public API is in `src/lib.rs`; the `main.rs` is a thin clap wrapper.

## Build & run

From repo root:

    make fetch        # populate ../../data/pagila/*.parquet
    cargo run --release -- --report customers --limit 5
    cargo run --release -- --report films     --limit 5
    cargo run --release -- --report actors    --limit 5

## Test

    cargo test --release    # requires data/pagila/ to be populated
