//! Module 3: DuckDB result sets as Polars DataFrames.
//!
//! Provable contract: `query_polars` yields one DataFrame per chunk; the summed
//! heights equal the SQL COUNT(*). Run: `cargo run -p duckdb-reports --features duckdb/polars --example polars_df`.
use duckdb::Connection;
fn main() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t AS SELECT range AS id FROM range(10000);")?;
    let mut stmt = conn.prepare("SELECT id FROM t")?;
    let frames: Vec<_> = stmt.query_polars([])?.collect();
    let rows: usize = frames.iter().map(|f| f.height()).sum();
    println!("polars rows={}", rows);
    assert_eq!(rows, 10_000);
    println!("contract: polars frame height equals COUNT(*) OK");
    Ok(())
}
