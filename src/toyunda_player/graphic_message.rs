use ::toyunda_player::log_messages::*;


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Category {
    Error,
    Warn,
    Announcement,
}

#[derive(Debug)]
pub struct GraphicMessage {
    pub category: Category,
    pub text: String,
}

impl GraphicMessage {
    fn from_log_message(log_message:&LogMessage) -> Option<GraphicMessage> {
        use log::LogLevel;
        let level : Option<Category> = match log_message.level {
            LogLevel::Error => Some(Category::Error),
            LogLevel::Warn => Some(Category::Warn),
            _ => None,
        };
        level.map(|c| GraphicMessage { category: c, text: log_message.msg.clone()})
    }
}

pub fn get_graphic_messages() -> Vec<GraphicMessage> {
    use chrono::{Duration,Local};
    let mut g_messages = Vec::with_capacity(0);
    if let Ok(log_messages) = LOG_MESSAGES.read() {
        let last_log_messages = log_messages.split_at(log_messages.len().saturating_sub(5)).1;
        for log_message in last_log_messages {
            if let Some(graphic_message) = GraphicMessage::from_log_message(log_message) {
                if log_message.time + Duration::seconds(6) > Local::now() {
                    g_messages.push(graphic_message)
                }
            };
        }
    }
    g_messages
}
