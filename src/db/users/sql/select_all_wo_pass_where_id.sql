SELECT id, username, email, display_name
FROM users
WHERE
    id = $1;
