use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use postgres::Client;
use uuid::Uuid;

use crate::{
    db::articles::Article,
    http::{error::ResponseError, request::Request, routing::Headers},
    routes::{parse_page, require_admin},
};

pub enum ArticlesRoute {
    New,
    Edit,
    Publish,
    GetPage,
}

impl super::super::Route for ArticlesRoute {
    fn from_path(path: &str) -> Result<ArticlesRoute, ResponseError> {
        Ok(match path {
            "/api/articles/new" => ArticlesRoute::New,
            "/api/articles/edit" => ArticlesRoute::Edit,
            "/api/articles/publish" => ArticlesRoute::Publish,
            "/api/articles/get-page" => ArticlesRoute::GetPage,
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        match self {
            ArticlesRoute::New => {
                let user = require_admin(request, &db)?;

                if request.method != "POST" {
                    return Err(ResponseError::MethodNotAllowed(vec!["POST"]));
                }

                let form = ArticleForm::parse(&request.body)?;

                let mut client = db.lock().unwrap();
                let article = Article::new(
                    &mut client,
                    user.id,
                    &form.slug,
                    &form.title,
                    &form.content,
                    form.short.as_deref(),
                )?;
                drop(client);

                Ok(redirect(format!("/app/articles/edit?id={}", article.id)))
            }
            ArticlesRoute::Edit => {
                require_admin(request, &db)?;

                if request.method != "POST" {
                    return Err(ResponseError::MethodNotAllowed(vec!["POST"]));
                }

                let id = parse_id(&parse_form(&request.body))?;
                let form = ArticleForm::parse(&request.body)?;

                let mut client = db.lock().unwrap();
                let found = Article::update(
                    &mut client,
                    id,
                    &form.slug,
                    &form.title,
                    &form.content,
                    form.short.as_deref(),
                )?;
                drop(client);

                if !found {
                    return Err(ResponseError::NotFound);
                }

                Ok(redirect(format!("/app/articles/edit?id={id}")))
            }
            ArticlesRoute::Publish => {
                require_admin(request, &db)?;

                if request.method != "POST" {
                    return Err(ResponseError::MethodNotAllowed(vec!["POST"]));
                }

                let form = parse_form(&request.body);
                let id = parse_id(&form)?;

                // The button sends the state it wants rather than "toggle", so a
                // double submit or a stale tab can't flip it back.
                let published = match form.get("published").map(String::as_str) {
                    Some("true") => true,
                    Some("false") => false,
                    _ => return Err(ResponseError::BadRequest("Invalid published.")),
                };

                let mut client = db.lock().unwrap();
                let found = Article::set_published(&mut client, id, published)?;
                drop(client);

                if !found {
                    return Err(ResponseError::NotFound);
                }

                Ok(redirect(format!("/app/articles/edit?id={id}")))
            }
            ArticlesRoute::GetPage => {
                if request.method != "GET" {
                    return Err(ResponseError::MethodNotAllowed(vec!["GET"]));
                }

                let page = parse_page(request)?;

                let mut client = db.lock().unwrap();
                let articles = Article::get_page(&mut client, page, true)?;
                drop(client);

                let json = serde_json::to_string(&articles)
                    .map_err(|_| ResponseError::InternalServerError)?;

                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());

                Ok((200, json.into_bytes(), headers))
            }
        }
    }
}

/// The fields shared by the new and edit forms.
struct ArticleForm {
    slug: String,
    title: String,
    content: String,
    short: Option<String>,
}

impl ArticleForm {
    fn parse(body: &[u8]) -> Result<ArticleForm, ResponseError> {
        let mut form = parse_form(body);

        let mut required = |name: &str, message: &'static str| {
            form.remove(name)
                .filter(|value| !value.trim().is_empty())
                .ok_or(ResponseError::BadRequest(message))
        };

        let slug = required("slug", "Slug is required.")?;
        let title = required("title", "Title is required.")?;
        let content = required("content", "Content is required.")?;
        let short = form.remove("short").filter(|value| !value.trim().is_empty());

        // It becomes part of the article's URL.
        if slug.contains('/') || slug.chars().any(char::is_whitespace) {
            return Err(ResponseError::BadRequest("Slug can't contain / or spaces."));
        }

        Ok(ArticleForm {
            slug,
            title,
            content,
            short,
        })
    }
}

fn parse_form(body: &[u8]) -> HashMap<String, String> {
    form_urlencoded::parse(body).into_owned().collect()
}

fn parse_id(form: &HashMap<String, String>) -> Result<Uuid, ResponseError> {
    let id = form
        .get("id")
        .ok_or(ResponseError::BadRequest("Id is required."))?;
    Uuid::try_parse(id).map_err(|_| ResponseError::BadRequest("Invalid id."))
}

fn redirect(location: String) -> (u16, Vec<u8>, Headers) {
    let mut headers = HashMap::new();
    headers.insert("Location".to_string(), location);
    (303, b"".to_vec(), headers)
}
