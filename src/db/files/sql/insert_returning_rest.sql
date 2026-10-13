INSERT INTO
    files (
        original_name,
        content_type,
        size
    )
VALUES ($1, $2, $3)
RETURNING
    id,
    created_at;
