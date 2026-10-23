use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::http::error::ResponseError;

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
}

impl User {
    pub fn new(
        client: &mut impl GenericClient,
        username: &str,
        email: &str,
        display_name: &str,
        password_hash: &str,
    ) -> Result<User, ResponseError> {
        let row = client
            .query_one(
                include_str!("./sql/insert_returning_id.sql"),
                &[&username, &email, &display_name, &password_hash],
            )
            .map_err(|e| match e.code() {
                Some(&postgres::error::SqlState::UNIQUE_VIOLATION) => ResponseError::Conflict,
                _ => ResponseError::from(e),
            })?;

        let id = row.get("id");

        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash: password_hash.to_string(),
        })
    }

    pub fn count(client: &mut Client) -> Result<i64, ResponseError> {
        let row = client.query_one(include_str!("./sql/count_all.sql"), &[])?;
        let count: i64 = row.get(0);

        Ok(count)
    }

    pub fn get_by_login(client: &mut Client, login: &str) -> Result<Option<User>, ResponseError> {
        let row = client.query_opt(include_str!("./sql/select_all_where_login.sql"), &[&login])?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),
        }))
    }

    pub fn delete(client: &mut Client, id: Uuid) -> Result<(), ResponseError> {
        client.execute(include_str!("./sql/delete_where_id.sql"), &[&id])?;

        Ok(())
    }

    pub fn exists(client: &mut Client, email: &str, username: &str) -> Result<bool, ResponseError> {
        let rows = client.query(
            include_str!("./sql/select_id_where_username_or_email.sql"),
            &[&email, &username],
        )?;

        Ok(rows.len() > 0)
    }
}
