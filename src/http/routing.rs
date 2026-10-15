use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use postgres::Client;

use crate::http::{error::ResponseError, request::Request};

pub type Headers = HashMap<String, String>;

pub trait Route: Sized {
    fn from_path(path: &str) -> Result<Self, ResponseError>;
    fn handle(
        &self,
        request: &mut Request,
        db: Arc<Mutex<Client>>,
    ) -> Result<(u16, String, Headers), ResponseError>;
}
