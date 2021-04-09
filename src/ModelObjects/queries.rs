use super::expression_representation;
use super::parse_queries;
use serde::{Deserialize, Deserializer};

//The struct containing a single query
#[derive(Debug, Deserialize)]
pub struct Query {
    #[serde(deserialize_with = "decode_query")]
    query: Option<expression_representation::QueryExpression>,
    comment: String,
}

impl Query {
    pub fn get_query(&self) -> &Option<expression_representation::QueryExpression> {
        &self.query
    }
    pub fn get_comment(&self) -> &String {
        &self.comment
    }
}

//Function used for deserializing queries
fn decode_query<'de, D>(
    deserializer: D,
) -> Result<Option<expression_representation::QueryExpression>, D::Error>
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
