# DuckDB From Zero — Companion Repo

The runnable companion to the Coursera course **DuckDB From Zero**, part of the
*Rust for Data Engineering* specialization.

This repo bundles every example shown in the videos plus the optional Rust
capstone artifact (`duckdb-reports`). The Coursera lab boots a sandbox with the
same fixtures pre-loaded — but you can also run everything locally.

## Quick start

    git clone https://github.com/paiml/duckdb-from-zero
    cd duckdb-from-zero
    make demo         # downloads fixtures, runs the three SQL tour scripts
    make capstone     # builds and runs the Rust binary against Pagila Parquets

The first `make` call takes 60–120 seconds (TPC-H sf=1 generation + Pagila
download + NYC Taxi download). Subsequent calls are instant.

## What's here

- `sql/` — the SQL the videos walk through, plus all 22 TPC-H benchmark queries
- `crates/duckdb-reports/` — Rust binary that runs three Sakila analytics reports
  with runtime contracts; the foundation for the course capstone
- `scripts/` — the bash glue for fetch, demo, and verify

## Prerequisites

- DuckDB CLI 1.1+ (`curl https://install.duckdb.org/ | sh`)
- Rust 1.95+ (use rustup; `rust-toolchain.toml` will pin automatically)
- ~500 MB free disk for fixtures

## Why DuckDB

Three things you can do here that you cannot easily do with a server-based
database:

1. `SELECT * FROM 'data/pagila/film.parquet'` — the Parquet file is the table.
   No `CREATE TABLE`. No import step. (See `sql/02-files-as-tables.sql`.)
2. Query 6 million rows of TPC-H `lineitem` and have the answer in well under
   a second on a laptop. (See `sql/03-tpch-q1.sql`.)
3. Embed the entire database engine inside a Rust binary that ships as a
   single file with no daemon and no network port. (See `crates/duckdb-reports/`.)

## Capstone

The capstone for this course extends `crates/duckdb-reports/` from three reports
to a configurable analytics tool. See the course's capstone reading for the
detailed brief.
