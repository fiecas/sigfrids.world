use std::time::SystemTime;

use postgres::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::http::error::ResponseError;

const PAGE_SIZE: i64 = 50;

/// A taken slug is the only unique column a caller can collide with.
fn conflict_on_unique_violation(error: postgres::Error) -> ResponseError {
    match error.code() {
        Some(&postgres::error::SqlState::UNIQUE_VIOLATION) => ResponseError::Conflict,
        _ => ResponseError::from(error),
    }
}

#[derive(Serialize)]
pub struct Article {
    pub id: Uuid,
    pub author_id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub short: Option<String>,
    pub published_at: Option<SystemTime>,
    pub created_at: SystemTime,
    pub edited_at: Option<SystemTime>,
}

impl Article {
    pub fn new(
        client: &mut Client,
        author_id: Uuid,
        slug: &str,
        title: &str,
        content: &str,
        short: Option<&str>,
    ) -> Result<Article, ResponseError> {
        let row = client
            .query_one(
                include_str!("./sql/insert_returning_rest.sql"),
                &[&author_id, &slug, &title, &content, &short],
            )
            .map_err(conflict_on_unique_violation)?;

        let id = row.get("id");
        let created_at = row.get("created_at");

        Ok(Article {
            id,
            author_id,
            slug: slug.to_owned(),
            title: title.to_owned(),
            content: content.to_owned(),
            short: short.map(|s| s.to_owned()),
            published_at: None,
            created_at,
            edited_at: None,
        })
    }

    pub fn from_id(client: &mut Client, id: Uuid) -> Result<Option<Article>, ResponseError> {
        let row = match client.query_opt(include_str!("./sql/select_all_where_id.sql"), &[&id])? {
            Some(row) => row,
            None => return Ok(None),
        };

        let author_id = row.get("author_id");
        let slug = row.get("slug");
        let title = row.get("title");
        let content = row.get("content");
        let short = row.get("short");
        let published_at = row.get("published_at");
        let created_at = row.get("created_at");
        let edited_at = row.get("edited_at");

        Ok(Some(Article {
            id,
            author_id,
            slug,
            title,
            content,
            short,
            published_at,
            created_at,
            edited_at,
        }))
    }

    pub fn from_slug(client: &mut Client, slug: String) -> Result<Option<Article>, ResponseError> {
        let row =
            match client.query_opt(include_str!("./sql/select_all_where_slug.sql"), &[&slug])? {
                Some(row) => row,
                None => return Ok(None),
            };

        let id = row.get("id");
        let author_id = row.get("author_id");
        let title = row.get("title");
        let content = row.get("content");
        let short = row.get("short");
        let published_at = row.get("published_at");
        let created_at = row.get("created_at");
        let edited_at = row.get("edited_at");

        Ok(Some(Article {
            id,
            author_id,
            slug,
            title,
            content,
            short,
            published_at,
            created_at,
            edited_at,
        }))
    }

    /// Returns false if no article has this id.
    pub fn update(
        client: &mut Client,
        id: Uuid,
        slug: &str,
        title: &str,
        content: &str,
        short: Option<&str>,
    ) -> Result<bool, ResponseError> {
        let updated = client
            .execute(
                include_str!("./sql/update_where_id.sql"),
                &[&id, &slug, &title, &content, &short],
            )
            .map_err(conflict_on_unique_violation)?;

        Ok(updated > 0)
    }

    /// Newest first. Drafts are left out when `published_only` is set.
    pub fn get_page(
        client: &mut Client,
        page: u32,
        published_only: bool,
    ) -> Result<Vec<Article>, ResponseError> {
        let offset = i64::from(page) * PAGE_SIZE;

        let row = client
            .query(
                include_str!("./sql/select_wo_content_by_page.sql"),
                &[&published_only, &PAGE_SIZE, &offset],
            )?
            .iter()
            .map(|row| Article {
                id: row.get("id"),
                author_id: row.get("author_id"),
                slug: row.get("slug"),
                title: row.get("title"),
                content: String::new(),
                short: row.get("short"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                edited_at: row.get("edited_at"),
            })
            .collect();

        Ok(row)
    }

    pub fn set_published(
        client: &mut Client,
        id: Uuid,
        published: bool,
    ) -> Result<bool, ResponseError> {
        let updated = client.execute(
            include_str!("./sql/update_published_at_where_id.sql"),
            &[&id, &published],
        )?;

        Ok(updated > 0)
    }

    pub fn delete(client: &mut Client, id: Uuid) -> Result<(), ResponseError> {
        client.execute(include_str!("./sql/delete_where_id.sql"), &[&id])?;

        Ok(())
    }
}
