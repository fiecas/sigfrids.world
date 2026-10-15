use postgres::Client;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::{http::error::ResponseError, passwords::hash};

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
}

impl User {
    pub fn new(
        client: &mut Client,
        username: &str,
        email: &str,
        display_name: &str,
        password_hash: &str,
    ) -> Result<User, ResponseError> {
        let rows = client.query(
            include_str!("./sql/insert_returning_id.sql"),
            &[&username, &email, &display_name, &password_hash],
        )?;
        let row = rows.get(0).unwrap();

        let id = row.get("id");

        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            password_hash: password_hash.to_string(),
        })
    }

    pub fn get(client: &mut Client, username: &str) -> Result<Option<User>, ResponseError> {
        let rows = client
            .query(
                include_str!("./sql/select_all_where_login.sql"),
                &[&username],
            )
            .map_err(|e| match e.code() {
                Some(&postgres::error::SqlState::UNIQUE_VIOLATION) => ResponseError::Conflict,
                _ => ResponseError::from(e),
            })?;
        let row = match rows.get(0) {
            Some(row) => row,
            None => return Ok(None),
        };

        let id = row.get("id");
        let username = row.get("username");
        let email = row.get("email");
        let display_name = row.get("display_name");
        let password_hash = row.get("password_hash");

        Ok(Some(User {
            id,
            username,
            email,
            display_name,
            password_hash,
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
