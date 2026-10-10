mod db;
mod http;
mod threading;

use std::env;
use std::net::{TcpListener, TcpStream};
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::available_parallelism;

use postgres::NoTls;

use crate::db::migrations::migrate;
use crate::threading::ThreadPool;

const PORT: u16 = 7878;

fn main() {
    let db_url = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL is required.");

    let mut client = postgres::Client::connect(&db_url, NoTls).unwrap();

    migrate(&mut client).unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        flag.store(false, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", PORT));
    })
    .expect("Error setting Ctrl-C handler");

    let listener = TcpListener::bind(("0.0.0.0", PORT)).unwrap();
    let pool = ThreadPool::new(
        available_parallelism()
            .unwrap_or(NonZero::new(4).unwrap())
            .get()
            * 2,
    );

    for stream in listener.incoming() {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        };

        pool.execute(|| http::handle_connection(stream));
    }

    println!("Shutting down...");
}
