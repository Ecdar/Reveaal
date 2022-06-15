extern crate pest;
use crate::bail;
use crate::DataReader::parse_utils::TryNextable;
use crate::ModelObjects::representations::BoolExpression;
use anyhow::Result;
use pest::Parser;

/// This file handles parsing the invariants based on the abstract syntax described in the .pest files in the grammar folder
/// For clarification see documentation on pest crate
#[derive(Parser)]
#[grammar = "DataReader/grammars/invariant_grammar.pest"]
pub struct InvariantParser;

pub fn parse(edge_attribute_str: &str) -> Result<BoolExpression> {
    let mut pairs = InvariantParser::parse(Rule::invariant, edge_attribute_str)?;
    let pair = pairs.try_next()?;
    match pair.as_rule() {
        Rule::invariant => build_invariant_from_pair(pair),
        err => {
            bail!("Unable to match invariant string as rule: {:?}", err)
        }
    }
}

pub fn build_invariant_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let pair = pair.into_inner().try_next()?;
    match pair.as_rule() {
        Rule::andExpr => {
            let pair_span = pair.as_span();

            // check if we have an empty pair
            if pair_span.start() == pair_span.end() {
                return Ok(BoolExpression::Bool(true));
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = inner_pairs.try_next()?;

            build_expression_from_pair(inner_pair)
        }
        _ => bail!("Unable to match: {:?} as rule, guard", pair),
    }
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(BoolExpression::Parentheses(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::andExpr => build_and_from_pair(pair),
        Rule::orExpr => build_or_from_pair(pair),
        Rule::compareExpr => build_compareExpr_from_pair(pair),
        Rule::terms => build_expression_from_pair(pair.into_inner().try_next()?),
        unknown => bail!("Got unknown pair: {:?}", unknown),
    }
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<BoolExpression> {
    let inner_pair = pair.into_inner().try_next()?;
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
    let left_side_pair = inner_pair.try_next()?;

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
    let left_side_pair = inner_pair.try_next()?;

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
    let left_side_pair = inner_pair.try_next()?;

    match inner_pair.next() {
        None => build_expression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.try_next()?;

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
