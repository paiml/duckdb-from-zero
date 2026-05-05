-- 01-tour.sql — three things to notice on first contact with DuckDB.
--
-- Run with: duckdb data/tpch.duckdb -f sql/01-tour.sql
--
-- Notice (1): we never CREATE TABLE. The tpch extension generated the schema
-- when we ran `CALL dbgen(sf=1)` in fetch-data.sh; the .duckdb file IS the
-- database, the way a SQLite file is.

.timer on

-- (2) We can introspect the schema without leaving SQL.
SELECT table_name, estimated_size
FROM duckdb_tables()
ORDER BY estimated_size DESC
LIMIT 8;

-- (3) Six million rows; DuckDB's vectorised executor returns the answer
-- in roughly the time the disk took to read it.
SELECT count(*) AS lineitem_rows FROM lineitem;

SELECT l_shipmode,
       count(*)         AS shipments,
       avg(l_quantity)  AS avg_quantity
FROM lineitem
GROUP BY l_shipmode
ORDER BY shipments DESC;
