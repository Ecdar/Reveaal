use anyhow::{Context, Result};

#[doc(hidden)]
pub fn _add_info_to_result<T>(res: Result<T>, info: String) -> Result<T> {
    res.with_context(|| info)
}

/// Add file and line number information + optional context to an anyhow::Result
#[macro_export]
macro_rules! info {
    ($result:expr) => { info!($result, "Caused an error") };
    ($result:expr, $($args:expr ),*) => { $crate::Macros::errors::_add_info_to_result($result, format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* ))) };
}

/// Construct an anyhow::Error with file and line number information
#[macro_export]
macro_rules! error {
    ($($args:expr ),*) => {$crate::anyhow::Error::msg(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}

/// Convert any Result into an anyhow::Result and add file and line number information + optional context
#[macro_export]
macro_rules! into_info {
    ($result:expr) => { $crate::info!($result.map_err(anyhow::Error::msg)) };
    ($result:expr, $($args:expr ),*) => { $crate::info!($result.map_err(anyhow::Error::msg), $($args ),*) };
}

/// Convert an option to a result with file and line number information + optional context
#[macro_export]
macro_rules! open {
    ($option:expr) => { open!($option, "Failed to open Option") };
    ($option:expr, $($args:expr ),*) => {$option.ok_or($crate::error!( $( $args ),*))}
}

/// Bail with a result with file and line number information + optional context
#[macro_export]
macro_rules! bail {
    ($($args:expr ),*) => {$crate::anyhow::bail!(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}
