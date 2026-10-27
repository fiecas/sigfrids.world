UPDATE articles
SET
    published_at = CASE
        WHEN $2 THEN COALESCE(published_at, now())
        ELSE NULL
    END
WHERE
    id = $1;
