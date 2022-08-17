use std::{fmt::Display, sync::{Mutex, MutexGuard}};

use chrono::{DateTime, Local, Timelike, Utc};

use strum::Display;

#[derive(Display, PartialEq)]
pub enum MessageLevel {
    Info,
    Warning,
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
            time: Utc::now().timestamp(),
        }
    }

    fn info(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Info, title, content)
    }

    fn warning(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Warning, title, content)
    }

    fn error(title: &str, content: &str) -> Self {
        Self::new(MessageLevel::Error, title, content)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_message = match self.level {
            MessageLevel::Info => format!("[INFO]  {}  {}", self.title, self.content),
            MessageLevel::Warning => format!("[WARNING]  {}  {}", self.title, self.content),
            MessageLevel::Error => format!("[ERROR]  {}  {}", self.title, self.content),
        }
            .replace("\n", "  ");
        f.write_str(&format_message)
    }
}

impl Message {
    pub fn display_time(&self) -> String {
        let time = chrono::NaiveDateTime::from_timestamp(self.time, 0);
        let time: DateTime<Local> = DateTime::from_utc(time, Local::now().offset().to_owned());
        let time_text = format!("{:02}:{:02}:{:02}", time.hour(), time.minute(), time.second());

        time_text

    }
}

lazy_static! {
    static ref MESSAGE_LIST: Mutex<Vec<Message>> = Mutex::new(Vec::new());
}

pub fn messenger() -> MutexGuard<'static, Vec<Message>> {
    MESSAGE_LIST.lock().unwrap()
}

pub fn message_info(title: &str, content: &str) {
    messenger().push(Message::info(title, content));
}

pub fn message_warning(title: &str, content: &str) {
    messenger().push(Message::warning(title, content));
}

pub fn message_error(title: &str, content: &str) {
    messenger().push(Message::error(title, content));
}