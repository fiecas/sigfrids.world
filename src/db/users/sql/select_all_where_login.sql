SELECT
    id,
    username,
    email,
    display_name,
    password_hash,
    role::text AS role
FROM users
WHERE
    lower(email) = lower($1)
    OR lower(username) = lower($1);
