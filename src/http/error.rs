use std::fmt::{self, write};

#[repr(i16)]
pub enum ResponseError {
    BadRequest = 400,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseError::BadRequest => write!(f, "Bad Request"),
            ResponseError::Unauthorized => write!(f, "Unauthorized"),
            ResponseError::PaymentRequired => write!(f, "Payment Required"),
            ResponseError::Forbidden => write!(f, "Forbidden"),
            ResponseError::NotFound => write!(f, "Not Found"),
        }
    }
}
