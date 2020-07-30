use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<BoolExpression>, Box<BoolExpression>),
    GreatEQ(Box<BoolExpression>, Box<BoolExpression>),
    LessT(Box<BoolExpression>, Box<BoolExpression>),
    GreatT(Box<BoolExpression>, Box<BoolExpression>),
    Parentheses(Box<BoolExpression>),
    Not(Box<BoolExpression>),
    VarName(String),
    Bool(bool),
    Int(i32),
}

#[derive(Debug, Clone, Deserialize)]
pub enum QueryExpression {
    Refinement(Box<QueryExpression>, Box<QueryExpression>),
    Consistency(Box<QueryExpression>),
    Implementation(Box<QueryExpression>),
    Determinism(Box<QueryExpression>),
    Specification(Box<QueryExpression>),
    Conjunction(Box<QueryExpression>, Box<QueryExpression>),
    Composition(Box<QueryExpression>, Box<QueryExpression>),
    Quotient(Box<QueryExpression>, Box<QueryExpression>),
    Possibly(Box<BoolExpression>),
    Invariantly(Box<BoolExpression>),
    EventuallyAlways(Box<BoolExpression>),
    Potentially(Box<BoolExpression>),
    Parentheses(Box<QueryExpression>),
    ComponentExpression(Box<QueryExpression>, Box<BoolExpression>),
    VarName(String),
    Bool(bool),
    Int(i32),
}