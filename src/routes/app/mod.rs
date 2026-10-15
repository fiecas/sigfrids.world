use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    db::sessions::Session,
    http::{error::ResponseError, request::Request, routing::Headers},
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
    ) -> Result<(u16, String, Headers), ResponseError> {
        let session_cookie = match request.cookies.get("session") {
            Some(session_cookie) => session_cookie,
            None => return Err(ResponseError::Forbidden),
        };

        let mut client = db.lock().unwrap();
        match Session::from_cookie(&mut client, session_cookie)? {
            Some(session) => request.context.session = Some(session),
            None => return Err(ResponseError::Forbidden),
        };
        drop(client);

        match self {
            AppRoute::Root => Ok((
                200,
                include_str!("../../../templates/app.html").to_string(),
                HashMap::new(),
            )),
        }
    }
}
