#![allow(non_snake_case)]
pub mod DataReader;
pub mod DataTypes;
pub mod EdgeEval;
pub mod ModelObjects;
pub mod ProtobufServer;
pub mod Simulation;
pub mod System;
pub mod TransitionSystems;
pub mod logging;
pub mod tests;

pub use crate::DataReader::component_loader::{
    ComponentLoader, JsonProjectLoader, ProjectLoader, XmlProjectLoader,
};
pub use crate::DataReader::{parse_queries, xml_parser};
pub use crate::ModelObjects::queries::Query;
use crate::ProtobufServer::services::query_request::Settings;
pub use crate::System::extract_system_rep;
pub use ModelObjects::component;
pub use ModelObjects::queries;
pub use ProtobufServer::start_grpc_server_with_tokio;

/// The default settings
pub const DEFAULT_SETTINGS: Settings = Settings {
    disable_clock_reduction: true,
};

static mut IS_SERVER: Option<bool> = None;

pub fn set_server(is_server: bool) {
    unsafe {
        IS_SERVER = Some(is_server);
    }
}

fn is_server() -> bool {
    unsafe { IS_SERVER.expect("Server or CLI never specified") }
}

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate core;
extern crate serde;
extern crate serde_xml_rs;
extern crate simple_error;
extern crate xml;
