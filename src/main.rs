mod constants;
mod db;
mod http;
mod passwords;
mod routes;
mod threading;
mod utils;

use std::env;
use std::net::{TcpListener, TcpStream};
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;

use crate::db::Client;
use crate::db::users::User;
use crate::threading::ThreadPool;

const PORT: u16 = 7878;

fn main() {
    let db_url = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL is required");

    let mut client = Client::connect(&db_url);
    client.migrate();

    if User::count(&mut client.postgres).expect("Could not count users.") == 0 {
        let default_admin_email = env::var("ADMIN_EMAIL")
            .expect("Environment variable ADMIN_EMAIL is required on first startup");
        let default_admin_password = env::var("ADMIN_PASSWORD")
            .expect("Environment variable ADMIN_EMAIL is required on first startup");

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
