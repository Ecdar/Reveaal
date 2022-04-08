use anyhow::{Context, Result};

#[doc(hidden)]
pub fn _add_info_to_result<T>(res: Result<T>, info: String) -> Result<T> {
    res.with_context(|| info)
}

#[macro_export]
macro_rules! info {
    ($result:expr) => { info!($result, "") };
    ($result:expr, $($args:expr ),*) => { $crate::Macros::errors::_add_info_to_result($result.map_err(|err| err.into()), format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* ))) };
}

#[macro_export]
macro_rules! error {
    ($($args:expr ),*) => {$crate::anyhow::Error::msg(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}

#[macro_export]
macro_rules! open {
    ($option:expr) => { open!($option, "Failed to open Option") };
    ($option:expr, $($args:expr ),*) => {$option.ok_or($crate::error!( $( $args ),*))}
}

#[macro_export]
macro_rules! bail {
    ($($args:expr ),*) => {$crate::anyhow::bail!(format!("{}: {}", concat!(file!(), ":", line!(), ":", column!()) ,format!( $( $args ),* )))}
}
