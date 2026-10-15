INSERT INTO
    sessions (
        user_id,
        secret_hash,
        expires_at
    )
VALUES ($1, $2, $3)
RETURNING
    id
