use crate::DataReader::parse_queries;
use crate::ModelObjects::representations;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use simple_error::bail;

/// The struct containing a single query
#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    #[serde(deserialize_with = "decode_query")]
    pub query: representations::QueryExpression,
    pub comment: String,
}

impl Query {
    pub fn get_query(&self) -> &representations::QueryExpression {
        &self.query
    }
}

/// Function used for deserializing queries
pub fn decode_query<'de, D>(deserializer: D) -> Result<representations::QueryExpression, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(D::Error::custom(
            "Received an empty input while deserializing queries",
        ));
    }

    let mut queries = match parse_queries::parse(&s) {
        Ok(queries) => queries,
        Err(error) => {
            return Err(D::Error::custom(format!(
                "Failed to parse queries: {}",
                error
            )))
        }
    };
    if queries.len() > 1 {
        Err(D::Error::custom(format!(
            "Could not parse query {} contains multiple queries",
            s
        )))
    } else if queries.len() == 0 {
        Err(D::Error::custom(format!(
            "Could not parse query {} contains no queries",
            s
        )))
    } else {
        Ok(queries.remove(0))
    }
}
