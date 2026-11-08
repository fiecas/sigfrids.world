use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use postgres::Client;

use crate::{
    auth_client,
    db::{sessions::Session, users::User},
    http::{error::ResponseError, request::Request, routing::Headers},
};

/// Where auth sends the browser after login: swaps the code for the user,
/// mirrors the user locally and starts our own session.
pub fn handle(
    request: &mut Request,
    db: Arc<Mutex<Client>>,
) -> Result<(u16, Vec<u8>, Headers), ResponseError> {
    if request.method != "GET" {
        return Err(ResponseError::MethodNotAllowed(vec!["GET"]));
    }

    let code = request
        .query
        .get("code")
        .ok_or(ResponseError::BadRequest("Code is required."))?;

    let user = auth_client::exchange_code(code)?;

    let (session, token) = {
        let mut client = db.lock().unwrap();
        let mut transaction = client.transaction()?;
        User::upsert(&mut transaction, &user)?;
        let session = Session::new(&mut transaction, user.id)?;
        transaction.commit()?;
        session
    };

    let mut headers = HashMap::new();
    headers.insert("Set-Cookie".to_string(), session.to_cookie(&token)?);
    headers.insert("Location".to_string(), "/app".to_string());

    Ok((303, Vec::new(), headers))
}
