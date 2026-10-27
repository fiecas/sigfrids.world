use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    db::articles::Article,
    http::{error::ResponseError, request::Request, routing::Headers},
    utils::{escape_html, format_time, markdown_to_html, with_default_template},
};

use postgres::Client;

pub enum ArticlesRoute {
    Article(String),
}

impl super::Route for ArticlesRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        match path.strip_prefix("/articles/") {
            Some(slug) if !slug.is_empty() && !slug.contains('/') => {
                Ok(ArticlesRoute::Article(slug.to_string()))
            }
            _ => Err(ResponseError::NotFound),
        }
    }

    fn handle(
        &self,
        _request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        match self {
            ArticlesRoute::Article(slug) => {
                let mut client = db.lock().unwrap();
                let article = Article::from_slug(&mut client, slug.clone())?
                    .ok_or(ResponseError::NotFound)?;
                drop(client);

                // Drafts look the same as articles that don't exist.
                let published_at = article.published_at.ok_or(ResponseError::NotFound)?;

                Ok((
                    200,
                    with_default_template(include_str!("../../../content/article.html"))
                        .replace("{{title}}", &escape_html(&article.title))
                        .replace("{{published_at}}", &format_time(published_at))
                        // Last, so placeholders written in the article stay as they are.
                        .replace("{{content}}", &markdown_to_html(&article.content))
                        .into_bytes(),
                    HashMap::new(),
                ))
            }
        }
    }
}
