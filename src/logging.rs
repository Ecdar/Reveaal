use crate::ProtobufServer::services::query_response::Information;
use chrono::Local;
use colored::{ColoredString, Colorize};
use log::SetLoggerError;
use once_cell::sync::Lazy;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::thread::ThreadId;
//use std::time::Duration;

#[cfg(feature = "logging")]
/// Sets up the logging
pub fn setup_logger() -> Result<(), SetLoggerError> {
    fn colored_level(level: log::Level) -> ColoredString {
        match level {
            log::Level::Error => level.to_string().red(),
            log::Level::Warn => level.to_string().yellow(),
            log::Level::Info => level.to_string().cyan(),
            log::Level::Debug => level.to_string().blue(),
            log::Level::Trace => level.to_string().magenta(),
        }
    }

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
macro_rules! msg { //TODO: Check for server or not; if server, save in table, otherwise use info (for now) --- thread::current().id()
    ($severity:expr, subject: $subject:expr, msg: $msg:expr) => ($crate::logging::__set_info__($severity, $subject, $msg));


    ($severity:expr, subject: $subject:expr, msg: $($msg:tt)+) => (msg!($severity, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($subject:expr, msg: $msg:expr) => (msg!(0, subject: $subject, msg: $msg));
    ($subject:expr, msg: $($msg:tt)+) => (msg!(0, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($msg:expr) => (msg!(0, subject: "general", msg: $msg.to_string()));
    ($($msg:tt)+) => (msg!(0, subject: "general", msg: format_args!($($msg)+).to_string()));
}

#[doc(hidden)]
static __MESSAGES__: Lazy<Mutex<HashMap<ThreadId, Vec<Information>>>> = Lazy::new(Mutex::default);

/// Gets messages saved for other clients (through gRPC)
#[doc(hidden)]
pub fn __set_info__(severity: i32, subject: &str, message: String) {
    let msg = Information {
        serverity: severity,
        subject: subject.to_string(),
        message,
    };

    match __MESSAGES__.lock().unwrap().entry(thread::current().id()) {
        Entry::Occupied(mut o) => o.get_mut().push(msg),
        Entry::Vacant(v) => {
            v.insert(vec![msg]);
        }
    };
}

//static mut TEMP: i32 = 10;

pub fn get_messages() -> Option<Vec<Information>> {
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

    __MESSAGES__.lock().unwrap().remove(&thread::current().id())
}
