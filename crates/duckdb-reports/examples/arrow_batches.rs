//! Module 3: DuckDB result sets as Arrow record batches.
//!
//! Provable contract: `query_arrow` yields columnar RecordBatches whose
//! total row count equals the SQL COUNT(*), and column types come from the schema.
use duckdb::{arrow::record_batch::RecordBatch, Connection};

fn main() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE t AS SELECT range AS id, range * 2 AS twice FROM range(10000);",
    )?;
    let mut stmt = conn.prepare("SELECT id, twice FROM t")?;
    let batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!(
        "batches={} rows={} schema={:?}",
        batches.len(),
        rows,
        batches[0].schema()
    );
    assert_eq!(rows, 10_000);
    assert_eq!(batches[0].num_columns(), 2);
    println!("contract: arrow row count equals COUNT(*) OK");
    Ok(())
}
