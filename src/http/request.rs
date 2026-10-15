use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    time::Instant,
};

use crate::{
    constants::{MAX_BODY_SIZE, MAX_HEADERS, MAX_LINE_SIZE, REQUEST_TIMEOUT},
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
    let deadline = Instant::now() + REQUEST_TIMEOUT;

    let request_line = read_line(reader, deadline)?;

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
        let line = read_line(reader, deadline)?;
        if line.is_empty() {
            break;
        }

        if headers.len() >= MAX_HEADERS {
            return Err(ResponseError::BadRequest("Too many headers."));
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
    let mut filled = 0;
    while filled < length {
        check_deadline(deadline)?;

        let read = reader.read(&mut body[filled..])?;
        if read == 0 {
            return Err(ResponseError::BadRequest("Unexpected end of request"));
        }
        filled += read;
    }

    Ok(Request {
        method,
        path,
        headers,
        cookies,
        body,
        context: Context { session: None },
    })
}

fn read_line(
    reader: &mut BufReader<&TcpStream>,
    deadline: Instant,
) -> Result<String, ResponseError> {
    let mut line = Vec::new();

    loop {
        check_deadline(deadline)?;

        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(ResponseError::BadRequest("Unexpected end of request"));
        }

        let (chunk, found_newline) = match available.iter().position(|&byte| byte == b'\n') {
            Some(index) => (&available[..=index], true),
            None => (available, false),
        };
        line.extend_from_slice(chunk);
        let consumed = chunk.len();
        reader.consume(consumed);

        if line.len() > MAX_LINE_SIZE {
            return Err(ResponseError::BadRequest("Line too long."));
        }
        if found_newline {
            break;
        }
    }

    let line = String::from_utf8(line).map_err(|_| ResponseError::BadRequest("Invalid UTF-8."))?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn check_deadline(deadline: Instant) -> Result<(), ResponseError> {
    if Instant::now() > deadline {
        return Err(ResponseError::BadRequest("Request took too long."));
    }
    Ok(())
}
