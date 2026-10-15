use std::time::Duration;

pub const MAX_BODY_SIZE: usize = 1024 * 1024;
pub const MAX_LINE_SIZE: usize = 8 * 1024;
pub const MAX_HEADERS: usize = 100;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const SESSION_LENGTH: Duration = Duration::new(60 * 60 * 24 * 30, 0);
