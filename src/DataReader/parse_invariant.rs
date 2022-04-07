extern crate pest;
use crate::bail;
use crate::ModelObjects::representations::BoolExpression;
use anyhow::Result;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

/// This file handles parsing the invariants based on the abstract syntax described in the .pest files in the grammar folder
/// For clarification see documentation on pest crate
#[derive(Parser)]
#[grammar = "DataReader/grammars/invariant_grammar.pest"]
pub struct InvariantParser;

pub fn parse(edge_attribute_str: &str) -> Result<BoolExpression> {
    let mut pairs = InvariantParser::parse(Rule::invariant, edge_attribute_str)?;
    let pair = try_next(&mut pairs)?;
    match pair.as_rule() {
        Rule::invariant => build_invariant_from_pair(pair),
        err => {
            bail!("Unable to match invariant string as rule: {:?}", err)
        }
    }
}

pub fn build_invariant_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let pair = try_next(&mut pair.into_inner())?;
    match pair.as_rule() {
        Rule::andExpr => {
            let pair_span = pair.as_span();

            // check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return Ok(BoolExpression::Bool(true));
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = try_next(&mut inner_pairs)?;

            build_expression_from_pair(inner_pair)
        }
        _ => bail!("Unable to match: {:?} as rule, guard", pair),
    }
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
        Rule::andExpr => build_and_from_pair(pair),
        Rule::orExpr => build_or_from_pair(pair),
        Rule::compareExpr => build_compareExpr_from_pair(pair),
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
                build_term_from_pair(inner_pair)
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
        None => build_or_from_pair(left_side_pair),
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
        None => build_compareExpr_from_pair(left_side_pair),
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
        None => build_expression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = try_next(&mut inner_pair)?;

            let lside = build_expression_from_pair(left_side_pair)?;
            let rside = build_expression_from_pair(right_side_pair)?;

            match operator_pair.as_str() {
                ">=" => Ok(BoolExpression::GreatEQ(Box::new(lside), Box::new(rside))),
                "<=" => Ok(BoolExpression::LessEQ(Box::new(lside), Box::new(rside))),
                "<" => Ok(BoolExpression::LessT(Box::new(lside), Box::new(rside))),
                ">" => Ok(BoolExpression::GreatT(Box::new(lside), Box::new(rside))),
                unknown_operator => bail!(
                    "Got unknown boolean operator: {}. Only able to match >=,<=,<,>",
                    unknown_operator
                ),
            }
        }
    }
}

fn try_next<'i>(iterator: &mut Pairs<'i, Rule>) -> Result<Pair<'i, Rule>> {
    if let Some(pair) = iterator.next() {
        Ok(pair)
    } else {
        bail!("Expected pair but got None instead")
    }
}
