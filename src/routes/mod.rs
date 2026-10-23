use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod api;
mod app;

use postgres::Client;

use crate::{
    http::{
        error::ResponseError,
        request::Request,
        routing::{Headers, Route},
    },
    routes::{api::ApiRoute, app::AppRoute},
    utils::parse_markdown,
};

#[repr(u16)]
pub enum RootRoute {
    Root,
    Login,
    Signup,
    Api(ApiRoute),
    App(AppRoute),
}

impl Route for RootRoute {
    fn from_path(path: &str) -> Result<RootRoute, ResponseError> {
        Ok(match path {
            "/" => RootRoute::Root,
            "/login" => RootRoute::Login,
            "/signup" => RootRoute::Signup,
            p if p.starts_with("/api") => RootRoute::Api(ApiRoute::from_path(p)?),
            p if p.starts_with("/app") => RootRoute::App(AppRoute::from_path(p)?),
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, String, Headers), ResponseError> {
        match self {
            RootRoute::Root => {
                let content = parse_markdown(include_str!("../../content/home.md"));
                Ok((200, content, HashMap::new()))
            }
            RootRoute::Login => {
                let content = include_str!("../../templates/login.html").to_string();
                Ok((200, content, HashMap::new()))
            }
            RootRoute::Signup => {
                let content = include_str!("../../templates/signup.html").to_string();
                Ok((200, content, HashMap::new()))
            }
            RootRoute::Api(api_route) => api_route.handle(request, db),
            RootRoute::App(app_route) => app_route.handle(request, db),
            _ => Err(ResponseError::NotFound),
        }
    }
}
