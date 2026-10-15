use postgres::NoTls;

use crate::db::migrations::migrate;

pub mod files;
pub mod migrations;
pub mod sessions;
pub mod users;

pub struct Client {
    pub postgres: postgres::Client,
}

impl Client {
    pub fn connect(connection_str: &str) -> Client {
        let postgres = postgres::Client::connect(connection_str, NoTls).unwrap();

        Client { postgres }
    }

    pub fn migrate(&mut self) {
        migrate(&mut self.postgres).unwrap();
    }
}
