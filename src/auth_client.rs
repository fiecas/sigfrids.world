//! Talks to auth.fiecas.com, which owns accounts and passwords.
//!
//! Login is a redirect: we send the browser to auth with our callback URL,
//! auth sends it back with a one-time `code`, and we trade that code for the
//! user over a server-to-server request.

use std::{collections::HashMap, env, time::Duration};

use uuid::Uuid;

use crate::{
    db::users::{Role, User},
    http::{error::ResponseError, routing::Headers},
};

/// Fails fast at startup if the auth settings are missing.
pub fn check_config() {
    for name in ["AUTH_URL", "AUTH_CLIENT_SECRET", "PUBLIC_URL"] {
        env::var(name).unwrap_or_else(|_| panic!("Environment variable {name} is required"));
    }
}

fn var(name: &str) -> Result<String, ResponseError> {
    env::var(name).map_err(|_| ResponseError::InternalServerError)
}

fn auth_url() -> Result<String, ResponseError> {
    Ok(var("AUTH_URL")?.trim_end_matches('/').to_string())
}

/// Where auth sends people back to after logging in.
fn callback_url() -> Result<String, ResponseError> {
    Ok(format!(
        "{}/auth/callback",
        var("PUBLIC_URL")?.trim_end_matches('/')
    ))
}

/// 303 to auth's `/login` or `/signup`, coming back to our callback.
pub fn redirect_to_auth(page: &str) -> (u16, Vec<u8>, Headers) {
    let location = (|| -> Result<String, ResponseError> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("redirect", &callback_url()?)
            .finish();
        Ok(format!("{}/{page}?{query}", auth_url()?))
    })();

    match location {
        Ok(location) => {
            let mut headers = HashMap::new();
            headers.insert("Location".to_string(), location);
            (303, Vec::new(), headers)
        }
        Err(error) => (error.status_code(), error.to_string().into_bytes(), HashMap::new()),
    }
}

/// Trades the one-time code from the callback for the user it belongs to.
pub fn exchange_code(code: &str) -> Result<User, ResponseError> {
    let body = form_urlencoded::Serializer::new(String::new())
        .append_pair("code", code)
        .append_pair("redirect_uri", &callback_url()?)
        .finish();

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .into();

    let mut response = agent
        .post(format!("{}/api/token", auth_url()?))
        .header("Authorization", format!("Bearer {}", var("AUTH_CLIENT_SECRET")?))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(body)
        .map_err(|error| match error {
            // Auth rejected the code (unknown, expired or already used).
            ureq::Error::StatusCode(400 | 401) => ResponseError::Unauthorized,
            error => {
                eprintln!("auth token request failed: {error}");
                ResponseError::InternalServerError
            }
        })?;

    let text = response
        .body_mut()
        .read_to_string()
        .map_err(|_| ResponseError::InternalServerError)?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| ResponseError::InternalServerError)?;

    let field = |name: &str| json[name].as_str().ok_or(ResponseError::InternalServerError);

    Ok(User {
        id: Uuid::parse_str(field("id")?)?,
        username: field("username")?.to_string(),
        email: field("email")?.to_string(),
        display_name: field("display_name")?.to_string(),
        role: Role::from_text(field("role")?),
    })
}
