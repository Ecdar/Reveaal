extern crate pest;
use crate::DataReader::serialization::encode_boolexpr;
use crate::ModelObjects::representations::BoolExpression;
use pest::Parser;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use std::collections::HashMap;
use std::error::Error;

///This file handles parsing the edges based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate
#[derive(Parser)]
#[grammar = "DataReader/grammars/edge_grammar.pest"]
pub struct EdgeParser;

#[derive(Debug, Clone, Deserialize)]
pub enum EdgeAttribute {
    Updates(Vec<Update>),
    Guard(BoolExpression),
}

#[derive(Debug, Clone, Deserialize, Serialize, std::cmp::PartialEq)]
pub struct Update {
    variable: String,
    #[serde(serialize_with = "encode_boolexpr")]
    expression: BoolExpression,
}

#[allow(dead_code)]
impl Update {
    pub fn get_expression(&self) -> &BoolExpression {
        &self.expression
    }

    pub fn get_variable_name(&self) -> &str {
        self.variable.as_str()
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        if self.variable == from_name {
            self.variable = to_name.to_string();
        }

        self.expression.swap_var_name(from_name, to_name);
    }

    pub fn swap_clock_names(
        &mut self,
        from_vars: &HashMap<String, u32>,
        to_vars: &HashMap<String, u32>,
    ) {
        let index = from_vars.get(&self.variable).unwrap();
        let new_name = to_vars
            .iter()
            .find_map(|(key, val)| if *val == *index { Some(key) } else { None })
            .unwrap();
        self.variable = new_name.clone();
        self.expression = self.expression.swap_clock_names(from_vars, to_vars);
    }
}

pub fn parse(edge_attribute_str: &str) -> Result<EdgeAttribute, Box<dyn Error>> {
    let mut pairs = EdgeParser::parse(Rule::edgeAttribute, edge_attribute_str)?;
    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::edgeAttribute => Ok(build_edgeAttribute_from_pair(pair)),
            err => {
                bail!("Unable to match edgeAttribute string as rule: {:?}", err)
            }
        }
    } else {
        bail!("Expected a pair during parsing but found none")
    }
}

pub fn build_edgeAttribute_from_pair(pair: pest::iterators::Pair<Rule>) -> EdgeAttribute {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::update => build_update_from_pair(pair),
        Rule::guard => build_guard_from_pair(pair),
        err => {
            panic!("Unable to match update string as rule: {:?}", err)
        }
    }
}

fn build_guard_from_pair(pair: pest::iterators::Pair<Rule>) -> EdgeAttribute {
    match pair.as_rule() {
        Rule::guard => {
            let pair_span = pair.as_span();

            //check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return EdgeAttribute::Guard(BoolExpression::Bool(true));
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = inner_pairs.next().unwrap();

            EdgeAttribute::Guard(build_expression_from_pair(inner_pair))
        }
        _ => panic!("Unable to match: {:?} as rule, guard", pair),
    }
}

fn build_update_from_pair(pair: pest::iterators::Pair<Rule>) -> EdgeAttribute {
    match pair.as_rule() {
        Rule::update => {
            let mut updates: Vec<Update> = vec![];
            let pair_span = pair.as_span();

            //check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return EdgeAttribute::Updates(updates);
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = inner_pairs.next().unwrap();

            updates = build_assignments_from_pair(inner_pair);

            EdgeAttribute::Updates(updates)
        }
        _ => panic!("Unable to match: {:?} as rule, update", pair),
    }
}

fn build_assignments_from_pair(pair: pest::iterators::Pair<Rule>) -> Vec<Update> {
    let mut updates: Vec<Update> = vec![];
    match pair.as_rule() {
        Rule::assignments => {
            let mut pairs = pair.into_inner();
            let assignment_pair = pairs.next().unwrap();
            match assignment_pair.as_rule() {
                Rule::assignment => {
                    updates = build_assignments_from_pair(pairs.next().unwrap());
                    updates.push(build_assignment_from_pair(assignment_pair));
                }
                Rule::finalAssignment => {
                    updates.push(build_assignment_from_pair(assignment_pair));
                }
                err => panic!(
                    "Unable to match: {:?} as rule assignment or finalAssignment",
                    err
                ),
            }
        }
        unknown_pair => panic!(
            "Tried to match pair as assignment, but it was {:?}",
            unknown_pair
        ),
    }

    updates
}

fn build_assignment_from_pair(pair: pest::iterators::Pair<Rule>) -> Update {
    let mut inner_pairs = pair.into_inner();
    let variable = inner_pairs.next().unwrap().as_str();
    let expression_pair = inner_pairs.next().unwrap();

    let expression = build_expression_from_pair(expression_pair);
    let update = Update {
        variable: variable.trim().to_string(),
        expression,
    };

    update
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            BoolExpression::Parentheses(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::and => build_and_from_pair(pair),
        Rule::or => build_or_from_pair(pair),
        Rule::compareExpr => build_compareExpr_from_pair(pair),
        Rule::expression => build_expression_from_pair(pair.into_inner().next().unwrap()),
        Rule::terms => build_expression_from_pair(pair.into_inner().next().unwrap()),
        unknown => panic!("Got unknown pair: {:?}", unknown),
    }
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>() {
                BoolExpression::Bool(n)
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>() {
                BoolExpression::Int(n)
            } else {
                build_term_from_pair(inner_pair)
            }
        }
        Rule::variable => BoolExpression::VarName(inner_pair.as_str().trim().to_string()),
        err => panic!("Unable to match: {:?} as rule atom or variable", err),
    }
}

fn build_and_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_or_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_expression_from_pair(left_side_pair);
            let rside = build_expression_from_pair(right_side_pair);

            BoolExpression::AndOp(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_or_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_compareExpr_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_expression_from_pair(left_side_pair);
            let rside = build_expression_from_pair(right_side_pair);

            BoolExpression::OrOp(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_compareExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_expression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.next().unwrap();

            let lside = build_expression_from_pair(left_side_pair);
            let rside = build_expression_from_pair(right_side_pair);

            match operator_pair.as_str() {
                ">=" => BoolExpression::GreatEQ(Box::new(lside), Box::new(rside)),
                "<=" => BoolExpression::LessEQ(Box::new(lside), Box::new(rside)),
                "==" => BoolExpression::EQ(Box::new(lside), Box::new(rside)),
                "<" => BoolExpression::LessT(Box::new(lside), Box::new(rside)),
                ">" => BoolExpression::GreatT(Box::new(lside), Box::new(rside)),
                unknown_operator => panic!(
                    "Got unknown boolean operator: {}. Only able to match >=,<=, ==,<,>",
                    unknown_operator
                ),
            }
        }
    }
}
