use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use postgres::Client;

use crate::{
    db::{sessions::Session, users::User},
    http::{error::ResponseError, request::Request, routing::Headers},
    passwords::{self, hash, verify},
    utils::{valid_email, valid_username},
};

static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| hash("dummy_hash").expect("failed to generate dummy hash."));

pub enum AuthRoute {
    Login,
    Signup,
    Logout,
}

impl super::super::Route for AuthRoute {
    fn from_path(path: &str) -> Result<AuthRoute, ResponseError> {
        Ok(match path {
            "/api/auth/login" => AuthRoute::Login,
            "/api/auth/signup" => AuthRoute::Signup,
            "/api/auth/logout" => AuthRoute::Logout,
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, String, Headers), ResponseError> {
        match self {
            AuthRoute::Login => {
                let form: HashMap<String, String> = form_urlencoded::parse(&request.body)
                    .into_owned()
                    .filter(|(_, v)| !v.is_empty())
                    .collect();

                let email = form
                    .get("email")
                    .ok_or(ResponseError::BadRequest("Email is required."))?
                    .as_str();

                if !valid_email(email) {
                    return Err(ResponseError::BadRequest("Invalid email."));
                }

                let password = form
                    .get("password")
                    .ok_or(ResponseError::BadRequest("Password is required."))?
                    .as_str();

                if !(8..=128).contains(&password.chars().count()) {
                    return Err(ResponseError::BadRequest("Invalid password."));
                }

                let mut client = db.lock().unwrap();
                let user = User::get(&mut client, email)?;
                drop(client);

                let hash = user
                    .as_ref()
                    .map(|user| user.password_hash.as_str())
                    .unwrap_or(DUMMY_HASH.as_str());

                let valid = verify(password, hash);

                let Some(user) = user.filter(|_| valid) else {
                    return Err(ResponseError::BadRequest("Invalid username or password."));
                };

                client = db.lock().unwrap();
                let session = Session::new(&mut client, user.id)?;
                drop(client);

                let mut headers = HashMap::new();
                headers.insert("Set-Cookie".to_string(), session.to_cookie()?);
                headers.insert("Location".to_string(), "/".to_string());

                Ok((303, "".to_string(), headers))
            }

            AuthRoute::Signup => {
                let form: HashMap<String, String> = form_urlencoded::parse(&request.body)
                    .into_owned()
                    .filter(|(_, v)| !v.is_empty())
                    .collect();

                let email = form
                    .get("email")
                    .ok_or(ResponseError::BadRequest("Email is required."))?
                    .as_str();

                if !valid_email(email) {
                    return Err(ResponseError::BadRequest("Invalid email."));
                }

                let username = form
                    .get("username")
                    .ok_or(ResponseError::BadRequest("Username is required."))?
                    .as_str();

                if !valid_username(username) {
                    return Err(ResponseError::BadRequest("Invalid email."));
                }

                let display_name = form
                    .get("display-name")
                    .ok_or(ResponseError::BadRequest("Display name is required."))?
                    .as_str();

                if !(1..=64).contains(&display_name.chars().count())
                    && !display_name.chars().any(char::is_control)
                {
                    return Err(ResponseError::BadRequest("Invalid display name."));
                }

                let password = form
                    .get("password")
                    .ok_or(ResponseError::BadRequest("Password is required."))?
                    .as_str();

                if !(8..=128).contains(&password.chars().count()) {
                    return Err(ResponseError::BadRequest("Invalid password."));
                }

                let mut client = db.lock().unwrap();
                if User::exists(&mut client, email, username)
                    .map_err(|_| ResponseError::InternalServerError)?
                {
                    return Err(ResponseError::Conflict);
                }
                drop(client);

                let password_hash =
                    passwords::hash(password).map_err(|_| ResponseError::InternalServerError)?;

                client = db.lock().unwrap();
                let user = User::new(
                    &mut client,
                    username,
                    email,
                    display_name,
                    password_hash.as_str(),
                )?;

                let session = Session::new(&mut client, user.id)?;
                drop(client);

                let mut headers = HashMap::new();
                headers.insert("Set-Cookie".to_string(), session.to_cookie()?);
                headers.insert("Location".to_string(), "/".to_string());

                Ok((303, "".to_string(), headers))
            }
            AuthRoute::Logout => {
                let mut headers = HashMap::new();
                headers.insert(
                    "Set-Cookie".to_string(),
                    "session=; Path=/; Max-Age=0".to_string(),
                );
                headers.insert("Location".to_string(), "/".to_string());

                let mut client = db.lock().unwrap();
                if let Some(session) = &request.context.session {
                    let _ = session.delete(&mut client);
                }
                drop(client);

                Ok((303, "".to_string(), headers))
            }
        }
    }
}
