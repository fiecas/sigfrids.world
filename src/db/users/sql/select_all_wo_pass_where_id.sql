SELECT id, username, email, display_name, role::text AS role
FROM users
WHERE
    id = $1;
