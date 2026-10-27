use std::sync::{Arc, Mutex};

use crate::{
    http::{error::ResponseError, request::Request, routing::Headers},
    routes::api::articles::ArticlesRoute,
};

mod articles;
mod auth;

use auth::AuthRoute;
use postgres::Client;

pub enum ApiRoute {
    Auth(AuthRoute),
    Articles(ArticlesRoute),
}

impl super::Route for ApiRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        Ok(match path {
            p if p.starts_with("/api/auth") => ApiRoute::Auth(AuthRoute::from_path(p)?),
            p if p.starts_with("/api/articles") => ApiRoute::Articles(ArticlesRoute::from_path(p)?),
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        match self {
            ApiRoute::Auth(route) => route.handle(request, db),
            ApiRoute::Articles(route) => route.handle(request, db),
        }
    }
}
