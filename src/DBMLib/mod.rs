#[cfg(target_os = "linux")]
pub mod lib_linux;
#[cfg(not(target_os = "linux"))]
pub mod lib_stub;

#[cfg(target_os = "linux")]
pub use lib_linux as lib;
#[cfg(not(target_os = "linux"))]
pub use lib_stub as lib;
