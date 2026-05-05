use anyhow::{Context, Result};
use duckdb::{params, Connection};
use serde::Serialize;

// `cargo run` from repo root has CWD = repo root, so `data/pagila/...` is
// correct. `cargo test` runs with CWD = package dir, so the same relative
// path misses. Resolve it once, picking whichever location actually exists.
fn pagila_root() -> String {
    if std::path::Path::new("data/pagila/customer.parquet").exists() {
        return "data/pagila".to_string();
    }
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest)
        .join("..")
        .join("..")
        .join("data")
        .join("pagila")
        .to_string_lossy()
        .into_owned()
}

#[derive(Debug, Serialize)]
pub struct TopCustomer {
    pub customer_id: i32,
    pub name: String,
    pub rental_count: i64,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TopFilm {
    pub film_id: i32,
    pub title: String,
    pub rental_count: i64,
}

#[derive(Debug, Serialize)]
pub struct TopActor {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub film_count: i64,
}

pub fn open() -> Result<Connection> {
    Connection::open_in_memory().context("failed to open in-memory DuckDB")
}

pub fn top_customers(conn: &Connection, limit: u32) -> Result<Vec<TopCustomer>> {
    let sql = format!(
        "SELECT c.customer_id, \
                c.first_name || ' ' || c.last_name AS name, \
                COUNT(r.rental_id) AS rental_count, \
                c.email \
         FROM '{0}/customer.parquet' c \
         LEFT JOIN '{0}/rental.parquet' r ON r.customer_id = c.customer_id \
         GROUP BY c.customer_id, c.first_name, c.last_name, c.email \
         ORDER BY rental_count DESC \
         LIMIT ?",
        pagila_root()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(TopCustomer {
                customer_id: r.get(0)?,
                name: r.get(1)?,
                rental_count: r.get(2)?,
                email: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_customers(&rows, limit as usize);
    Ok(rows)
}

pub fn top_films(conn: &Connection, limit: u32) -> Result<Vec<TopFilm>> {
    let sql = format!(
        "SELECT f.film_id, f.title, COUNT(r.rental_id) AS rental_count \
         FROM '{0}/film.parquet' f \
         LEFT JOIN '{0}/inventory.parquet' i ON i.film_id = f.film_id \
         LEFT JOIN '{0}/rental.parquet'    r ON r.inventory_id = i.inventory_id \
         GROUP BY f.film_id, f.title \
         ORDER BY rental_count DESC \
         LIMIT ?",
        pagila_root()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(TopFilm {
                film_id: r.get(0)?,
                title: r.get(1)?,
                rental_count: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_films(&rows, limit as usize);
    Ok(rows)
}

pub fn top_actors(conn: &Connection, limit: u32) -> Result<Vec<TopActor>> {
    let sql = format!(
        "SELECT a.actor_id, a.first_name, a.last_name, \
                COUNT(DISTINCT fa.film_id) AS film_count \
         FROM '{0}/actor.parquet' a \
         LEFT JOIN '{0}/film_actor.parquet' fa ON fa.actor_id = a.actor_id \
         GROUP BY a.actor_id, a.first_name, a.last_name \
         ORDER BY film_count DESC \
         LIMIT ?",
        pagila_root()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(TopActor {
                actor_id: r.get(0)?,
                first_name: r.get(1)?,
                last_name: r.get(2)?,
                film_count: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_actors(&rows, limit as usize);
    Ok(rows)
}

fn assert_contracts_customers(rows: &[TopCustomer], limit: usize) {
    assert_eq!(rows.len(), limit, "row count must match --limit");
    assert!(rows[0].rental_count >= 1, "top customer has at least one rental");
    for r in rows {
        assert!(r.customer_id > 0, "customer_id positive (got {})", r.customer_id);
        assert!(r.name.contains(' '), "name should be 'First Last' (got {:?})", r.name);
    }
    for w in rows.windows(2) {
        assert!(w[0].rental_count >= w[1].rental_count, "ORDER BY DESC preserved");
    }
}

fn assert_contracts_films(rows: &[TopFilm], limit: usize) {
    assert_eq!(rows.len(), limit);
    assert!(rows[0].rental_count >= 1);
    for r in rows {
        assert!(r.film_id > 0);
        assert!(!r.title.is_empty());
    }
    for w in rows.windows(2) {
        assert!(w[0].rental_count >= w[1].rental_count);
    }
}

fn assert_contracts_actors(rows: &[TopActor], limit: usize) {
    assert_eq!(rows.len(), limit);
    assert!(rows[0].film_count >= 1);
    for r in rows {
        assert!(r.actor_id > 0);
        assert!(!r.first_name.is_empty() && !r.last_name.is_empty());
    }
    for w in rows.windows(2) {
        assert!(w[0].film_count >= w[1].film_count);
    }
}
