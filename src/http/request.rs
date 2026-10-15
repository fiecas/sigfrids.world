use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use crate::{
    constants::MAX_BODY_SIZE,
    db::sessions::Session,
    http::{error::ResponseError, routing::Headers},
};

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Headers,
    pub cookies: HashMap<String, String>,
    pub body: Vec<u8>,
    pub context: Context,
}

pub struct Context {
    pub session: Option<Session>,
}

pub fn parse(reader: &mut BufReader<&TcpStream>) -> Result<Request, ResponseError> {
    let request_line = read_line(reader)?;

    let mut request_line_split = request_line.split(" ");
    let method = request_line_split
        .next()
        .ok_or(ResponseError::BadRequest("Request method missing."))?
        .to_string();
    let path = request_line_split
        .next()
        .ok_or(ResponseError::BadRequest("Request path missing."))?
        .to_string();

    let mut headers: HashMap<String, String> = HashMap::new();

    loop {
        let line = read_line(reader)?;
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, ":");
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            return Err(ResponseError::BadRequest("Malformed header."));
        }
    }

    let cookies: HashMap<String, String> = match headers.get("cookie") {
        Some(cookie_header) => cookie_header
            .split(";")
            .map(|cookie_string| match cookie_string.split_once("=") {
                Some((key, value)) => (key.trim().to_string(), value.trim().to_string()),
                None => (cookie_string.to_string(), "".to_string()),
            })
            .collect(),
        None => HashMap::new(),
    };

    let length = match headers.get("content-length") {
        Some(value) => value
            .parse()
            .map_err(|_| ResponseError::BadRequest("Invalid content-length."))?,
        None => 0,
    };
    if length > MAX_BODY_SIZE {
        return Err(ResponseError::PayloadTooLarge);
    }

    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;

    Ok(Request {
        method,
        path,
        headers,
        cookies,
        body,
        context: Context { session: None },
    })
}

fn read_line(reader: &mut BufReader<&TcpStream>) -> Result<String, ResponseError> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Err(ResponseError::BadRequest("Unexpected end of request"));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}
