SELECT f.film_id, f.title, COUNT(r.rental_id) AS rental_count
FROM 'data/pagila/film.parquet' f
LEFT JOIN 'data/pagila/inventory.parquet' i ON i.film_id = f.film_id
LEFT JOIN 'data/pagila/rental.parquet' r    ON r.inventory_id = i.inventory_id
GROUP BY f.film_id, f.title
ORDER BY rental_count DESC
LIMIT 10;
