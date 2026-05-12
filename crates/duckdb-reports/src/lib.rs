//! # duckdb-reports
//!
//! Three Sakila/Pagila analytics reports built on DuckDB's Parquet support,
//! each enforced by five named runtime contracts that match
//! [`contracts/duckdb-rust-v1.yaml`](../../../contracts/duckdb-rust-v1.yaml).
//!
//! ## Reports
//!
//! | Function           | Returns                | Source files                                                        |
//! |--------------------|------------------------|---------------------------------------------------------------------|
//! | [`top_customers`]  | [`Vec<TopCustomer>`]   | `customer.parquet`, `rental.parquet`                                |
//! | [`top_films`]      | [`Vec<TopFilm>`]       | `film.parquet`, `inventory.parquet`, `rental.parquet`               |
//! | [`top_actors`]     | [`Vec<TopActor>`]      | `actor.parquet`, `film_actor.parquet`                               |
//!
//! ## Provable contracts (C1–C5)
//!
//! After every successful query the result vector is checked against five
//! contracts named in the YAML spec and re-asserted at runtime:
//!
//! 1. **`row_count_exact`** — `rows.len() == limit`
//! 2. **`top_record_has_count`** — `rows[0].count_field >= 1`
//! 3. **`id_positive`** — every primary key is `> 0`
//! 4. **`string_field_well_formed`** — text fields satisfy a per-report invariant
//! 5. **`count_descending`** — count column is monotonically non-increasing
//!
//! A breach panics with the contract name in the message; both the positive
//! and negative path of each rule are covered by the unit tests in this crate.
//!
//! ## Example
//!
//! ```no_run
//! use duckdb_reports::{open, top_customers};
//! let conn = open()?;
//! let rows = top_customers(&conn, 10)?;
//! assert_eq!(rows.len(), 10);
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use duckdb::{params, Connection};
use serde::Serialize;

// `cargo run` from repo root has CWD = repo root, so `data/pagila/...` is
// correct. `cargo test` runs with CWD = package dir, so the same relative
// path misses. Resolve it once, picking whichever location actually exists.
//
// Split into a thin wrapper + a pure resolver so both branches are unit-testable.
fn pagila_root() -> String {
    resolve_pagila_root(env!("CARGO_MANIFEST_DIR"), |p| {
        std::path::Path::new(p).exists()
    })
}

fn resolve_pagila_root(manifest_dir: &str, exists: impl Fn(&str) -> bool) -> String {
    if exists("data/pagila/customer.parquet") {
        return "data/pagila".to_string();
    }
    std::path::Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("data")
        .join("pagila")
        .to_string_lossy()
        .into_owned()
}

/// One row of the "top customers by rental count" report.
#[derive(Debug, Serialize)]
pub struct TopCustomer {
    /// Sakila `customer.customer_id` — primary key, contract C3 requires `> 0`.
    pub customer_id: i32,
    /// `"First Last"` rendering; contract C4 requires an embedded space.
    pub name: String,
    /// Count of rentals joined from `rental.parquet`; contracts C2 + C5.
    pub rental_count: i64,
    /// Sakila `customer.email`, nullable in the source schema.
    pub email: Option<String>,
}

/// One row of the "top films by rental count" report.
#[derive(Debug, Serialize)]
pub struct TopFilm {
    /// Sakila `film.film_id` — primary key, contract C3 requires `> 0`.
    pub film_id: i32,
    /// Sakila `film.title`; contract C4 requires non-empty.
    pub title: String,
    /// Count of rentals joined via `inventory.parquet`; contracts C2 + C5.
    pub rental_count: i64,
}

/// One row of the "top actors by distinct film count" report.
#[derive(Debug, Serialize)]
pub struct TopActor {
    /// Sakila `actor.actor_id` — primary key, contract C3 requires `> 0`.
    pub actor_id: i32,
    /// Sakila `actor.first_name`; contract C4 requires non-empty.
    pub first_name: String,
    /// Sakila `actor.last_name`; contract C4 requires non-empty.
    pub last_name: String,
    /// Distinct films via `film_actor.parquet`; contracts C2 + C5.
    pub film_count: i64,
}

/// Open a fresh in-memory DuckDB connection.
///
/// All three reports run against this same connection — DuckDB resolves the
/// `'…/customer.parquet'` literals to its built-in Parquet scanner.
pub fn open() -> Result<Connection> {
    Connection::open_in_memory().context("failed to open in-memory DuckDB")
}

/// Run the top-customers-by-rental-count report.
///
/// Joins `customer.parquet` with `rental.parquet` from the Pagila fixture
/// directory and returns the first `limit` rows ordered by descending rental
/// count. Panics if any of the five named contracts (C1–C5) is violated.
pub fn top_customers(conn: &Connection, limit: u32) -> Result<Vec<TopCustomer>> {
    top_customers_in(conn, limit, &pagila_root())
}

/// Run the top-films-by-rental-count report.
///
/// Joins `film.parquet` → `inventory.parquet` → `rental.parquet` from the
/// Pagila fixture directory. Same contract guarantees as [`top_customers`].
pub fn top_films(conn: &Connection, limit: u32) -> Result<Vec<TopFilm>> {
    top_films_in(conn, limit, &pagila_root())
}

/// Run the top-actors-by-distinct-film-count report.
///
/// Joins `actor.parquet` with `film_actor.parquet` from the Pagila fixture
/// directory. Same contract guarantees as [`top_customers`].
pub fn top_actors(conn: &Connection, limit: u32) -> Result<Vec<TopActor>> {
    top_actors_in(conn, limit, &pagila_root())
}

// `*_in` helpers take an explicit Pagila directory so the production wrappers
// stay parameter-free and the negative-path tests can point them at a
// non-existent dir to exercise the `?` error-propagation lines.
//
// Each report is split into three pieces — `build_*_sql` (pure SQL builder),
// `map_*_row` (row deserializer), and `run_query_*` (prepare + query + collect
// + contracts). The split lets unit tests inject a SQL whose result columns
// have the wrong type so every `?` arm in the row-mapper is covered.

fn build_customer_sql(dir: &str) -> String {
    format!(
        "SELECT c.customer_id, \
                c.first_name || ' ' || c.last_name AS name, \
                COUNT(r.rental_id) AS rental_count, \
                c.email \
         FROM '{0}/customer.parquet' c \
         LEFT JOIN '{0}/rental.parquet' r ON r.customer_id = c.customer_id \
         GROUP BY c.customer_id, c.first_name, c.last_name, c.email \
         ORDER BY rental_count DESC \
         LIMIT ?",
        dir
    )
}

fn build_film_sql(dir: &str) -> String {
    format!(
        "SELECT f.film_id, f.title, COUNT(r.rental_id) AS rental_count \
         FROM '{0}/film.parquet' f \
         LEFT JOIN '{0}/inventory.parquet' i ON i.film_id = f.film_id \
         LEFT JOIN '{0}/rental.parquet'    r ON r.inventory_id = i.inventory_id \
         GROUP BY f.film_id, f.title \
         ORDER BY rental_count DESC \
         LIMIT ?",
        dir
    )
}

fn build_actor_sql(dir: &str) -> String {
    format!(
        "SELECT a.actor_id, a.first_name, a.last_name, \
                COUNT(DISTINCT fa.film_id) AS film_count \
         FROM '{0}/actor.parquet' a \
         LEFT JOIN '{0}/film_actor.parquet' fa ON fa.actor_id = a.actor_id \
         GROUP BY a.actor_id, a.first_name, a.last_name \
         ORDER BY film_count DESC \
         LIMIT ?",
        dir
    )
}

fn map_customer_row(r: &duckdb::Row<'_>) -> duckdb::Result<TopCustomer> {
    Ok(TopCustomer {
        customer_id: r.get(0)?,
        name: r.get(1)?,
        rental_count: r.get(2)?,
        email: r.get(3)?,
    })
}

fn map_film_row(r: &duckdb::Row<'_>) -> duckdb::Result<TopFilm> {
    Ok(TopFilm {
        film_id: r.get(0)?,
        title: r.get(1)?,
        rental_count: r.get(2)?,
    })
}

fn map_actor_row(r: &duckdb::Row<'_>) -> duckdb::Result<TopActor> {
    Ok(TopActor {
        actor_id: r.get(0)?,
        first_name: r.get(1)?,
        last_name: r.get(2)?,
        film_count: r.get(3)?,
    })
}

fn run_query_customers(conn: &Connection, limit: u32, sql: &str) -> Result<Vec<TopCustomer>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![limit], map_customer_row)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_customers(&rows, limit as usize);
    Ok(rows)
}

fn run_query_films(conn: &Connection, limit: u32, sql: &str) -> Result<Vec<TopFilm>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![limit], map_film_row)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_films(&rows, limit as usize);
    Ok(rows)
}

fn run_query_actors(conn: &Connection, limit: u32, sql: &str) -> Result<Vec<TopActor>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![limit], map_actor_row)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_contracts_actors(&rows, limit as usize);
    Ok(rows)
}

fn top_customers_in(conn: &Connection, limit: u32, dir: &str) -> Result<Vec<TopCustomer>> {
    run_query_customers(conn, limit, &build_customer_sql(dir))
}

fn top_films_in(conn: &Connection, limit: u32, dir: &str) -> Result<Vec<TopFilm>> {
    run_query_films(conn, limit, &build_film_sql(dir))
}

fn top_actors_in(conn: &Connection, limit: u32, dir: &str) -> Result<Vec<TopActor>> {
    run_query_actors(conn, limit, &build_actor_sql(dir))
}

// Provable contracts — formal spec: contracts/duckdb-rust-v1.yaml
// Each assertion is a runtime check whose Lean theorem name appears in the YAML.

fn assert_contracts_customers(rows: &[TopCustomer], limit: usize) {
    // Provable contract C1 row_count_exact: LIMIT N must return exactly N rows.
    assert_eq!(rows.len(), limit, "C1 row_count_exact: rows.len() == limit");
    // Provable contract C2 top_record_has_count: top customer has ≥ 1 rental.
    assert!(
        rows[0].rental_count >= 1,
        "C2 top_record_has_count: rows[0].rental_count >= 1"
    );
    for r in rows {
        // Provable contract C3 id_positive: AUTO_INCREMENT customer_ids start at 1.
        assert!(r.customer_id > 0, "C3 id_positive: customer_id > 0");
        // Provable contract C4 string_field_well_formed: name is "First Last".
        assert!(
            r.name.contains(' '),
            "C4 string_field_well_formed: name contains space"
        );
    }
    // Provable contract C5 count_descending: ORDER BY rental_count DESC holds pairwise.
    for w in rows.windows(2) {
        assert!(
            w[0].rental_count >= w[1].rental_count,
            "C5 count_descending: rental_count monotonically non-increasing"
        );
    }
}

fn assert_contracts_films(rows: &[TopFilm], limit: usize) {
    // C1 row_count_exact
    assert_eq!(rows.len(), limit, "C1 row_count_exact");
    // C2 top_record_has_count
    assert!(rows[0].rental_count >= 1, "C2 top_record_has_count");
    for r in rows {
        // C3 id_positive
        assert!(r.film_id > 0, "C3 id_positive: film_id > 0");
        // C4 string_field_well_formed: title is non-empty
        assert!(!r.title.is_empty(), "C4 string_field_well_formed: title");
    }
    // C5 count_descending
    for w in rows.windows(2) {
        assert!(
            w[0].rental_count >= w[1].rental_count,
            "C5 count_descending"
        );
    }
}

fn assert_contracts_actors(rows: &[TopActor], limit: usize) {
    // C1 row_count_exact
    assert_eq!(rows.len(), limit, "C1 row_count_exact");
    // C2 top_record_has_count
    assert!(rows[0].film_count >= 1, "C2 top_record_has_count");
    for r in rows {
        // C3 id_positive
        assert!(r.actor_id > 0, "C3 id_positive: actor_id > 0");
        // C4 string_field_well_formed: first_name AND last_name non-empty
        assert!(
            !r.first_name.is_empty() && !r.last_name.is_empty(),
            "C4 string_field_well_formed: first_name + last_name"
        );
    }
    // C5 count_descending
    for w in rows.windows(2) {
        assert!(w[0].film_count >= w[1].film_count, "C5 count_descending");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_uses_cwd_relative_when_data_present() {
        let r = resolve_pagila_root("/imaginary/crates/foo", |_| true);
        assert_eq!(r, "data/pagila");
    }

    #[test]
    fn resolve_falls_back_to_manifest_relative_when_data_missing() {
        let r = resolve_pagila_root("/imaginary/crates/foo", |_| false);
        // Two levels up from the crate dir, then data/pagila.
        assert!(r.ends_with("data/pagila"));
        assert!(r.contains("imaginary"));
    }

    // Direct tests of the contract assertions to lock down each rule independently
    // and to exercise the negative path of every Provable contract.

    #[test]
    fn customers_contract_holds_on_valid_input() {
        let rows = vec![
            TopCustomer {
                customer_id: 1,
                name: "ALICE A".to_string(),
                rental_count: 3,
                email: None,
            },
            TopCustomer {
                customer_id: 2,
                name: "BOB B".to_string(),
                rental_count: 2,
                email: None,
            },
        ];
        assert_contracts_customers(&rows, 2);
    }

    #[test]
    #[should_panic(expected = "C1 row_count_exact")]
    fn customers_contract_c1_row_count_violation() {
        let rows = vec![TopCustomer {
            customer_id: 1,
            name: "A B".to_string(),
            rental_count: 1,
            email: None,
        }];
        assert_contracts_customers(&rows, 5);
    }

    #[test]
    #[should_panic(expected = "C2 top_record_has_count")]
    fn customers_contract_c2_top_count_violation() {
        let rows = vec![TopCustomer {
            customer_id: 1,
            name: "A B".to_string(),
            rental_count: 0,
            email: None,
        }];
        assert_contracts_customers(&rows, 1);
    }

    #[test]
    #[should_panic(expected = "C3 id_positive")]
    fn customers_contract_c3_id_violation() {
        let rows = vec![TopCustomer {
            customer_id: 0,
            name: "A B".to_string(),
            rental_count: 1,
            email: None,
        }];
        assert_contracts_customers(&rows, 1);
    }

    #[test]
    #[should_panic(expected = "C4 string_field_well_formed")]
    fn customers_contract_c4_name_violation() {
        let rows = vec![TopCustomer {
            customer_id: 1,
            name: "AliceWithoutSpace".to_string(),
            rental_count: 1,
            email: None,
        }];
        assert_contracts_customers(&rows, 1);
    }

    #[test]
    #[should_panic(expected = "C5 count_descending")]
    fn customers_contract_c5_order_violation() {
        let rows = vec![
            TopCustomer {
                customer_id: 1,
                name: "A B".to_string(),
                rental_count: 1,
                email: None,
            },
            TopCustomer {
                customer_id: 2,
                name: "C D".to_string(),
                rental_count: 5,
                email: None,
            },
        ];
        assert_contracts_customers(&rows, 2);
    }

    #[test]
    fn films_contract_holds_on_valid_input() {
        let rows = vec![
            TopFilm {
                film_id: 1,
                title: "ACADEMY DINOSAUR".to_string(),
                rental_count: 3,
            },
            TopFilm {
                film_id: 2,
                title: "BEDROOM ROMANCE".to_string(),
                rental_count: 2,
            },
        ];
        assert_contracts_films(&rows, 2);
    }

    #[test]
    #[should_panic(expected = "C1 row_count_exact")]
    fn films_contract_c1_violation() {
        assert_contracts_films(
            &[TopFilm {
                film_id: 1,
                title: "X".to_string(),
                rental_count: 1,
            }],
            5,
        );
    }

    #[test]
    #[should_panic(expected = "C2 top_record_has_count")]
    fn films_contract_c2_violation() {
        assert_contracts_films(
            &[TopFilm {
                film_id: 1,
                title: "X".to_string(),
                rental_count: 0,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C3 id_positive: film_id")]
    fn films_contract_c3_violation() {
        assert_contracts_films(
            &[TopFilm {
                film_id: 0,
                title: "X".to_string(),
                rental_count: 1,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C4 string_field_well_formed")]
    fn films_contract_c4_violation() {
        assert_contracts_films(
            &[TopFilm {
                film_id: 1,
                title: String::new(),
                rental_count: 1,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C5 count_descending")]
    fn films_contract_c5_violation() {
        assert_contracts_films(
            &[
                TopFilm {
                    film_id: 1,
                    title: "X".to_string(),
                    rental_count: 1,
                },
                TopFilm {
                    film_id: 2,
                    title: "Y".to_string(),
                    rental_count: 5,
                },
            ],
            2,
        );
    }

    #[test]
    fn actors_contract_holds_on_valid_input() {
        let rows = vec![
            TopActor {
                actor_id: 1,
                first_name: "PENELOPE".to_string(),
                last_name: "GUINESS".to_string(),
                film_count: 5,
            },
            TopActor {
                actor_id: 2,
                first_name: "NICK".to_string(),
                last_name: "WAHLBERG".to_string(),
                film_count: 4,
            },
        ];
        assert_contracts_actors(&rows, 2);
    }

    #[test]
    #[should_panic(expected = "C1 row_count_exact")]
    fn actors_contract_c1_violation() {
        assert_contracts_actors(
            &[TopActor {
                actor_id: 1,
                first_name: "A".to_string(),
                last_name: "B".to_string(),
                film_count: 1,
            }],
            5,
        );
    }

    #[test]
    #[should_panic(expected = "C2 top_record_has_count")]
    fn actors_contract_c2_violation() {
        assert_contracts_actors(
            &[TopActor {
                actor_id: 1,
                first_name: "A".to_string(),
                last_name: "B".to_string(),
                film_count: 0,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C3 id_positive: actor_id")]
    fn actors_contract_c3_violation() {
        assert_contracts_actors(
            &[TopActor {
                actor_id: 0,
                first_name: "A".to_string(),
                last_name: "B".to_string(),
                film_count: 1,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C4 string_field_well_formed")]
    fn actors_contract_c4_violation() {
        assert_contracts_actors(
            &[TopActor {
                actor_id: 1,
                first_name: String::new(),
                last_name: "B".to_string(),
                film_count: 1,
            }],
            1,
        );
    }

    #[test]
    #[should_panic(expected = "C5 count_descending")]
    fn actors_contract_c5_violation() {
        assert_contracts_actors(
            &[
                TopActor {
                    actor_id: 1,
                    first_name: "A".to_string(),
                    last_name: "B".to_string(),
                    film_count: 1,
                },
                TopActor {
                    actor_id: 2,
                    first_name: "C".to_string(),
                    last_name: "D".to_string(),
                    film_count: 5,
                },
            ],
            2,
        );
    }

    // `?` error-path coverage — pointing each `*_in` at a non-existent Pagila dir
    // forces DuckDB to fail at prepare()-time, exercising the error branch of
    // every `?` operator inside the query body.

    #[test]
    fn top_customers_in_returns_err_for_missing_data_dir() {
        let conn = open().expect("open in-memory DuckDB");
        let r = top_customers_in(&conn, 5, "/nonexistent/duckdb/from/zero/dir");
        assert!(r.is_err(), "expected query against missing parquet to fail");
    }

    #[test]
    fn top_films_in_returns_err_for_missing_data_dir() {
        let conn = open().expect("open in-memory DuckDB");
        let r = top_films_in(&conn, 5, "/nonexistent/duckdb/from/zero/dir");
        assert!(r.is_err());
    }

    #[test]
    fn top_actors_in_returns_err_for_missing_data_dir() {
        let conn = open().expect("open in-memory DuckDB");
        let r = top_actors_in(&conn, 5, "/nonexistent/duckdb/from/zero/dir");
        assert!(r.is_err());
    }

    // SQL builders must produce a `LIMIT ?` placeholder that the runner binds.

    #[test]
    fn build_customer_sql_embeds_dir_and_limit_placeholder() {
        let sql = build_customer_sql("/some/dir");
        assert!(sql.contains("'/some/dir/customer.parquet'"));
        assert!(sql.contains("'/some/dir/rental.parquet'"));
        assert!(sql.ends_with("LIMIT ?"));
    }

    #[test]
    fn build_film_sql_embeds_dir_and_limit_placeholder() {
        let sql = build_film_sql("/some/dir");
        assert!(sql.contains("'/some/dir/film.parquet'"));
        assert!(sql.contains("'/some/dir/inventory.parquet'"));
        assert!(sql.contains("'/some/dir/rental.parquet'"));
        assert!(sql.ends_with("LIMIT ?"));
    }

    #[test]
    fn build_actor_sql_embeds_dir_and_limit_placeholder() {
        let sql = build_actor_sql("/some/dir");
        assert!(sql.contains("'/some/dir/actor.parquet'"));
        assert!(sql.contains("'/some/dir/film_actor.parquet'"));
        assert!(sql.ends_with("LIMIT ?"));
    }

    // Row-deserializer error paths: feed each `run_query_*` a SELECT whose
    // column types do not match the struct fields, forcing `r.get::<_, T>(i)?`
    // inside the corresponding `map_*_row` helper to return Err. This covers
    // the `?` propagation arms that are otherwise unreachable when the real
    // Pagila parquet schema is used.

    #[test]
    fn run_query_customers_propagates_row_mapper_error_on_bad_types() {
        // customer_id expected i32 but we hand back a VARCHAR.
        let conn = open().expect("open");
        let bad =
            "SELECT 'not-an-int'::VARCHAR, 'A B', 1::BIGINT, NULL::VARCHAR FROM range(?) LIMIT ?";
        // range(?) consumes the bind once; LIMIT ? consumes it again. We only
        // want one prepared parameter, so wrap with an explicit LIMIT and drop
        // the FROM range parameter — DuckDB will still parse the column types.
        let _ = bad; // keep for clarity; below is the actual minimal repro.
        let sql = "SELECT 'not-an-int'::VARCHAR AS a, 'A B' AS b, 1::BIGINT AS c, NULL::VARCHAR AS d FROM range(5) LIMIT ?";
        let r = run_query_customers(&conn, 5, sql);
        assert!(
            r.is_err(),
            "expected row-mapper to surface a type-mismatch error"
        );
    }

    #[test]
    fn run_query_films_propagates_row_mapper_error_on_bad_types() {
        let conn = open().expect("open");
        // film_id expected i32 but VARCHAR provided.
        let sql = "SELECT 'oops'::VARCHAR AS a, 'title' AS b, 1::BIGINT AS c FROM range(5) LIMIT ?";
        let r = run_query_films(&conn, 5, sql);
        assert!(r.is_err());
    }

    #[test]
    fn run_query_actors_propagates_row_mapper_error_on_bad_types() {
        let conn = open().expect("open");
        // actor_id expected i32 but VARCHAR provided.
        let sql =
            "SELECT 'oops'::VARCHAR AS a, 'F' AS b, 'L' AS c, 1::BIGINT AS d FROM range(5) LIMIT ?";
        let r = run_query_actors(&conn, 5, sql);
        assert!(r.is_err());
    }
}
