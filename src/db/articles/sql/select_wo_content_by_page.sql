SELECT
    id,
    author_id,
    slug,
    title,
    short,
    published_at,
    created_at,
    edited_at
FROM articles
WHERE
    NOT $1
    OR published_at IS NOT NULL
ORDER BY COALESCE(published_at, created_at) DESC
LIMIT $2
OFFSET
    $3
