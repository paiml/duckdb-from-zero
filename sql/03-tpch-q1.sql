-- 03-tpch-q1.sql — TPC-H Q1 ("Pricing Summary Report") against 6M rows.
--
-- Run with: duckdb data/tpch.duckdb -f sql/03-tpch-q1.sql
--
-- This is the canonical TPC-H Q1 from the benchmark spec. It scans every row
-- of `lineitem` filtered to shipdate <= 1998-09-02 (~98% of the table),
-- groups by (returnflag, linestatus), and aggregates seven values per group.
-- On a laptop, DuckDB returns this in well under a second.

.timer on

SELECT
    l_returnflag,
    l_linestatus,
    sum(l_quantity)                                       AS sum_qty,
    sum(l_extendedprice)                                  AS sum_base_price,
    sum(l_extendedprice * (1 - l_discount))               AS sum_disc_price,
    sum(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge,
    avg(l_quantity)                                       AS avg_qty,
    avg(l_extendedprice)                                  AS avg_price,
    avg(l_discount)                                       AS avg_disc,
    count(*)                                              AS count_order
FROM
    lineitem
WHERE
    l_shipdate <= CAST('1998-09-02' AS date)
GROUP BY
    l_returnflag,
    l_linestatus
ORDER BY
    l_returnflag,
    l_linestatus;
