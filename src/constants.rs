use std::time::Duration;

pub const MAX_BODY_SIZE: usize = 1024 * 1024;
pub const SESSION_LENGTH: Duration = Duration::new(60 * 60 * 24 * 30, 0);
