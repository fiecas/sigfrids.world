pub mod error;
pub mod request;
pub mod routing;

use std::{
    io::{BufReader, prelude::*},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use postgres::Client;

use crate::{
    http::routing::{Headers, Route},
    routes::RootRoute,
};

pub fn handle_connection(stream: TcpStream, db: Arc<Mutex<Client>>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let buf_reader = &mut BufReader::new(&stream);
    let mut request = match request::parse(buf_reader) {
        Ok(request) => request,
        Err(error) => {
            return respond(stream, error.status_code(), "", Headers::new());
        }
    };

    let route = match RootRoute::from_path(request.path.as_str()) {
        Ok(route) => route,
        Err(error) => {
            let status_text = error.to_string();
            return respond(stream, error.status_code(), &status_text, Headers::new());
        }
    };

    let (code, content, headers) = match route.handle(&mut request, db) {
        Ok(handle) => handle,
        Err(error) => {
            let status_text = error.to_string();
            return respond(stream, error.status_code(), &status_text, Headers::new());
        }
    };

    return respond(stream, code, content.as_str(), headers);
}

fn respond(mut stream: TcpStream, code: u16, contents: &str, headers: Headers) {
    let status_line = format!("HTTP/1.1 {code}");

    let length = contents.len();
    let mut response = format!("{status_line}\r\n");
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str(&format!("Content-Length: {length}\r\n\r\n{contents}"));

    let _ = stream.write_all(response.as_bytes());
}
