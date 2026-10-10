mod http;
mod threading;

use std::net::TcpListener;
use std::num::NonZero;
use std::thread::available_parallelism;

use crate::threading::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let pool = ThreadPool::new(
        available_parallelism()
            .unwrap_or(NonZero::new(4).unwrap())
            .get()
            * 2,
    );

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| http::handle_connection(stream));
    }
}
