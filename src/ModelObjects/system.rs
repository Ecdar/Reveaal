use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub name: String,
    pub componentInstances : ComponentInstances,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComponentInstances {
    pub id: usize,
}

pub enum OperatorType{
    composition,
    refinement,

}

#[derive(Debug, Deserialize, Clone)]
pub struct Operators {
    pub id: usize,
}