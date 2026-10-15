use pulldown_cmark::{Parser, html};

pub fn valid_email(email: &str) -> bool {
    email.len() <= 254
        && !email.chars().any(char::is_whitespace)
        && match email.split_once('@') {
            Some((local, domain)) => !local.is_empty() && domain.contains('.'),
            None => false,
        }
}

pub fn valid_username(username: &str) -> bool {
    (3..=32).contains(&username.len())
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn parse_markdown(markdown: &str) -> String {
    let template = include_str!("../templates/default.html");

    let mut html = String::new();
    let parser = Parser::new(markdown);
    html::push_html(&mut html, parser);

    template.replace("<content />", &html)
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect()
}
