#[cfg(not(feature = "dbm-stub"))]
pub mod lib_dbm;
#[cfg(feature = "dbm-stub")]
pub mod lib_stub;

#[cfg(not(feature = "dbm-stub"))]
pub use lib_dbm as lib;
#[cfg(feature = "dbm-stub")]
pub use lib_stub as lib;

pub mod dbm;
