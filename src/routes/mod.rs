use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod api;
mod app;
mod articles;

use include_dir::{Dir, include_dir};
use postgres::Client;

use crate::{
    db::{
        sessions::Session,
        users::{Role, User},
    },
    http::{
        error::ResponseError,
        request::Request,
        routing::{Headers, Route},
    },
    routes::{api::ApiRoute, app::AppRoute, articles::ArticlesRoute},
    utils::{parse_markdown, with_default_template},
};

/// The logged-in user. Also puts their session in the request context.
pub fn current_user(request: &mut Request, db: &Arc<Mutex<Client>>) -> Result<User, ResponseError> {
    let cookie = request
        .cookies
        .get("session")
        .ok_or(ResponseError::Unauthorized)?;

    let mut client = db.lock().unwrap();
    let session = Session::from_cookie(&mut client, cookie)?.ok_or(ResponseError::Unauthorized)?;
    let user = User::get_by_id(&mut client, session.user_id)?.ok_or(ResponseError::Unauthorized)?;
    drop(client);

    request.context.session = Some(session);
    Ok(user)
}

/// `?page=`, counted from 0. Missing means the first page.
pub fn parse_page(request: &Request) -> Result<u32, ResponseError> {
    match request.query.get("page") {
        Some(page) => page
            .parse()
            .map_err(|_| ResponseError::BadRequest("Invalid page.")),
        None => Ok(0),
    }
}

/// The logged-in user, if they are an admin.
pub fn require_admin(
    request: &mut Request,
    db: &Arc<Mutex<Client>>,
) -> Result<User, ResponseError> {
    let user = current_user(request, db)?;
    if user.role != Role::Admin {
        return Err(ResponseError::Forbidden);
    }
    Ok(user)
}

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[repr(u16)]
pub enum RootRoute {
    Root,
    Health,
    Login,
    Signup,
    Articles(ArticlesRoute),
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
            p if p.starts_with("/articles") => RootRoute::Articles(ArticlesRoute::from_path(p)?),
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
                let content = with_default_template(include_str!("../../content/home.html"));
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Health => {
                let mut db = db.lock().map_err(|_| ResponseError::InternalServerError)?;
                db.simple_query("SELECT 1")?;
                Ok((200, b"OK".to_vec(), HashMap::new()))
            }
            RootRoute::Login => {
                let content = with_default_template(include_str!("../../content/login.html"));
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Signup => {
                let content = with_default_template(include_str!("../../content/signup.html"));
                Ok((200, content.into_bytes(), HashMap::new()))
            }
            RootRoute::Articles(route) => route.handle(request, db),
            RootRoute::Api(route) => route.handle(request, db),
            RootRoute::App(route) => route.handle(request, db),
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
