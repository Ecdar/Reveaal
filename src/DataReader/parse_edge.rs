extern crate pest;
use crate::DataReader::serialization::encode_boolexpr;
use crate::ModelObjects::representations::BoolExpression;
use crate::{bail, open};
use anyhow::Result;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    ) -> Result<()> {
        if let Some(index) = from_vars.get(&self.variable) {
            if let Some(new_name) =
                to_vars
                    .iter()
                    .find_map(|(key, val)| if *val == *index { Some(key) } else { None })
            {
                self.variable = new_name.clone();
                self.expression = self.expression.swap_clock_names(from_vars, to_vars)?;
                return Ok(());
            }
        }
        bail!("Couldnt find clock names to switch")
    }
}

pub fn parse(edge_attribute_str: &str) -> Result<EdgeAttribute> {
    let mut pairs = EdgeParser::parse(Rule::edgeAttribute, edge_attribute_str)?;
    let pair = try_next(&mut pairs)?;
    match pair.as_rule() {
        Rule::edgeAttribute => build_edgeAttribute_from_pair(pair),
        err => {
            bail!("Unable to match edgeAttribute string as rule: {:?}", err)
        }
    }
}

pub fn build_edgeAttribute_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<EdgeAttribute> {
    let pair_rule = try_next(&mut pair.into_inner())?;
    match pair_rule.as_rule() {
        Rule::update => build_update_from_pair(pair_rule),
        Rule::guard => build_guard_from_pair(pair_rule),
        err => {
            bail!("Unable to match update string as rule: {:?}", err)
        }
    }
}

fn build_guard_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<EdgeAttribute> {
    match pair.as_rule() {
        Rule::guard => {
            let pair_span = pair.as_span();

            //check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return Ok(EdgeAttribute::Guard(BoolExpression::Bool(true)));
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = try_next(&mut inner_pairs)?;
            Ok(EdgeAttribute::Guard(build_expression_from_pair(
                inner_pair,
            )?))
        }
        _ => bail!("Unable to match: {:?} as rule, guard", pair),
    }
}

fn build_update_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<EdgeAttribute> {
    match pair.as_rule() {
        Rule::update => {
            let mut updates: Vec<Update> = vec![];
            let pair_span = pair.as_span();

            //check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return Ok(EdgeAttribute::Updates(updates));
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = try_next(&mut inner_pairs)?;
            updates = build_assignments_from_pair(inner_pair)?;
            Ok(EdgeAttribute::Updates(updates))
        }
        _ => bail!("Unable to match: {:?} as rule, update", pair),
    }
}

fn build_assignments_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Update>> {
    let mut updates: Vec<Update> = vec![];
    match pair.as_rule() {
        Rule::assignments => {
            let mut pairs = pair.into_inner();
            let assignment_pair = try_next(&mut pairs)?;
            match assignment_pair.as_rule() {
                Rule::assignment => {
                    let next_pair = try_next(&mut pairs)?;
                    updates = build_assignments_from_pair(next_pair)?;
                    updates.push(build_assignment_from_pair(assignment_pair)?);
                }
                Rule::finalAssignment => {
                    updates.push(build_assignment_from_pair(assignment_pair)?);
                }
                err => bail!(
                    "Unable to match: {:?} as rule assignment or finalAssignment",
                    err
                ),
            }
        }
        unknown_pair => bail!(
            "Tried to match pair as assignment, but it was {:?}",
            unknown_pair
        ),
    }

    Ok(updates)
}

fn build_assignment_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<Update> {
    let mut inner_pairs = pair.into_inner();
    let variable = try_next(&mut inner_pairs)?.as_str();
    let expression_pair = try_next(&mut inner_pairs)?;

    let expression = build_expression_from_pair(expression_pair)?;
    let update = Update {
        variable: variable.trim().to_string(),
        expression,
    };

    Ok(update)
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedExp => {
            let inner_pair = try_next(&mut pair.into_inner())?;
            Ok(BoolExpression::Parentheses(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::and => build_and_from_pair(pair),
        Rule::or => build_or_from_pair(pair),
        Rule::compareExpr => build_compareExpr_from_pair(pair),
        Rule::expression => build_expression_from_pair(try_next(&mut pair.into_inner())?),
        Rule::terms => build_expression_from_pair(try_next(&mut pair.into_inner())?),
        unknown => bail!("Got unknown pair: {:?}", unknown),
    }
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let inner_pair = try_next(&mut pair.into_inner())?;
    match inner_pair.as_rule() {
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>() {
                Ok(BoolExpression::Bool(n))
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>() {
                Ok(BoolExpression::Int(n))
            } else {
                Ok(build_term_from_pair(inner_pair)?)
            }
        }
        Rule::variable => Ok(BoolExpression::VarName(
            inner_pair.as_str().trim().to_string(),
        )),
        err => bail!("Unable to match: {:?} as rule atom or variable", err),
    }
}

fn build_and_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = try_next(&mut inner_pair)?;

    match inner_pair.next() {
        None => Ok(build_or_from_pair(left_side_pair)?),
        Some(right_side_pair) => {
            let lside = build_expression_from_pair(left_side_pair)?;
            let rside = build_expression_from_pair(right_side_pair)?;

            Ok(BoolExpression::AndOp(Box::new(lside), Box::new(rside)))
        }
    }
}

fn build_or_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = try_next(&mut inner_pair)?;

    match inner_pair.next() {
        None => Ok(build_compareExpr_from_pair(left_side_pair)?),
        Some(right_side_pair) => {
            let lside = build_expression_from_pair(left_side_pair)?;
            let rside = build_expression_from_pair(right_side_pair)?;

            Ok(BoolExpression::OrOp(Box::new(lside), Box::new(rside)))
        }
    }
}

fn build_compareExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = try_next(&mut inner_pair)?;

    match inner_pair.next() {
        None => Ok(build_expression_from_pair(left_side_pair)?),
        Some(operator_pair) => {
            let right_side_pair = try_next(&mut inner_pair)?;

            let lside = build_expression_from_pair(left_side_pair)?;
            let rside = build_expression_from_pair(right_side_pair)?;

            match operator_pair.as_str() {
                ">=" => Ok(BoolExpression::GreatEQ(Box::new(lside), Box::new(rside))),
                "<=" => Ok(BoolExpression::LessEQ(Box::new(lside), Box::new(rside))),
                "==" => Ok(BoolExpression::EQ(Box::new(lside), Box::new(rside))),
                "<" => Ok(BoolExpression::LessT(Box::new(lside), Box::new(rside))),
                ">" => Ok(BoolExpression::GreatT(Box::new(lside), Box::new(rside))),
                unknown_operator => bail!(
                    "Got unknown boolean operator: {}. Only able to match >=,<=, ==,<,>",
                    unknown_operator
                ),
            }
        }
    }
}

fn try_next<'i>(iterator: &mut Pairs<'i, Rule>) -> Result<Pair<'i, Rule>> {
    open!(iterator.next(), "Expected pair but got None instead")
}
