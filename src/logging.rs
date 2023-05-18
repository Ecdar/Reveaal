use crate::ProtobufServer::services::query_response::Information;
use chrono::Local;
use colored::{ColoredString, Colorize};
use log::SetLoggerError;
use once_cell::sync::Lazy;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::thread::ThreadId;

#[cfg(feature = "logging")]
fn colored_level(level: log::Level) -> ColoredString {
    match level {
        log::Level::Error => level.to_string().red(),
        log::Level::Warn => level.to_string().yellow(),
        log::Level::Info => level.to_string().cyan(),
        log::Level::Debug => level.to_string().blue(),
        log::Level::Trace => level.to_string().magenta(),
    }
}

#[cfg(feature = "logging")]
/// Sets up the logging
pub fn setup_logger() -> Result<(), SetLoggerError> {
    env_logger::Builder::from_env(env_logger::Env::default())
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{} {}] - {}",
                Local::now().format("%H:%M:%S").to_string().cyan(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                colored_level(record.level()),
                record.args()
            )
        })
        .try_init()
}

#[macro_export]
macro_rules! msg { //TODO: Maybe format the information when not server
    ($severity:expr, subject: $subject:expr, msg: $msg:expr) => ({
        if $crate::is_server() {
            $crate::logging::__set_info__($crate::logging::__as_information__($severity, $subject, $msg));
        } else {
            //let lvl = $crate::logging::__severity__($severity);
            //log::log!(lvl, "{}", $crate::logging::__as_information__($severity, $subject, $msg));
            println!("{}", $crate::logging::__as_information__($severity, $subject, $msg));
        }
    });


    ($severity:expr, subject: $subject:expr, msg: $($msg:tt)+) => (msg!($severity, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($subject:expr, msg: $msg:expr) => (msg!(0, subject: $subject, msg: $msg));
    ($subject:expr, msg: $($msg:tt)+) => (msg!(0, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($msg:expr) => (msg!(0, subject: "general", msg: $msg.to_string()));
    ($($msg:tt)+) => (msg!(0, subject: "general", msg: format_args!($($msg)+).to_string()));
}

#[doc(hidden)]
pub fn __severity__(severity: i32) -> log::Level {
    match severity {
        0 => log::Level::Info,
        1 => log::Level::Warn,
        _ => unreachable!(),
    }
}

impl Display for Information {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} {}: {}] - {}",
            Local::now().format("%H:%M:%S").to_string().cyan(),
            colored_level(__severity__(self.serverity)),
            self.subject,
            self.message
        )
    }
}

#[doc(hidden)]
static __MESSAGES__: Lazy<Mutex<HashMap<ThreadId, Vec<Information>>>> = Lazy::new(Mutex::default);

#[doc(hidden)]
pub fn __as_information__(severity: i32, subject: &str, message: String) -> Information {
    Information {
        serverity: severity,
        subject: subject.to_string(),
        message,
    }
}

#[doc(hidden)]
pub fn __set_info__(info: Information) {
    match __MESSAGES__.lock().unwrap().entry(thread::current().id()) {
        Entry::Occupied(mut o) => o.get_mut().push(info),
        Entry::Vacant(v) => {
            v.insert(vec![info]);
        }
    };
}

//static mut TEMP: i32 = 10;

/// Function to get information messages.
/// ### Info
/// Will always return `None` when Reveaal is run through the CLI, only use as server.
pub fn get_messages() -> Vec<Information> {
    //println!("{:?}", thread::current().name());
    /*
    unsafe {
        println!("{}", TEMP);
    }
    thread::spawn(|| unsafe {
        TEMP = 20;
        println!("{}", TEMP);
    });
    thread::sleep(Duration::from_millis(1000));
    unsafe {
        println!("{}", TEMP);
    }
    */

    __MESSAGES__
        .lock()
        .unwrap()
        .remove(&thread::current().id())
        .unwrap_or_default()
}
