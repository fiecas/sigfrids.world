use std::{collections::HashMap, hash::Hash};

use crate::http::error::ResponseError;

pub enum Route {
    Root,
    ApiRoot = 0x100,
}

impl Route {
    pub fn from_path(path: &str) -> Result<Route, ResponseError> {
        println!("{}", path);
        match path {
            "/" => Ok(Route::Root),
            _ => Err(ResponseError::NotFound),
        }
    }

    pub fn handle(&self) -> Result<(&str, HashMap<&str, &str>), ResponseError> {
        match self {
            Route::Root => Ok(("root", HashMap::new())),
            _ => Err(ResponseError::NotFound),
        }
    }
}
