.PHONY: help demo capstone verify clean fetch test fmt lint coverage pmat

help:
	@echo "DuckDB From Zero — companion repo"
	@echo ""
	@echo "  make fetch     — download fixtures (TPC-H, Pagila Parquets, NYC Taxi)"
	@echo "  make demo      — run the three lab examples against the fetched data"
	@echo "  make capstone  — build and run the Rust duckdb-reports binary"
	@echo "  make verify    — assert all headline counts (CI smoke test)"
	@echo "  make test      — cargo test for the Rust crate"
	@echo "  make coverage  — cargo llvm-cov line coverage report"
	@echo "  make pmat      — pmat quality-gate (entropy excluded — small-repo metric artifact)"
	@echo "  make fmt lint  — cargo fmt && cargo clippy"
	@echo "  make clean     — wipe data/ and target/"

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
