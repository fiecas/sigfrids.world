use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use postgres::Client;

use crate::{
    db::{sessions::Session, users::User},
    http::{error::ResponseError, request::Request, routing::Headers},
    passwords,
    utils::{valid_email, valid_username},
};

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
        if request.method != "POST" {
            return Err(ResponseError::MethodNotAllowed(vec!["POST"]));
        }

        match self {
            AuthRoute::Login => {
                let form: HashMap<String, String> = form_urlencoded::parse(&request.body)
                    .into_owned()
                    .filter(|(_, v)| !v.is_empty())
                    .collect();

                let login = form
                    .get("login")
                    .ok_or(ResponseError::BadRequest("Email or username is required."))?
                    .as_str();

                let password = form
                    .get("password")
                    .ok_or(ResponseError::BadRequest("Password is required."))?
                    .as_str();

                if login.len() > 254 || password.chars().count() > 128 {
                    return Err(ResponseError::Unauthorized);
                }

                let user = {
                    let mut client = db.lock().unwrap();
                    User::get_by_login(&mut client, login)?
                };

                let stored_hash = user.as_ref().map(|user| user.password_hash.as_str());
                let valid = passwords::verify_login(password, stored_hash);

                let Some(user) = user.filter(|_| valid) else {
                    return Err(ResponseError::Unauthorized);
                };

                let (session, token) = {
                    let mut client = db.lock().unwrap();
                    Session::new(&mut *client, user.id)?
                };

                let mut headers = HashMap::new();
                headers.insert("Set-Cookie".to_string(), session.to_cookie(&token)?);
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
                    return Err(ResponseError::BadRequest("Invalid username."));
                }

                let display_name = form
                    .get("display-name")
                    .ok_or(ResponseError::BadRequest("Display name is required."))?
                    .as_str();

                if !(1..=64).contains(&display_name.chars().count())
                    || display_name.chars().any(char::is_control)
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

                {
                    let mut client = db.lock().unwrap();
                    if User::exists(&mut client, email, username)? {
                        return Err(ResponseError::Conflict);
                    }
                }

                let password_hash =
                    passwords::hash(password).map_err(|_| ResponseError::InternalServerError)?;

                let (session, token) = {
                    let mut client = db.lock().unwrap();
                    let mut transaction = client.transaction()?;
                    let user = User::new(
                        &mut transaction,
                        username,
                        email,
                        display_name,
                        password_hash.as_str(),
                    )?;
                    let session = Session::new(&mut transaction, user.id)?;
                    transaction.commit()?;
                    session
                };

                let mut headers = HashMap::new();
                headers.insert("Set-Cookie".to_string(), session.to_cookie(&token)?);
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

                if let Some(cookie) = request.cookies.get("session") {
                    let mut client = db.lock().unwrap();
                    if let Some(session) = Session::from_cookie(&mut client, cookie)? {
                        session.delete(&mut client)?;
                    }
                }

                Ok((303, "".to_string(), headers))
            }
        }
    }
}
