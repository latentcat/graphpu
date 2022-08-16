use std::{fmt::Display, sync::{Mutex, MutexGuard}};

use chrono::Local;

pub enum MessageLevel {
    Info,
    Warn,
    Error,
}

pub struct Message {
    pub level: MessageLevel,
    pub title: String,
    pub content: String,
    pub time: i64,
}

impl Message {
    fn new(level: MessageLevel, title: &str, content: &str) -> Self {
        Self {
            level,
            title: String::from(title),
            content: String::from(content),
            time: Local::now().timestamp(),
        }
    }

    fn info(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Info, title, content)
    }

    fn warn(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Warn, title, content)
    }

    fn error(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Error, title, content)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_message = match self.level {
            MessageLevel::Info => format!("[INFO] {} {}", self.title, self.content),
            MessageLevel::Warn => format!("[WARN] {} {}", self.title, self.content),
            MessageLevel::Error => format!("[ERROR] {} {}", self.title, self.content),
        };
        f.write_str(&format_message)
    }
}

lazy_static! {
    static ref MESSAGE_LIST: Mutex<Vec<Message>> = Mutex::new(Vec::new());
}

pub fn messenger() -> MutexGuard<'static, Vec<Message>> {
    MESSAGE_LIST.lock().unwrap()
}

pub fn info(title: &str, content: &str) {
    messenger().push(Message::info(title, content));
}

pub fn warn(title: &str, content: &str) {
    messenger().push(Message::warn(title, content));
}

pub fn error(title: &str, content: &str) {
    messenger().push(Message::error(title, content));
}