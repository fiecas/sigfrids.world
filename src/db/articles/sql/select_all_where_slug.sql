SELECT
    id,
    author_id,
    title,
    content,
    short,
    published_at,
    created_at,
    edited_at
FROM articles
WHERE
    slug = $1;
