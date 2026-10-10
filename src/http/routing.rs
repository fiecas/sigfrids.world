use pulldown_cmark::{Parser, html};
use std::{collections::HashMap, fs};

use crate::http::error::ResponseError;

pub trait Route {
    fn from_path(path: &str) -> Result<impl Route, ResponseError>;
    fn handle(&self) -> Result<(String, HashMap<&str, &str>), ResponseError>;
}

#[repr(u16)]
pub enum RootRoute {
    Root,
    Api(ApiRoute),
}

#[allow(refining_impl_trait)]
impl Route for RootRoute {
    fn from_path(path: &str) -> Result<RootRoute, ResponseError> {
        match path {
            "/" => Ok(RootRoute::Root),
            _ => Err(ResponseError::NotFound),
        }
    }

    fn handle(&self) -> Result<(String, HashMap<&str, &str>), ResponseError> {
        match self {
            RootRoute::Root => {
                let content = match handle_markdown("./content/home.md") {
                    Ok(content) => content,
                    Err(error) => return Err(error),
                };
                Ok((content, HashMap::new()))
            }
            RootRoute::Api(api_route) => api_route.handle(),
        }
    }
}

pub enum ApiRoute {}
impl Route for ApiRoute {
    fn from_path(path: &str) -> Result<impl Route, ResponseError> {
        match path {
            "/" => Ok(RootRoute::Root),
            _ => Err(ResponseError::NotFound),
        }
    }

    fn handle(&self) -> Result<(String, HashMap<&str, &str>), ResponseError> {
        match self {
            _ => Err(ResponseError::NotFound),
        }
    }
}

fn handle_file(file_path: &str) -> Result<String, ResponseError> {
    fs::read_to_string(file_path).or(Err(ResponseError::InternalServerError))
}

fn handle_markdown(markdown_path: &str) -> Result<String, ResponseError> {
    let template = match fs::read_to_string("./templates/default.html") {
        Ok(content) => content,
        Err(_) => return Err(ResponseError::InternalServerError),
    };

    let markdown = match fs::read_to_string(markdown_path) {
        Ok(content) => content,
        Err(_) => return Err(ResponseError::InternalServerError),
    };

    let mut html = String::new();
    let parser = Parser::new(&markdown);
    html::push_html(&mut html, parser);

    return Ok(template.replace("<content />", &html));
}
