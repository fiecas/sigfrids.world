pub mod error;
pub mod request;
pub mod routing;

use std::{
    collections::HashMap,
    io::{BufReader, prelude::*},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use postgres::Client;

use crate::{
    http::{
        error::ResponseError,
        routing::{Headers, Route},
    },
    routes::RootRoute,
};

pub fn handle_connection(stream: TcpStream, db: Arc<Mutex<Client>>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let (code, content, headers) = match handle_request(&stream, db) {
        Ok(response) => response,
        Err(ResponseError::RequestTimeout) => return,
        Err(error) => {
            let mut headers: Headers = HashMap::new();
            match &error {
                ResponseError::MethodNotAllowed(allowed_methods) => {
                    headers.insert("Allow".to_string(), allowed_methods.join(","));
                }
                _ => {}
            }
            (error.status_code(), error.to_string().into_bytes(), headers)
        }
    };

    return respond(stream, code, content, headers);
}

fn handle_request(
    stream: &TcpStream,
    db: Arc<Mutex<Client>>,
) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
    let buf_reader = &mut BufReader::new(stream);
    let mut request = request::parse(buf_reader)?;

    let route = RootRoute::from_path(request.path.as_str())?;

    Ok(route.handle(&mut request, db)?)
}

fn respond(mut stream: TcpStream, code: u16, content: Vec<u8>, headers: Headers) {
    let status_line = format!("HTTP/1.1 {code}");

    let mut head = format!("{status_line}\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", content.len()));

    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&content);
}
