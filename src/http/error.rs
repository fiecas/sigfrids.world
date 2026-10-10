use std::fmt::{self};

#[repr(i16)]
pub enum ResponseError {
    BadRequest = 400,
    NotFound = 404,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseError::BadRequest => write!(f, "Bad Request"),
            ResponseError::NotFound => write!(f, "Not Found"),
        }
    }
}
