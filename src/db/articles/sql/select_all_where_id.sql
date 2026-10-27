SELECT
    author_id,
    slug,
    title,
    content,
    short,
    published_at,
    created_at,
    edited_at
FROM articles
WHERE
    id = $1;
