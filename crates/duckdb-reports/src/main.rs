use anyhow::Result;
use clap::{Parser, ValueEnum};
use duckdb::Connection;
use duckdb_reports::{open, top_actors, top_customers, top_films};
use std::path::Path;

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

fn render(cli: &Cli, conn: &Connection) -> Result<String> {
    let json = match cli.report {
        Report::Customers => serde_json::to_string_pretty(&top_customers(conn, cli.limit)?)?,
        Report::Films => serde_json::to_string_pretty(&top_films(conn, cli.limit)?)?,
        Report::Actors => serde_json::to_string_pretty(&top_actors(conn, cli.limit)?)?,
    };
    Ok(json)
}

fn write_output(path: &Path, json: &str) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    let conn = open()?;
    let json = render(&cli, &conn)?;
    println!("{}", json);
    if let Some(path) = cli.out {
        write_output(&path, &json)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    run(Cli::parse())
}
