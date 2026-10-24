use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod api;
mod app;

use include_dir::{Dir, include_dir};
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

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[repr(u16)]
pub enum RootRoute {
    Root,
    Health,
    Login,
    Signup,
    Api(ApiRoute),
    App(AppRoute),
    Assets(String),
}

impl Route for RootRoute {
    fn from_path(path: &str) -> Result<RootRoute, ResponseError> {
        Ok(match path {
            "/" => RootRoute::Root,
            "/health" => RootRoute::Health,
            "/login" => RootRoute::Login,
            "/signup" => RootRoute::Signup,
            p if p.starts_with("/api") => RootRoute::Api(ApiRoute::from_path(p)?),
            p if p.starts_with("/app") => RootRoute::App(AppRoute::from_path(p)?),
            p if p.starts_with("/assets") && path.len() > 8 => {
                RootRoute::Assets(path[8..].to_string())
            }
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        match self {
            RootRoute::Root => {
                let content = parse_markdown(include_str!("../../content/home.md"));
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Health => {
                let mut db = db.lock().map_err(|_| ResponseError::InternalServerError)?;
                db.simple_query("SELECT 1")?;
                Ok((200, b"OK".to_vec(), HashMap::new()))
            }
            RootRoute::Login => {
                let content = include_str!("../../templates/login.html").to_string();
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Signup => {
                let content = include_str!("../../templates/signup.html").to_string();
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Api(api_route) => api_route.handle(request, db),
            RootRoute::App(app_route) => app_route.handle(request, db),
            RootRoute::Assets(path) => {
                let file = ASSETS.get_file(path).ok_or(ResponseError::NotFound)?;
                let content_type = match file.path().extension().and_then(|e| e.to_str()) {
                    Some("css") => "text/css",
                    Some("js") => "text/javascript",
                    Some("png") => "image/png",
                    Some("gif") => "image/gif",
                    Some("jpg" | "jpeg") => "image/jpeg",
                    Some("svg") => "image/svg+xml",
                    Some("woff2") => "font/woff2",
                    _ => "application/octet-stream",
                };

                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), content_type.to_string());
                Ok((200, file.contents().to_vec(), headers))
            }
        }
    }
}
