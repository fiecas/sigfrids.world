use pulldown_cmark::{Parser, html};
use std::collections::HashMap;

use crate::http::error::ResponseError;

pub trait Route {
    fn from_path(path: &str) -> Result<impl Route, ResponseError>;
    fn handle(&self) -> Result<(String, HashMap<&str, &str>), ResponseError>;
}

#[repr(u16)]
pub enum RootRoute {
    Root,
    _Api(ApiRoute),
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
                let content = parse_markdown(include_str!("../../content/home.md"));
                Ok((content, HashMap::new()))
            }
            RootRoute::_Api(api_route) => api_route.handle(),
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

fn parse_markdown(markdown: &str) -> String {
    let template = include_str!("../../templates/default.html");

    let mut html = String::new();
    let parser = Parser::new(markdown);
    html::push_html(&mut html, parser);

    template.replace("<content />", &html)
}
