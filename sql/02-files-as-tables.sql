-- 02-files-as-tables.sql — Parquet files ARE tables in DuckDB.
--
-- Run with: duckdb -f sql/02-files-as-tables.sql
--
-- Notice we are not connected to any database file. Every FROM clause names
-- a Parquet file directly. There is no CREATE TABLE, no COPY, no import step.
-- The query engine reads the column statistics out of the Parquet footer
-- and only touches the bytes it needs.

.timer on

-- A single-file query.
SELECT count(*) AS films FROM 'data/pagila/film.parquet';

-- Aggregation against a Parquet file.
SELECT rating, count(*) AS n
FROM 'data/pagila/film.parquet'
GROUP BY rating
ORDER BY n DESC;

-- A two-file join. Note both arguments to JOIN are Parquet paths —
-- DuckDB pushes the column projection and the join through both readers.
SELECT a.first_name, a.last_name,
       count(DISTINCT fa.film_id) AS film_count
FROM 'data/pagila/actor.parquet' a
JOIN 'data/pagila/film_actor.parquet' fa ON fa.actor_id = a.actor_id
GROUP BY a.first_name, a.last_name
ORDER BY film_count DESC, a.last_name
LIMIT 5;
