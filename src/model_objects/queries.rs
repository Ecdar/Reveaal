use crate::data_reader::parse_queries::parse_to_expression_tree;
use crate::model_objects::expressions::QueryExpression;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// The struct containing a single query
#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    #[serde(deserialize_with = "decode_query")]
    pub query: Option<QueryExpression>,
    pub comment: String,
}

impl Query {
    pub fn get_query(&self) -> &Option<QueryExpression> {
        &self.query
    }
}

/// Function used for deserializing queries
pub fn decode_query<'de, D>(deserializer: D) -> Result<Option<QueryExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }

    let queries = parse_to_expression_tree(&s).unwrap();
    match queries.len() {
        0 => Err(Error::custom(format!(
            "Could not parse query {} contains no queries",
            s
        ))),
        1 => Ok(queries.into_iter().next()),
        _ => Err(Error::custom(format!(
            "Could not parse query {} contains multiple queries",
            s
        ))),
    }
}
