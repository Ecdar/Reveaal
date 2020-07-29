use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub name: String,
}