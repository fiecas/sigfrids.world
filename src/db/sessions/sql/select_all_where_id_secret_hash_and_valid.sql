SELECT *
FROM sessions
WHERE
    id = $1
    AND secret_hash = $2
    AND expires_at > now();
