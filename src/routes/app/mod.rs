use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    db::{sessions::Session, users::User},
    http::{error::ResponseError, request::Request, routing::Headers},
    utils::with_default_template,
};

use postgres::Client;

pub enum AppRoute {
    Root,
}

impl super::Route for AppRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        Ok(match path {
            "/app" => AppRoute::Root,
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        let session_cookie = match request.cookies.get("session") {
            Some(session_cookie) => session_cookie,
            None => return Err(ResponseError::Forbidden),
        };

        let mut client = db.lock().unwrap();
        let session =
            Session::from_cookie(&mut client, session_cookie)?.ok_or(ResponseError::Forbidden)?;
        let user =
            User::get_by_id(&mut client, session.user_id)?.ok_or(ResponseError::Forbidden)?;
        drop(client);

        request.context.session = Some(session);

        match self {
            AppRoute::Root => Ok((
                200,
                with_default_template(include_str!("../../../content/app.html"))
                    .replace("<display-name />", &user.display_name)
                    .replace("<username />", &user.username)
                    .replace("<email />", &user.email)
                    .into_bytes(),
                HashMap::new(),
            )),
        }
    }
}
