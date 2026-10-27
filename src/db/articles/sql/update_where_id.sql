UPDATE articles
SET
    slug = $2,
    title = $3,
    content = $4,
    short = $5,
    edited_at = now()
WHERE
    id = $1;
