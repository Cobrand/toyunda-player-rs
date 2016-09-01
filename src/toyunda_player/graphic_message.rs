use std::time::Instant;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Category {
    Error,
    Announcement,
    Info,
    Warn,
}

#[derive(Debug)]
pub struct GraphicMessage {
    pub up_until: Instant,
    pub category: Category,
    pub text: String,
}
