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
                let content = match handle_file("./templates/default.html") {
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
