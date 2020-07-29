use serde::Deserialize;

//The struct containing a single query
#[derive(Debug, Deserialize)]
pub struct Query {
    query: String,
    comment: String,
}

impl Query {
    pub fn get_query(&self) -> &String {
        &self.query
    }
    pub fn get_comment(&self) -> &String {
        &self.comment
    }
}