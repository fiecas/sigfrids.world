SELECT *
FROM users
WHERE
    lower(email) = lower($1)
    OR lower(username) = lower($1);
