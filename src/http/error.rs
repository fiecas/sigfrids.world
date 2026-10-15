use std::{
    fmt::{self},
    io,
    string::FromUtf8Error,
    time::SystemTimeError,
};

#[repr(u16)]
pub enum ResponseError {
    BadRequest(&'static str) = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Conflict = 409,
    PayloadTooLarge = 413,
    InternalServerError = 500,
}

impl ResponseError {
    pub fn status_code(&self) -> u16 {
        match self {
            ResponseError::BadRequest(_) => 400,
            ResponseError::Unauthorized => 401,
            ResponseError::Forbidden => 403,
            ResponseError::NotFound => 404,
            ResponseError::Conflict => 409,
            ResponseError::PayloadTooLarge => 413,
            ResponseError::InternalServerError => 500,
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseError::BadRequest(msg) => {
                write!(f, "Bad Request: {}", msg)
            }
            ResponseError::Unauthorized => write!(f, "Unauthorized"),
            ResponseError::Forbidden => write!(f, "Forbidden"),
            ResponseError::NotFound => write!(f, "Not Found"),
            ResponseError::Conflict => write!(f, "Conflict"),
            ResponseError::PayloadTooLarge => write!(f, "Payload Too Large"),
            ResponseError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

impl From<io::Error> for ResponseError {
    fn from(error: io::Error) -> Self {
        eprintln!("{}", error);

        ResponseError::InternalServerError
    }
}

impl From<postgres::Error> for ResponseError {
    fn from(error: postgres::Error) -> Self {
        eprintln!("{:?}", error);

        ResponseError::InternalServerError
    }
}

impl From<getrandom::Error> for ResponseError {
    fn from(error: getrandom::Error) -> Self {
        eprintln!("{}", error);

        ResponseError::InternalServerError
    }
}

impl From<FromUtf8Error> for ResponseError {
    fn from(error: FromUtf8Error) -> Self {
        eprintln!("{}", error);

        ResponseError::InternalServerError
    }
}

impl From<SystemTimeError> for ResponseError {
    fn from(error: SystemTimeError) -> Self {
        eprintln!("{}", error);

        ResponseError::InternalServerError
    }
}

impl From<uuid::Error> for ResponseError {
    fn from(error: uuid::Error) -> Self {
        eprintln!("{}", error);

        ResponseError::InternalServerError
    }
}
