mod constants;
mod db;
mod http;
mod passwords;
mod routes;
mod threading;
mod utils;

use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::num::NonZero;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, available_parallelism};
use std::time::Duration;

use socket2::{Domain, Socket, Type};

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
    let db = Arc::new(Mutex::new(client.postgres));

    let mut postgres = db.lock().unwrap();
    let mut transaction = postgres.transaction().unwrap();

    if User::count(&mut transaction).expect("Could not count users.") == 0 {
        let default_admin_email = env::var("ADMIN_EMAIL")
            .expect("Environment variable ADMIN_EMAIL is required on first startup");
        let default_admin_password = env::var("ADMIN_PASSWORD")
            .expect("Environment variable ADMIN_PASSWORD is required on first startup");

        User::new(
            &mut transaction,
            "admin",
            &default_admin_email,
            "Admin",
            passwords::hash(&default_admin_password)
                .expect("Could not hash admin password.")
                .as_str(),
        )
        .expect("Could not create admin user.");
    }
    transaction.commit().unwrap();
    drop(postgres);

    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || flag.store(false, Ordering::SeqCst))
        .expect("Error setting Ctrl-C handler");

    let listener = bind_listener().expect("Could not bind listener.");
    listener
        .set_nonblocking(true)
        .expect("Could not set listener to non-blocking.");
    let pool = ThreadPool::new(
        available_parallelism()
            .unwrap_or(NonZero::new(4).unwrap())
            .get()
            * 2,
    );

    // Polls instead of blocking so shutdown doesn't depend on a wake-up
    // connection, which could land in another process sharing the port.
    while running.load(Ordering::SeqCst) {
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        };

        // Accepted sockets inherit non-blocking mode on some platforms.
        if let Err(error) = stream.set_nonblocking(false) {
            eprintln!("{}", error);
            continue;
        }

        let db = Arc::clone(&db);

        pool.execute(|| http::handle_connection(stream, db));
    }

    println!("Shutting down...");
}

fn bind_listener() -> io::Result<TcpListener> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    // In dev, lets dev.sh start the new server before stopping the old one.
    socket.set_reuse_port(cfg!(debug_assertions))?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT)).into())?;
    socket.listen(128)?;

    Ok(socket.into())
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
