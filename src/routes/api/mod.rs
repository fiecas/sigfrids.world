use std::sync::{Arc, Mutex};

use crate::http::{error::ResponseError, request::Request, routing::Headers};

mod auth;
use auth::AuthRoute;
use postgres::Client;

pub enum ApiRoute {
    Auth(AuthRoute),
}

impl super::Route for ApiRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        Ok(match path {
            p if p.starts_with("/api/auth") => ApiRoute::Auth(AuthRoute::from_path(p)?),
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, String, Headers), ResponseError> {
        match self {
            ApiRoute::Auth(auth_route) => auth_route.handle(request, db),
        }
    }
}
