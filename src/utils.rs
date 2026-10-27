use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn with_default_template(content: &str) -> String {
    let template = include_str!("../templates/default.html");
    template.replace("<content />", &content)
}

pub fn parse_markdown(markdown: &str) -> String {
    with_default_template(&markdown_to_html(markdown))
}

/// Markdown as an HTML fragment, without the page template around it.
pub fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let parser = Parser::new(markdown);
    html::push_html(&mut html, parser);

    html
}

/// Makes text safe to put inside HTML, including attribute values.
pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            c => escaped.push(c),
        }
    }
    escaped
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

pub fn format_time(time: SystemTime) -> String {
    let secs = time.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (hour, minute) = (rest / 3600, rest % 3600 / 60);

    // Howard Hinnant's civil_from_days, for dates after 1970
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };

    format!("{hour:02}:{minute:02} {day:02}/{month:02}/{year}")
}
