mod error;
mod request;
mod routing;
use std::{
    io::{BufReader, prelude::*},
    net::TcpStream,
};

use crate::http::routing::Route;

pub fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request = match request::parse(buf_reader.lines()) {
        Ok(request) => request,
        Err(error) => {
            return respond(stream, error as u16, "");
        }
    };

    let route = match Route::from_path(request.path.as_str()) {
        Ok(route) => route,
        Err(error) => {
            let status_text = error.to_string();
            return respond(stream, error as u16, &status_text);
        }
    };

    let (content, headers) = match route.handle() {
        Ok(handle) => handle,
        Err(error) => {
            let status_text = error.to_string();
            return respond(stream, error as u16, &status_text);
        }
    };

    return respond(stream, 200, content);
}

fn respond(mut stream: TcpStream, code: u16, contents: &str) {
    let status_line = format!("HTTP/1.1 {code}");

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
