-- Pagila top-N actors by distinct film count.
-- Joins actor to the film_actor bridge table to count how many distinct
-- films each actor appeared in. The Rust binary `duckdb-reports --report
-- actors` runs the same query and enforces contracts C1–C5.
SELECT a.actor_id, a.first_name, a.last_name,
       COUNT(DISTINCT fa.film_id) AS film_count
FROM 'data/pagila/actor.parquet' a
LEFT JOIN 'data/pagila/film_actor.parquet' fa ON fa.actor_id = a.actor_id
GROUP BY a.actor_id, a.first_name, a.last_name
ORDER BY film_count DESC
LIMIT 10;
