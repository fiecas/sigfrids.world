INSERT INTO
    articles (
        author_id,
        slug,
        title,
        content,
        short
    )
VALUES ($1, $2, $3, $4, $5)
RETURNING
    id,
    created_at;
