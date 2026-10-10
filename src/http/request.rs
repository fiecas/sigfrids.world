use std::{
    collections::HashMap,
    io::{BufReader, Error, Lines},
    net::TcpStream,
};

use crate::http::error::ResponseError;

impl From<Error> for ResponseError {
    fn from(_: Error) -> Self {
        ResponseError::BadRequest
    }
}

pub struct Request {
    pub _method: String,
    pub path: String,
    pub _headers: HashMap<String, String>,
}

pub fn parse(mut lines: Lines<BufReader<&TcpStream>>) -> Result<Request, ResponseError> {
    let request_line = lines.next().ok_or(ResponseError::BadRequest)??;

    let mut request_line_split = request_line.split(" ");
    let _method = request_line_split
        .next()
        .ok_or(ResponseError::BadRequest)?
        .to_string();
    let path = request_line_split
        .next()
        .ok_or(ResponseError::BadRequest)?
        .to_string();

    let mut _headers: HashMap<String, String> = HashMap::new();

    for result in lines {
        let line = result?;
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, ":");
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            _headers.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            return Err(ResponseError::BadRequest);
        }
    }

    Ok(Request {
        _method,
        path,
        _headers,
    })
}
