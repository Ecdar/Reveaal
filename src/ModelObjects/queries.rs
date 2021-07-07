use crate::DataReader::parse_queries;
use crate::ModelObjects::representations;
use serde::{Deserialize, Deserializer};

/// The struct containing a single query
#[derive(Debug, Deserialize)]
pub struct Query {
    #[serde(deserialize_with = "decode_query")]
    pub query: Option<representations::QueryExpression>,
    pub comment: String,
}

impl Query {
    pub fn get_query(&self) -> &Option<representations::QueryExpression> {
        &self.query
    }
}

/// Function used for deserializing queries
pub fn decode_query<'de, D>(
    deserializer: D,
) -> Result<Option<representations::QueryExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() == 0 {
        return Ok(None);
    }
    match parse_queries::parse(&s) {
        Ok(edgeAttribute) => return Ok(Some(edgeAttribute)),
        Err(e) => panic!("Could not parse query {} got error: {:?}", s, e),
    }
}
