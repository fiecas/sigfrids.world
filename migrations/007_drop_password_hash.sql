-- Credentials now live in auth.fiecas.com. This table is only a copy of the
-- profile fields, refreshed on every login, so articles and comments can
-- reference their authors.
ALTER TABLE users DROP COLUMN password_hash;
