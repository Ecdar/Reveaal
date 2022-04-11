use anyhow::{Context, Result};

#[doc(hidden)]
pub fn _add_info_to_result<T, E, R: Context<T, E>>(res: R, info: String) -> Result<T> {
    res.with_context(|| info)
}

/// Add file and line number information + optional context to an Option<T> or an anyhow::Result<T, E>
#[macro_export]
macro_rules! context {
    ($result:expr) => { context!($result, "Unexpected error") };
    ($result:expr, $($args:expr ),*) => { $crate::Macros::errors::_add_info_to_result($result, format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* ))) };
}

/// Construct an anyhow::Error with file and line number information
#[macro_export]
macro_rules! error {
    ($($args:expr ),*) => {$crate::anyhow::Error::msg(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}

/// Convert any Result into an anyhow::Result and add file and line number information + optional context
#[macro_export]
macro_rules! into_context {
    ($result:expr) => { $crate::context!($result.map_err(anyhow::Error::msg)) };
    ($result:expr, $($args:expr ),*) => { $crate::context!($result.map_err(anyhow::Error::msg), $($args ),*) };
}

/// Try to unwrap an option and on fail return an error with file and line number information
#[macro_export]
macro_rules! open {
    ($option:expr) => {
        $option.ok_or($crate::error!(
            "Optional was expected to be Some but was None"
        ))
    };
}

/// Bail with a result with file and line number information + optional context
#[macro_export]
macro_rules! bail {
    ($($args:expr ),*) => {$crate::anyhow::bail!(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}
