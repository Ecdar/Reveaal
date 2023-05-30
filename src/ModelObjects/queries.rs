use crate::DataReader::parse_queries;
use crate::ModelObjects::Expressions;
use serde::{Deserialize, Deserializer};

/// The struct containing a single query
#[derive(Debug, Deserialize, Clone)]
pub struct Query {
    #[serde(deserialize_with = "decode_query")]
    pub query: Option<Expressions::QueryExpression>,
    pub comment: String,
}

impl Query {
    pub fn get_query(&self) -> &Option<Expressions::QueryExpression> {
        &self.query
    }
}

/// Function used for deserializing queries
pub fn decode_query<'de, D>(
    deserializer: D,
) -> Result<Option<Expressions::QueryExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }

    let queries = parse_queries::parse_to_expression_tree(&s).unwrap();
    if queries.len() > 1 {
        panic!("Could not parse query {} contains multiple queries", s);
    } else if queries.is_empty() {
        panic!("Could not parse query {} contains no queries", s);
    } else {
        Ok(queries.into_iter().next())
    }
}
