extern crate pest;
use crate::bail;
use crate::DataReader::parse_utils::TryNextable;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use anyhow::Result;
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

#[derive(Parser)]
#[grammar = "DataReader/grammars/query_grammar.pest"]
pub struct QueryParser;

///This file handles parsing the queries based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate

pub fn parse_to_query(query: &str) -> Result<Vec<Query>> {
    let query_expressions = parse_to_expression_tree(query)?;
    let queries = query_expressions
        .into_iter()
        .map(|q| Query {
            query: q,
            comment: "".to_string(),
        })
        .collect();
    Ok(queries)
}

pub fn parse_to_expression_tree(edge_attribute_str: &str) -> Result<Vec<QueryExpression>> {
    let mut pairs = QueryParser::parse(Rule::queries, edge_attribute_str)?;
    let pair = pairs.try_next()?;
    let mut queries = vec![];
    match pair.as_rule() {
        Rule::queries => {
            build_queries(pair, &mut queries)?;
            Ok(queries)
        }
        err => {
            bail!("Unable to match query string as rule: {:?}", err)
        }
    }
}

pub fn build_queries(
    pair: pest::iterators::Pair<Rule>,
    list: &mut Vec<QueryExpression>,
) -> Result<()> {
    match pair.as_rule() {
        Rule::queryList => {
            for p in pair.into_inner() {
                build_queries(p, list)?
            }
        }
        Rule::queries => {
            for p in pair.into_inner() {
                build_queries(p, list)?
            }
        }
        Rule::query => {
            list.push(build_query_from_pair(pair)?);
        }
        _ => {}
    }
    Ok(())
}

pub fn build_query_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let pair = pair.into_inner().try_next()?;
    let pair_span = pair.as_span();

    //check if we have an empty pair
    if pair_span.start() == pair_span.end() {
        return Ok(QueryExpression::Bool(true));
    }

    match pair.as_rule() {
        Rule::refinement => build_refinement_from_pair(pair),
        Rule::getComponent => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::GetComponent(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::prune => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Prune(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::bisim => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::BisimMinimize(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }

        Rule::consistency => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Consistency(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::implementation => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Implementation(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::determinism => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Determinism(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::specification => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Specification(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::logicFormulas => {
            let inner_pair = pair.into_inner().try_next()?;
            build_expression_from_pair(inner_pair)
        }
        unknown => bail!("Got unknown pair: {:?}", unknown),
    }
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Parentheses(Box::new(
                build_expression_from_pair(inner_pair)?,
            )))
        }
        Rule::possibly => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Possibly(Box::new(
                build_boolExpr_from_pair(inner_pair)?,
            )))
        }
        Rule::invariantly => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Invariantly(Box::new(
                build_boolExpr_from_pair(inner_pair)?,
            )))
        }
        Rule::eventuallyAlways => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::EventuallyAlways(Box::new(
                build_boolExpr_from_pair(inner_pair)?,
            )))
        }
        Rule::potentially => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Potentially(Box::new(
                build_boolExpr_from_pair(inner_pair)?,
            )))
        }
        Rule::expr => {
            //Set precedence in ascending order
            let precedence_climber = PrecClimber::new(vec![
                Operator::new(Rule::qoutient_op, Assoc::Left),
                Operator::new(Rule::composition_op, Assoc::Left),
                Operator::new(Rule::conjunction_op, Assoc::Left),
            ]);
            let primary = |pair| build_expression_from_pair(pair);
            let inner: Vec<Pair<Rule>> = pair.into_inner().collect();
            precedence_climber.climb(inner.into_iter(), primary, |lhs, op, rhs| {
                match op.as_rule() {
                    Rule::composition_op => {
                        Ok(QueryExpression::Composition(Box::new(lhs?), Box::new(rhs?)))
                    }
                    Rule::conjunction_op => {
                        Ok(QueryExpression::Conjunction(Box::new(lhs?), Box::new(rhs?)))
                    }
                    Rule::qoutient_op => {
                        Ok(QueryExpression::Quotient(Box::new(lhs?), Box::new(rhs?)))
                    }
                    _ => unreachable!(),
                }
            })
        }
        Rule::saveExpr => {
            let mut inner = pair.into_inner();
            let inner_pair = inner.try_next()?;
            let name = inner.try_next()?;
            let name = build_var_from_pair(name)?;
            match name {
                QueryExpression::VarName(save_name) => Ok(QueryExpression::SaveAs(
                    Box::new(build_expression_from_pair(inner_pair)?),
                    save_name,
                )),
                _ => bail!("Could not parse save-as name"),
            }
        }
        Rule::terms => {
            let inner_pair = pair.into_inner().try_next()?;
            build_expression_from_pair(inner_pair)
        }
        unknown => bail!("Got unknown pair: {:?}", unknown),
    }
}

fn build_refinement_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.try_next()?;
    let right_side_pair = inner_pair.try_next()?;

    let lside = build_expression_from_pair(left_side_pair)?;
    let rside = build_expression_from_pair(right_side_pair)?;

    Ok(QueryExpression::Refinement(
        Box::new(lside),
        Box::new(rside),
    ))
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let inner_pair = pair.into_inner().try_next()?;
    match inner_pair.as_rule() {
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>() {
                Ok(QueryExpression::Bool(n))
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>() {
                Ok(QueryExpression::Int(n))
            } else {
                build_term_from_pair(inner_pair)
            }
        }
        Rule::var => build_var_from_pair(inner_pair),
        err => bail!("Unable to match: {:?} as rule atom or variable", err),
    }
}

fn build_var_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.try_next()?;

    let lside = QueryExpression::VarName(left_side_pair.as_str().trim().to_string());

    match inner_pair.next() {
        None => Ok(lside),
        Some(right_side_pair) => {
            let inner_right_pair = right_side_pair.into_inner().try_next()?;
            let rside = build_expression_from_pair(inner_right_pair)?;

            Ok(QueryExpression::ComponentExpression(
                Box::new(lside),
                Box::new(rside),
            ))
        }
    }
}

fn build_boolExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    match pair.as_rule() {
        Rule::boolExpr => {
            let inner_pair = pair.into_inner().try_next()?;
            build_boolExpr_from_pair(inner_pair)
        }
        Rule::andExpr => build_and_from_pair(pair),
        Rule::orExpr => build_or_from_pair(pair),
        Rule::compExpr => build_compareExpr_from_pair(pair),
        Rule::subExpr => build_subExpression_from_pair(pair),
        unknown => bail!("Expected and pair but got unknown pair: {:?}", unknown),
    }
}

fn build_subExpression_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedSubExp => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Parentheses(Box::new(
                build_subExpression_from_pair(inner_pair)?,
            )))
        }
        Rule::notExpr => {
            let inner_pair = pair.into_inner().try_next()?;
            Ok(QueryExpression::Not(Box::new(
                build_subExpression_from_pair(inner_pair)?,
            )))
        }
        Rule::subExpr => {
            let inner_pair = pair.into_inner().try_next()?;
            build_subExpression_from_pair(inner_pair)
        }
        Rule::boolExpr => build_boolExpr_from_pair(pair),
        unknown => bail!("Got unknown pair: {:?}", unknown),
    }
}

fn build_and_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.try_next()?;

    match inner_pair.next() {
        None => build_or_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_boolExpr_from_pair(left_side_pair)?;
            let rside = build_boolExpr_from_pair(right_side_pair)?;

            Ok(QueryExpression::AndOp(Box::new(lside), Box::new(rside)))
        }
    }
}

fn build_or_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.try_next()?;

    match inner_pair.next() {
        None => build_compareExpr_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_boolExpr_from_pair(left_side_pair)?;
            let rside = build_boolExpr_from_pair(right_side_pair)?;

            Ok(QueryExpression::OrOp(Box::new(lside), Box::new(rside)))
        }
    }
}

fn build_compareExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> Result<QueryExpression> {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.try_next()?;

    match inner_pair.next() {
        None => build_subExpression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.try_next()?;

            let lside = build_boolExpr_from_pair(left_side_pair)?;
            let rside = build_boolExpr_from_pair(right_side_pair)?;

            match operator_pair.as_str() {
                ">=" => Ok(QueryExpression::GreatEQ(Box::new(lside), Box::new(rside))),
                "<=" => Ok(QueryExpression::LessEQ(Box::new(lside), Box::new(rside))),
                "<" => Ok(QueryExpression::LessT(Box::new(lside), Box::new(rside))),
                ">" => Ok(QueryExpression::GreatT(Box::new(lside), Box::new(rside))),
                unknown_operator => bail!(
                    "Got unknown boolean operator: {}. Only able to match >=,<=,<,>",
                    unknown_operator
                ),
            }
        }
    }
}
