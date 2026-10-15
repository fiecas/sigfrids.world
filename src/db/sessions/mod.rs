use postgres::{Client, GenericClient};
use sha2::{Digest, Sha256};
use std::{str::FromStr, time::SystemTime};
use uuid::Uuid;

use crate::{
    constants::SESSION_LENGTH,
    http::error::ResponseError,
    utils::{from_hex, to_hex},
};

pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub secret_hash: Vec<u8>,
    pub expires_at: SystemTime,
}

impl Session {
    pub fn new(
        client: &mut impl GenericClient,
        user_id: Uuid,
    ) -> Result<(Session, String), ResponseError> {
        let mut secret = [0u8; 32];
        getrandom::fill(&mut secret)?;

        let secret_hash = hash_secret(&secret);

        let expires_at = SystemTime::now() + SESSION_LENGTH;

        let row = client.query_one(
            include_str!("./sql/insert_returning_id.sql"),
            &[&user_id, &secret_hash, &expires_at],
        )?;

        let id = row.get("id");
        let token = format!("{}.{}", id, to_hex(&secret));

        Ok((
            Session {
                id,
                user_id,
                secret_hash,
                expires_at,
            },
            token,
        ))
    }

    pub fn get(
        client: &mut Client,
        id: Uuid,
        secret_hash: Vec<u8>,
    ) -> Result<Option<Session>, ResponseError> {
        let rows = client.query(
            include_str!("./sql/select_all_where_id_secret_hash_and_valid.sql"),
            &[&id, &secret_hash],
        )?;
        let row = match rows.get(0) {
            Some(row) => row,
            None => return Ok(None),
        };

        let user_id = row.get("user_id");
        let expires_at = row.get("expires_at");

        if expires_at < SystemTime::now() + SESSION_LENGTH / 2 {
            client.execute(
                include_str!("./sql/update_expires_at_where_id.sql"),
                &[&(SystemTime::now() + SESSION_LENGTH), &id],
            )?;
        }

        Ok(Some(Session {
            id,
            user_id,
            secret_hash,
            expires_at,
        }))
    }

    pub fn delete(&self, client: &mut Client) -> Result<(), ResponseError> {
        client.execute(include_str!("./sql/delete_where_id.sql"), &[&self.id])?;

        Ok(())
    }

    pub fn to_cookie(&self, token: &str) -> Result<String, ResponseError> {
        // TODO: make secure
        Ok(format!(
            "session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
            token,
            self.expires_at.duration_since(SystemTime::now())?.as_secs()
        ))
    }

    pub fn from_cookie(
        client: &mut Client,
        cookie: &str,
    ) -> Result<Option<Session>, ResponseError> {
        let Some((id, secret_hex)) = cookie.split_once(".") else {
            return Ok(None);
        };
        let Ok(id) = Uuid::from_str(id) else {
            return Ok(None);
        };
        let Some(secret) = from_hex(secret_hex) else {
            return Ok(None);
        };

        Session::get(client, id, hash_secret(&secret))
    }
}

fn hash_secret(secret: &[u8]) -> Vec<u8> {
    Sha256::digest(secret).to_vec()
}
