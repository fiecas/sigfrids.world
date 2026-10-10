mod http;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        http::handle_connection(stream);
    }
}
