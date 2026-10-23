mod constants;
mod db;
mod http;
mod passwords;
mod routes;
mod threading;
mod utils;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::num::NonZero;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use std::time::Duration;

use crate::db::Client;
use crate::db::users::User;
use crate::threading::ThreadPool;

const PORT: u16 = 7878;

fn main() {
    if env::args().any(|arg| arg == "--healthcheck") {
        process::exit(healthcheck());
    }

    let db_url = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL is required");

    let mut client = Client::connect(&db_url);
    client.migrate();

    if User::count(&mut client.postgres).expect("Could not count users.") == 0 {
        let default_admin_email = env::var("ADMIN_EMAIL")
            .expect("Environment variable ADMIN_EMAIL is required on first startup");
        let default_admin_password = env::var("ADMIN_PASSWORD")
            .expect("Environment variable ADMIN_PASSWORD is required on first startup");

        User::new(
            &mut client.postgres,
            "admin",
            &default_admin_email,
            "Admin",
            passwords::hash(&default_admin_password)
                .expect("Could not hash admin password.")
                .as_str(),
        )
        .expect("Could not create admin user.");
    }

    let db = Arc::new(Mutex::new(client.postgres));

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

        let db = Arc::clone(&db);

        pool.execute(|| http::handle_connection(stream, db));
    }

    println!("Shutting down...");
}

fn healthcheck() -> i32 {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, PORT));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_secs(2)) else {
        return 1;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));

    if stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return 1;
    }

    let mut status_line = String::new();
    let _ = BufReader::new(stream).read_line(&mut status_line);

    match status_line.split_whitespace().nth(1) {
        Some("200") => 0,
        _ => 1,
    }
}
