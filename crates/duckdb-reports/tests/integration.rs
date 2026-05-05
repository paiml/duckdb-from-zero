use duckdb_reports::*;

#[test]
fn customers_report_matches_contracts() {
    let conn = open().expect("DuckDB connect");
    let rows = top_customers(&conn, 10).expect("query");
    assert_eq!(rows.len(), 10);
    let names: Vec<_> = rows.iter().map(|r| r.name.clone()).collect();
    assert!(
        names.iter().all(|n| n.contains(' ')),
        "names should be 'First Last': {:?}",
        names
    );
}

#[test]
fn films_report_matches_contracts() {
    let conn = open().expect("DuckDB connect");
    let rows = top_films(&conn, 5).expect("query");
    assert_eq!(rows.len(), 5);
    assert!(rows[0].rental_count >= rows[4].rental_count);
}

#[test]
fn actors_report_matches_contracts() {
    let conn = open().expect("DuckDB connect");
    let rows = top_actors(&conn, 3).expect("query");
    assert_eq!(rows.len(), 3);
    assert!(rows[0].film_count >= 1);
}
