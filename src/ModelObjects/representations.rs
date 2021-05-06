use crate::DBMLib::lib;
use crate::ModelObjects::component::Component;
use serde::Deserialize;

/// This file contains the nested enums used to represent systems on each side of refinement as well as all guards, updates etc
/// note that the enum contains a box (pointer) to an object as they can only hold pointers to data on the heap

#[derive(Debug, Clone, Deserialize, std::cmp::PartialEq)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<BoolExpression>, Box<BoolExpression>),
    GreatEQ(Box<BoolExpression>, Box<BoolExpression>),
    LessT(Box<BoolExpression>, Box<BoolExpression>),
    GreatT(Box<BoolExpression>, Box<BoolExpression>),
    EQ(Box<BoolExpression>, Box<BoolExpression>),
    Parentheses(Box<BoolExpression>),
    Clock(u32),
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
    Possibly(Box<QueryExpression>),
    Invariantly(Box<QueryExpression>),
    EventuallyAlways(Box<QueryExpression>),
    Potentially(Box<QueryExpression>),
    Parentheses(Box<QueryExpression>),
    ComponentExpression(Box<QueryExpression>, Box<QueryExpression>),
    AndOp(Box<QueryExpression>, Box<QueryExpression>),
    OrOp(Box<QueryExpression>, Box<QueryExpression>),
    LessEQ(Box<QueryExpression>, Box<QueryExpression>),
    GreatEQ(Box<QueryExpression>, Box<QueryExpression>),
    LessT(Box<QueryExpression>, Box<QueryExpression>),
    GreatT(Box<QueryExpression>, Box<QueryExpression>),
    Not(Box<QueryExpression>),
    VarName(String),
    Bool(bool),
    Int(i32),
}

#[derive(Debug, Clone)]
pub enum SystemRepresentation {
    Composition(Box<SystemRepresentation>, Box<SystemRepresentation>),
    Conjunction(Box<SystemRepresentation>, Box<SystemRepresentation>),
    Parentheses(Box<SystemRepresentation>),
    Component(Component),
}

impl<'a> SystemRepresentation{
    pub fn any_composition<F>(
        &'a self,
        predicate: &mut F
    ) -> bool where
    F: FnMut(&'a Component) -> bool{
        match self{
            SystemRepresentation::Composition(left_side, right_side) => {
                left_side.any_composition(predicate) || right_side.any_composition(predicate)
            }
            SystemRepresentation::Conjunction(left_side, right_side) => {
                left_side.any_composition(predicate) && right_side.any_composition(predicate)
            }
            SystemRepresentation::Parentheses(rep) => {
                rep.any_composition(predicate)
            }
            SystemRepresentation::Component(comp) => {
                predicate(comp)
            }
        }
    }
}

pub fn print_DBM(dbm: &mut [i32], dimension: u32) {
    println!("DBM:");
    for i in 0..dimension {
        print!("( ");
        for j in 0..dimension {
            print!("{:?} ", lib::rs_dbm_get_constraint(dbm, dimension, i, j));
        }
        print!(")\n");
    }
}
