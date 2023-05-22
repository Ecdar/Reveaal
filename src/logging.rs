use chrono::Local;
use colored::{ColoredString, Colorize};
use log::SetLoggerError;
use std::io::Write;
use std::thread;

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
macro_rules! msg {
    ($severity:expr, subject: $subject:expr, msg: $msg:expr) => ({
        if $crate::is_server() {
            $crate::logging::message::__set_info__($crate::logging::message::__as_information__($severity, $subject, $msg));
        } else {
            //let lvl = $crate::logging::__severity__($severity);
            //log::log!(lvl, "{}", $crate::logging::__as_information__($severity, $subject, $msg));
            println!("{}", $crate::logging::message::__as_information__($severity, $subject, $msg));
        }
    });


    ($severity:expr, subject: $subject:expr, msg: $($msg:tt)+) => (msg!($severity, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($subject:expr, msg: $msg:expr) => (msg!(0, subject: $subject, msg: $msg));
    ($subject:expr, msg: $($msg:tt)+) => (msg!(0, subject: $subject, msg: format_args!($($msg)+).to_string()));

    ($msg:expr) => (msg!(0, subject: "general", msg: $msg.to_string()));
    ($($msg:tt)+) => (msg!(0, subject: "general", msg: format_args!($($msg)+).to_string()));
}

/// Function to get information messages.
/// ### Info
/// Will always return `None` when Reveaal is run through the CLI, only use as server.
pub fn get_messages() -> Vec<crate::ProtobufServer::services::query_response::Information> {
    message::__MESSAGES__
        .lock()
        .unwrap()
        .remove(&thread::current().id())
        .unwrap_or_default()
}

#[doc(hidden)]
pub mod message {
    use crate::ProtobufServer::services::query_response::{information, Information};
    use chrono::Local;
    use colored::Colorize;
    use once_cell::sync::Lazy;
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::sync::Mutex;
    use std::thread;
    use std::thread::ThreadId;

    impl Display for Information {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let lvl = match information::Severity::from_i32(self.severity) {
                Some(s @ information::Severity::Warning) => s.as_str_name().yellow(),
                Some(s @ information::Severity::Info) => s.as_str_name().cyan(),
                None => panic!("Couldn't parse severity"),
            };
            write!(
                f,
                "[{} {}: {}] - {}",
                Local::now().format("%H:%M:%S").to_string().cyan(),
                lvl,
                self.subject,
                self.message
            )
        }
    }

    #[doc(hidden)]
    pub static __MESSAGES__: Lazy<Mutex<HashMap<ThreadId, Vec<Information>>>> =
        Lazy::new(Mutex::default);

    #[doc(hidden)]
    pub fn __as_information__(severity: i32, subject: &str, message: String) -> Information {
        Information {
            severity,
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

    #[cfg(test)]
    mod tests {
        use crate::logging::get_messages;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn multithreading_msg_test() {
            msg!("{:?}", thread::current().id());
            for _ in 1..=10 {
                thread::spawn(|| {
                    msg!("{:?}", thread::current().id());
                    thread::sleep(Duration::from_millis(100));
                    let msgs = get_messages();
                    assert_eq!(msgs.len(), 1);
                    assert_eq!(get_messages().len(), 0);
                    assert_eq!(
                        msgs.first().unwrap().message,
                        format!("{:?}", thread::current().id())
                    );
                });
            }
            thread::sleep(Duration::from_millis(200));
            let msgs = get_messages();
            assert_eq!(msgs.len(), 1);
            assert_eq!(get_messages().len(), 0);
            assert_eq!(
                msgs.first().unwrap().message,
                format!("{:?}", thread::current().id())
            );
        }
    }
}
