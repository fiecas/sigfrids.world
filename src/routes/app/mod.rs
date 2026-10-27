use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    http::{error::ResponseError, request::Request, routing::Headers},
    routes::{app::articles::ArticlesRoute, current_user, require_admin},
    utils::with_default_template,
};

use postgres::Client;

mod articles;

pub enum AppRoute {
    Root,
    Articles(ArticlesRoute),
}

impl super::Route for AppRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        Ok(match path {
            "/app" => AppRoute::Root,
            p if p.starts_with("/app/articles") => {
                AppRoute::Articles(ArticlesRoute::from_path(path)?)
            }
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        let user = match self {
            AppRoute::Articles(_) => require_admin(request, &db)?,
            _ => current_user(request, &db)?,
        };

        match self {
            AppRoute::Root => Ok((
                200,
                with_default_template(include_str!("../../../content/app/index.html"))
                    .replace("<display-name />", &user.display_name)
                    .replace("<username />", &user.username)
                    .replace("<email />", &user.email)
                    .into_bytes(),
                HashMap::new(),
            )),
            AppRoute::Articles(route) => route.handle(request, db),
        }
    }
}
