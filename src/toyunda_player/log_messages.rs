use std::sync::RwLock;

use chrono::{DateTime,Local};

use log::LogLevel;

pub struct LogMessage {
    pub level : LogLevel,
    pub time : DateTime<Local>,
    pub msg : String
}

lazy_static! {
    pub static ref LOG_MESSAGES : RwLock<Vec<LogMessage>> = RwLock::new(Vec::new());
}
