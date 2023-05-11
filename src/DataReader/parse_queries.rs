extern crate pest;

use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::{
    ComponentVariable, OperandExpression, QueryExpression, SaveExpression, StateExpression,
    SystemExpression,
};

use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;

#[derive(Parser)]
#[grammar = "DataReader/grammars/query_grammar.pest"]
pub struct QueryParser;

///This file handles parsing the queries based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate

pub fn parse_system(
    pair: pest::iterators::Pair<Rule>,
    pratt: &PrattParser<Rule>,
) -> SystemExpression {
    pratt
        .map_primary(|pair| match pair.as_rule() {
            Rule::expr => parse_system(pair, pratt),
            Rule::component => {
                let mut pairs = pair.into_inner();
                let comp_name = pairs.next().unwrap().as_str().to_string();
                let special_id = pairs.next().map(|it| it.as_str().to_string());

                SystemExpression::Component(comp_name, special_id)
            }
            _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
        })
        .map_infix(|left, op, right| {
            let left = Box::new(left);
            let right = Box::new(right);
            match op.as_rule() {
                Rule::qoutient_op => SystemExpression::Quotient(left, right),
                Rule::composition_op => SystemExpression::Composition(left, right),
                Rule::conjunction_op => SystemExpression::Conjunction(left, right),
                _ => unreachable!(),
            }
        })
        .parse(pair.into_inner())
}

pub fn parse_operand(pair: pest::iterators::Pair<Rule>) -> OperandExpression {
    match pair.as_rule() {
        Rule::int => OperandExpression::Number(pair.as_str().parse().unwrap()),
        Rule::variable => OperandExpression::Clock(comp_var_from_variable_pair(pair)),
        Rule::boolDiff => {
            let mut pairs = pair.into_inner();

            let mut expr = parse_operand(pairs.next().unwrap());

            while pairs.peek().is_some() {
                let op = pairs.next().unwrap().as_rule();
                let operand = parse_operand(pairs.next().unwrap());

                match op {
                    Rule::sub_op => {
                        expr = OperandExpression::Difference(Box::new(expr), Box::new(operand));
                    }
                    Rule::sum_op => {
                        expr = OperandExpression::Sum(Box::new(expr), Box::new(operand));
                    }
                    _ => unreachable!(),
                }
            }

            expr
        }
        _ => unreachable!(),
    }
}

fn parse_state(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> StateExpression {
    pratt
        .map_primary(|pair| match pair.as_rule() {
            Rule::andExpr | Rule::orExpr => {
                let rule = pair.as_rule();

                let mut inner = pair.into_inner();
                let len = inner.clone().count();
                if len == 1 {
                    return parse_state(inner.next().unwrap(), pratt);
                }
                match rule {
                    Rule::andExpr => {
                        StateExpression::AND(inner.map(|pair| parse_state(pair, pratt)).collect())
                    }
                    Rule::orExpr => {
                        StateExpression::OR(inner.map(|pair| parse_state(pair, pratt)).collect())
                    }
                    _ => unreachable!(),
                }
            }
            Rule::notExpr => StateExpression::NOT(Box::new(parse_state(
                pair.into_inner().next().unwrap(),
                pratt,
            ))),
            Rule::compExpr => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                let op = pairs.next().unwrap();
                let second = pairs.next().unwrap();

                let first = parse_operand(first);
                let second = parse_operand(second);
                match op.as_rule() {
                    Rule::leq_op => StateExpression::LEQ(first, second),
                    Rule::geq_op => StateExpression::GEQ(first, second),
                    Rule::eq_op => StateExpression::EQ(first, second),
                    Rule::lt_op => StateExpression::LT(first, second),
                    Rule::gt_op => StateExpression::GT(first, second),
                    _ => unreachable!(),
                }
            }
            Rule::locExpr => StateExpression::Location(comp_var_from_variable_pair(
                pair.into_inner().next().unwrap(),
            )),
            Rule::bool_true => StateExpression::Bool(true),
            Rule::bool_false => StateExpression::Bool(false),
            _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
        })
        .parse(pair.into_inner())
}

fn comp_var_from_variable_pair(pair: pest::iterators::Pair<Rule>) -> ComponentVariable {
    let mut pairs = pair.into_inner();
    let mut comp_pairs = pairs.next().unwrap().into_inner();
    let variable = pairs.next().unwrap().as_str().to_string();
    let component = comp_pairs.next().unwrap().as_str().to_string();
    let special_id = comp_pairs.next().map(|it| it.as_str().to_string());

    ComponentVariable {
        component,
        special_id,
        variable,
    }
}

fn parse_query(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> QueryExpression {
    pratt
        .map_primary(|pair| {
            let query = match pair.as_rule() {
                Rule::refinement => {
                    let mut pairs = pair.into_inner();
                    let system1 = parse_system(pairs.next().unwrap(), pratt);
                    let system2 = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpression::Refinement(system1, system2)
                }
                Rule::consistency => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpression::Consistency(system)
                }
                Rule::reachability => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let to = parse_state(pairs.next_back().unwrap(), pratt);
                    let from = pairs.next().map(|pair| parse_state(pair, pratt));

                    QueryExpression::Reachability { system, from, to }
                }
                Rule::implementation => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpression::Implementation(system)
                }
                Rule::determinism => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpression::Determinism(system)
                }
                Rule::specification => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpression::Specification(system)
                }
                Rule::getComponent => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::GetComponent(SaveExpression { system, name })
                }
                Rule::prune => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::Prune(SaveExpression { system, name })
                }
                Rule::bisim => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::BisimMinim(SaveExpression { system, name })
                }
                _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
            };
            query
        })
        .parse(pair.into_inner())
}

fn parse_queries(
    pair: pest::iterators::Pair<Rule>,
    pratt: &PrattParser<Rule>,
) -> Vec<QueryExpression> {
    match pair.as_rule() {
        Rule::queryList => {
            let inner = pair.into_inner();
            inner.map(|pair| parse_query(pair, pratt)).collect()
        }
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

#[test]
pub fn test_parse() {
    let input = "reachability: A[AAB] || B @ init -> A[AAB].x - A[AAB].y - 5 + 5 <= 1";
    parse_to_expression_tree(input).unwrap();
}

pub fn parse_to_query(input: &str) -> Vec<Query> {
    let queries = parse_to_expression_tree(input).unwrap();
    queries
        .into_iter()
        .map(|q| Query {
            query: Option::from(q),
            comment: "".to_string(),
        })
        .collect()
}

pub fn parse_to_expression_tree(input: &str) -> Result<Vec<QueryExpression>, String> {
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::qoutient_op, Assoc::Left))
        .op(Op::infix(Rule::composition_op, Assoc::Left))
        .op(Op::infix(Rule::conjunction_op, Assoc::Left));

    let mut pairs = match QueryParser::parse(Rule::queries, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_queries(pairs.next().unwrap(), &pratt);

    Ok(result)
}

pub fn parse_to_system_expr(input: &str) -> Result<SystemExpression, String> {
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::qoutient_op, Assoc::Left))
        .op(Op::infix(Rule::composition_op, Assoc::Left))
        .op(Op::infix(Rule::conjunction_op, Assoc::Left));

    let mut pairs = match QueryParser::parse(Rule::expr, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_system(pairs.next().unwrap(), &pratt);

    Ok(result)
}

pub fn parse_to_state_expr(input: &str) -> Result<StateExpression, String> {
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::qoutient_op, Assoc::Left))
        .op(Op::infix(Rule::composition_op, Assoc::Left))
        .op(Op::infix(Rule::conjunction_op, Assoc::Left));

    let mut pairs = match QueryParser::parse(Rule::state, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_state(pairs.next().unwrap(), &pratt);

    Ok(result)
}
