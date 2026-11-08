use postgres::Client;
use uuid::Uuid;

use crate::http::error::ResponseError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    /// Reads `role::text`. Anything unknown gets the least access.
    fn from_text(text: &str) -> Role {
        match text {
            "admin" => Role::Admin,
            _ => Role::User,
        }
    }
}

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: Role,
}

impl User {
    pub fn get_by_id(client: &mut Client, id: Uuid) -> Result<Option<User>, ResponseError> {
        let row = client.query_opt(
            include_str!("./sql/select_all_wo_pass_where_id.sql"),
            &[&id],
        )?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            role: Role::from_text(row.get("role")),
        }))
    }
}
