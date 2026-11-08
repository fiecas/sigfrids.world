INSERT INTO
    users (
        id,
        username,
        email,
        display_name,
        role
    )
VALUES ($1, $2, $3, $4, $5::text::user_role)
ON CONFLICT (id) DO UPDATE
SET
    username = EXCLUDED.username,
    email = EXCLUDED.email,
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role;
