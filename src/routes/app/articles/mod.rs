use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    db::articles::Article,
    http::{error::ResponseError, request::Request, routing::Headers},
    routes::parse_page,
    utils::{escape_html, with_default_template},
};

use postgres::Client;
use uuid::Uuid;

pub enum ArticlesRoute {
    Root,
    New,
    Edit,
}

impl super::super::Route for ArticlesRoute {
    fn from_path(path: &str) -> Result<Self, ResponseError> {
        Ok(match path {
            "/app/articles" => ArticlesRoute::Root,
            "/app/articles/new" => ArticlesRoute::New,
            "/app/articles/edit" => ArticlesRoute::Edit,
            _ => return Err(ResponseError::NotFound),
        })
    }

    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
        match self {
            ArticlesRoute::Root => {
                let page = parse_page(request)?;

                let mut client = db.lock().unwrap();
                let articles = Article::get_page(&mut client, page, false)?;
                drop(client);

                Ok((
                    200,
                    with_default_template(include_str!(
                        "../../../../content/app/articles/index.html"
                    ))
                    .replace(
                        "<articles />",
                        articles
                            .iter()
                            .map(|a| {
                                format!(
                                    "<a href=\"/app/articles/edit?id={}\">{}</a>{}",
                                    a.id,
                                    escape_html(&a.title),
                                    if a.published_at.is_some() { "" } else { " (draft)" }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                            .as_str(),
                    )
                    .into_bytes(),
                    HashMap::new(),
                ))
            }
            ArticlesRoute::New => Ok((
                200,
                with_default_template(include_str!("../../../../content/app/articles/new.html"))
                    .into_bytes(),
                HashMap::new(),
            )),
            ArticlesRoute::Edit => {
                let id = Uuid::try_parse(
                    request
                        .query
                        .get("id")
                        .ok_or(ResponseError::NotFound)?
                        .as_str(),
                )
                .map_err(|_| ResponseError::NotFound)?;

                let mut client = db.lock().unwrap();
                let article = Article::from_id(&mut client, id)?.ok_or(ResponseError::NotFound)?;
                drop(client);

                let published = article.published_at.is_some();

                Ok((
                    200,
                    with_default_template(include_str!(
                        "../../../../content/app/articles/edit.html"
                    ))
                    // Before the article's own text, so a placeholder written in
                    // an article isn't replaced.
                    .replace("{{id}}", &article.id.to_string())
                    .replace("{{status}}", if published { "published" } else { "draft" })
                    .replace("{{publish_value}}", if published { "false" } else { "true" })
                    .replace("{{publish_label}}", if published { "make draft" } else { "publish" })
                    .replace("{{view_link}}", &if published {
                        format!(
                            "<a href=\"/articles/{}\">view</a>",
                            escape_html(&article.slug)
                        )
                    } else {
                        String::new()
                    })
                    .replace("{{slug}}", &escape_html(&article.slug))
                    .replace("{{title}}", &escape_html(&article.title))
                    .replace("{{short}}", &escape_html(article.short.as_deref().unwrap_or("")))
                    .replace("{{content}}", &escape_html(&article.content))
                    .into_bytes(),
                    HashMap::new(),
                ))
            }
        }
    }
}
