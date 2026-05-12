-- Pagila top-N customers by rental count.
-- Joins customer.parquet to rental.parquet and ranks customers by the number
-- of rentals they appear in. The Rust binary `duckdb-reports --report
-- customers` executes the same query and asserts contracts C1–C5 on the
-- result (see crates/duckdb-reports/src/lib.rs).
SELECT c.customer_id,
       c.first_name || ' ' || c.last_name AS name,
       COUNT(r.rental_id) AS rental_count,
       c.email
FROM 'data/pagila/customer.parquet' c
LEFT JOIN 'data/pagila/rental.parquet' r ON r.customer_id = c.customer_id
GROUP BY c.customer_id, c.first_name, c.last_name, c.email
ORDER BY rental_count DESC
LIMIT 10;
