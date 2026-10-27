UPDATE users SET role = $2::text::user_role WHERE id = $1;
