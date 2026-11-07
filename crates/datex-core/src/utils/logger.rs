use core::cell::RefCell;
use datex_macros_internal::Datex;
use crate::values::core_values::sender::Sender;
use crate::prelude::*;
use crate::std_sync::Mutex;


#[derive(Datex, Clone, Copy)]
#[datex(structural)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}


#[derive(Datex, Clone)]
#[datex(structural)]
pub struct LogMessage {
    level: LogLevel,
    module: String,
    tag: Option<String>,
    message: String,
}

impl LogMessage {
    pub fn new(level: LogLevel, module: String, tag: Option<String>, message: String) -> Self {
        Self { level, module, tag, message }
    }
}

// FIXME: better solution, should be thread_local here, but not supported on esp32 target
struct SingleContext<T>(T);
/// SAFETY: only works for single threaded
unsafe impl<T> Sync for SingleContext<T> {}

static SENDERS: SingleContext<RefCell<Vec<Sender<LogMessage>>>> =
    SingleContext(RefCell::new(Vec::new()));


/// Registers a sender which is called for any global log
pub fn register_log_sender(sender: Sender<LogMessage>) {
    SENDERS.0.borrow_mut().push(sender);

}

pub fn log_message(message: LogMessage) {
    let senders = &mut SENDERS.0.borrow_mut();

    // send message on all senders, remove all closed senders
    senders.retain_mut(|sender| {
        sender.send(message.clone()).is_ok()
    });
}

pub fn string_with_maybe_tag(string: String, tag: &Option<String>) -> String {
    if let Some(tag) = tag {
        format!("[{}] {}", tag, string)
    } else {
        string
    }
}

macro _log($level:expr, $tag:expr, $($arg:tt)+) {
    let string = format!($($arg)+);
    let string_with_tag = string_with_maybe_tag(string.clone(), &$tag);

    log::log!(log::Level::from($level), "{}", string_with_tag);
    log_message(LogMessage::new($level, module_path!().to_string(), $tag, string));
}


pub macro log_info($($arg:tt)+) {
    _log!(LogLevel::Info, None, $($arg)+);
}
pub macro log_info_with_tag($tag:tt, $($arg:tt)+) {
    _log!(LogLevel::Info, Some($tag.to_string()), $($arg)+);
}

pub macro log_error($($arg:tt)+) {
    _log!(LogLevel::Error, Some($tag.to_string()), $($arg)+);
}
pub macro log_error_with_tag($tag:tt, $($arg:tt)+) {
    _log!(LogLevel::Error, Some($tag.to_string()), $($arg)+);
}

pub macro log_warn($($arg:tt)+) {
    _log!(LogLevel::Warn, Some($tag.to_string()), $($arg)+);
}
pub macro log_warn_with_tag($tag:tt, $($arg:tt)+) {
    _log!(LogLevel::Warn, Some($tag.to_string()), $($arg)+);
}

pub macro log_trace($($arg:tt)+) {
    _log!(LogLevel::Trace, Some($tag.to_string()), $($arg)+);
}
pub macro log_trace_with_tag($tag:tt, $($arg:tt)+) {
    _log!(LogLevel::Trace, Some($tag.to_string()), $($arg)+);
}

pub macro log_debug($($arg:tt)+) {
    _log!(LogLevel::Debug, Some($tag.to_string()), $($arg)+);
}
pub macro log_debug_with_tag($tag:tt, $($arg:tt)+) {
    _log!(LogLevel::Debug, Some($tag.to_string()), $($arg)+);
}