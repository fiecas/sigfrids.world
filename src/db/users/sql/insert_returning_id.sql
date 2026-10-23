INSERT INTO
    users (
        username,
        email,
        display_name,
        password_hash
    )
VALUES ($1, $2, $3, $4)
RETURNING (id)
