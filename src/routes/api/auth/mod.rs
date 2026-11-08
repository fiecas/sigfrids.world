use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use postgres::Client;

use crate::{
    db::sessions::Session,
    http::{error::ResponseError, request::Request, routing::Headers},
};

pub enum AuthRoute {
    Logout,
}

impl super::super::Route for AuthRoute {
    fn from_path(path: &str) -> Result<AuthRoute, ResponseError> {
        Ok(match path {
            "/api/auth/logout" => AuthRoute::Logout,
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        if request.method != "POST" {
            return Err(ResponseError::MethodNotAllowed(vec!["POST"]));
        }

        match self {
            AuthRoute::Logout => {
                let mut headers = HashMap::new();
                headers.insert(
                    "Set-Cookie".to_string(),
                    "session=; Path=/; Max-Age=0".to_string(),
                );
                headers.insert("Location".to_string(), "/".to_string());

                if let Some(cookie) = request.cookies.get("session") {
                    let mut client = db.lock().unwrap();
                    if let Some(session) = Session::from_cookie(&mut client, cookie)? {
                        session.delete(&mut client)?;
                    }
                }

                Ok((303, b"".to_vec(), headers))
            }
        }
    }
}
