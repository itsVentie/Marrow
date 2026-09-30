use std::time::Duration;

pub const RELAY_ADDR: &str = "0.0.0.0:9000";
pub const CHANNEL_BUFFER: usize = 512;
pub const OFFLINE_TTL: Duration = Duration::from_secs(300);
pub const MAX_OFFLINE_QUEUE_LEN: usize = 100;
