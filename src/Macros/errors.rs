use anyhow::{Context, Result};
use core::fmt::Display;

#[doc(hidden)]
pub fn _add_info_to_result<T, E, R: Context<T, E>, C, F>(res: R, info: F) -> Result<T>
where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
{
    res.with_context(info)
}

#[macro_export]
macro_rules! location {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}

/// Add file and line number information + optional context to an Option<T> or Result<T, E>
#[macro_export]
macro_rules! context {
    ($result:expr) => { context!($result, "Unexpected error") };
    ($result:expr, $($args:expr ),*) => { $crate::Macros::errors::_add_info_to_result($result, || format!("{}\n\t at {}", format!( $( $args ),* ), crate::location!())) };
}

/// Construct an anyhow::Error with file and line number information
#[macro_export]
macro_rules! error {
    ($($args:expr ),*) => {$crate::anyhow::Error::msg(format!("{}\n\t at {}", format!( $( $args ),* ), crate::location!()))}
}

/// Try to unwrap an option and on fail return an error with file and line number information
#[macro_export]
macro_rules! to_result {
    ($option:expr) => {
        $option.ok_or_else(|| $crate::error!("Optional was expected to be Some but was None"))
    };
}

/// Bail with a result with file and line number information + optional context
#[macro_export]
macro_rules! bail {
    ($($args:expr ),*) => {$crate::anyhow::bail!(format!("{}\n\t at {}", format!( $( $args ),* ), crate::location!()))}
}
