.PHONY: help demo capstone verify clean fetch test fmt lint coverage pmat \
        demo-1-tour demo-2-pagila demo-3-tpch demo-4-rust demo-all

help:
	@echo "DuckDB From Zero — companion repo"
	@echo ""
	@echo "  make fetch         — download fixtures (TPC-H, Pagila Parquets, NYC Taxi)"
	@echo "  make demo          — run the three lab examples against the fetched data"
	@echo "  make capstone      — build and run the Rust duckdb-reports binary"
	@echo "  make verify        — assert all headline counts (CI smoke test)"
	@echo ""
	@echo "  Standalone demos (one script per concept):"
	@echo "    make demo-1-tour    — DuckDB feature tour (3 lab SQL files)"
	@echo "    make demo-2-pagila  — 3 raw Pagila analytics queries"
	@echo "    make demo-3-tpch    — all 22 TPC-H benchmark queries, timed"
	@echo "    make demo-4-rust    — Rust capstone binary, 3 contract-enforced reports"
	@echo "    make demo-all       — run every demo in sequence"
	@echo ""
	@echo "  make test          — cargo test for the Rust crate"
	@echo "  make coverage      — cargo llvm-cov line coverage report"
	@echo "  make pmat          — pmat quality-gate (entropy excluded — small-repo metric artifact)"
	@echo "  make fmt lint      — cargo fmt && cargo clippy"
	@echo "  make clean         — wipe data/ and target/"

fetch:
	@bash scripts/fetch-data.sh

demo: fetch
	@bash scripts/tour.sh

capstone: fetch
	@cargo run --release --bin duckdb-reports -- --report customers --limit 5
	@cargo run --release --bin duckdb-reports -- --report films     --limit 5
	@cargo run --release --bin duckdb-reports -- --report actors    --limit 5

verify: fetch
	@bash scripts/verify.sh

demo-1-tour:
	@bash scripts/demo-1-tour.sh

demo-2-pagila:
	@bash scripts/demo-2-pagila.sh

demo-3-tpch:
	@bash scripts/demo-3-tpch.sh

demo-4-rust:
	@bash scripts/demo-4-rust.sh

demo-all:
	@bash scripts/demo-all.sh

test:
	@cargo test --release

fmt:
	@cargo fmt --all

lint:
	@cargo clippy --all-targets -- -D warnings

coverage:
	@cargo llvm-cov --release --workspace --show-missing-lines

pmat:
	@pmat quality-gate --checks dead-code,complexity,coverage,sections,satd,security,duplicates,provability

clean:
	@rm -rf data/ target/
