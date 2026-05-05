use anyhow::Result;
use clap::{Parser, ValueEnum};
use duckdb_reports::{open, top_actors, top_customers, top_films};

#[derive(Parser, Debug)]
#[command(name = "duckdb-reports", version, about = "Sakila reports via DuckDB Parquet")]
struct Cli {
    #[arg(long, value_enum, default_value_t = Report::Customers)]
    report: Report,

    #[arg(long, default_value_t = 10)]
    limit: u32,

    #[arg(long)]
    out: Option<std::path::PathBuf>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Report {
    Customers,
    Films,
    Actors,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = open()?;

    let json = match cli.report {
        Report::Customers => serde_json::to_string_pretty(&top_customers(&conn, cli.limit)?)?,
        Report::Films => serde_json::to_string_pretty(&top_films(&conn, cli.limit)?)?,
        Report::Actors => serde_json::to_string_pretty(&top_actors(&conn, cli.limit)?)?,
    };

    println!("{}", json);
    if let Some(path) = cli.out {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(&path, &json)?;
    }
    Ok(())
}
