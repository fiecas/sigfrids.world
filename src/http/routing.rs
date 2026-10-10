use std::collections::HashMap;

use crate::http::error::ResponseError;

pub trait Route {
    fn from_path(path: &str) -> Result<impl Route, ResponseError>;
    fn handle(&self) -> Result<(&str, HashMap<&str, &str>), ResponseError>;
}

#[repr(u16)]
pub enum RootRoute {
    Root,
    Api(ApiRoute),
}

#[allow(refining_impl_trait)]
impl Route for RootRoute {
    fn from_path(path: &str) -> Result<RootRoute, ResponseError> {
        println!("{}", path);
        match path {
            "/" => Ok(RootRoute::Root),
            _ => Err(ResponseError::NotFound),
        }
    }

    fn handle(&self) -> Result<(&str, HashMap<&str, &str>), ResponseError> {
        match self {
            RootRoute::Root => Ok(("root", HashMap::new())),
            RootRoute::Api(api_route) => api_route.handle(),
        }
    }
}

pub enum ApiRoute {}
impl Route for ApiRoute {
    fn from_path(path: &str) -> Result<impl Route, ResponseError> {
        println!("{}", path);
        match path {
            "/" => Ok(RootRoute::Root),
            _ => Err(ResponseError::NotFound),
        }
    }

    fn handle(&self) -> Result<(&str, HashMap<&str, &str>), ResponseError> {
        match self {
            _ => Err(ResponseError::NotFound),
        }
    }
}
