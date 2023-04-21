extern crate pest;

use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;

use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;

#[derive(Parser)]
#[grammar = "DataReader/grammars/query_grammar.pest"]
pub struct QueryParser;

///This file handles parsing the queries based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate
#[derive(Debug, Clone)]
pub enum QueryExpr {
    Refinement(SystemExpr, SystemExpr),
    Consistency(SystemExpr),
    Reachability {
        system: SystemExpr,
        from: Option<StateExpr>,
        to: StateExpr,
    },
    Implementation(SystemExpr),
    Determinism(SystemExpr),
    Specification(SystemExpr),
    GetComponent(SaveExpr),
    Prune(SaveExpr),
    BisimMinim(SaveExpr),
}

#[derive(Debug, Clone)]
pub struct SaveExpr {
    pub system: SystemExpr,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StateExpr {
    LEQ(OperandExpr, OperandExpr),
    GEQ(OperandExpr, OperandExpr),
    EQ(OperandExpr, OperandExpr),
    LT(OperandExpr, OperandExpr),
    GT(OperandExpr, OperandExpr),
    AND(Vec<StateExpr>),
    OR(Vec<StateExpr>),
    Location(ComponentVariable),
    NOT(Box<StateExpr>),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum OperandExpr {
    Number(i32),
    Clock(ComponentVariable),
    Difference(Box<OperandExpr>, Box<OperandExpr>),
    Sum(Box<OperandExpr>, Box<OperandExpr>),
}

#[derive(Debug, Clone)]
pub struct ComponentVariable {
    /// Fx. `"A[Temp].x"` -> `"A"`
    pub component: String,
    /// Fx. `"A[Temp].x"` -> `Some("Temp")` or `"A.x"` -> `None`
    pub special_id: Option<String>,
    /// Fx. `"A[Temp].x"` -> `"x"`
    pub variable: String,
}

#[derive(Debug, Clone)]
pub enum SystemExpr {
    /// Fx. `"A[Temp]"` -> `Component("A", Some("Temp"))`
    /// Fx. `"A"` -> `Component("A", None)`
    Component(String, Option<String>),
    Quotient(Box<SystemExpr>, Box<SystemExpr>),
    Composition(Box<SystemExpr>, Box<SystemExpr>),
    Conjunction(Box<SystemExpr>, Box<SystemExpr>),
}

pub fn parse_system(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> SystemExpr {
    pratt
        .map_primary(|pair| match pair.as_rule() {
            Rule::expr => parse_system(pair, pratt),
            Rule::component => {
                println!("variable: {}", pair.as_str());
                let mut pairs = pair.into_inner();
                let comp_name = pairs.next().unwrap().as_str().to_string();
                let special_id = pairs.next().map(|it| it.as_str().to_string());

                SystemExpr::Component(comp_name, special_id)
            }
            _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
        })
        .map_infix(|left, op, right| {
            let left = Box::new(left);
            let right = Box::new(right);
            match op.as_rule() {
                Rule::qoutient_op => SystemExpr::Quotient(left, right),
                Rule::composition_op => SystemExpr::Composition(left, right),
                Rule::conjunction_op => SystemExpr::Conjunction(left, right),
                _ => unreachable!(),
            }
        })
        .parse(pair.into_inner())
}

pub fn parse_operand(pair: pest::iterators::Pair<Rule>) -> OperandExpr {
    match pair.as_rule() {
        Rule::int => OperandExpr::Number(pair.as_str().parse().unwrap()),
        Rule::variable => OperandExpr::Clock(comp_var_from_variable_pair(pair)),
        Rule::boolDiff => {
            let mut pairs = pair.into_inner();

            let mut expr = parse_operand(pairs.next().unwrap());

            while pairs.peek().is_some() {
                let op = pairs.next().unwrap().as_rule();
                let operand = parse_operand(pairs.next().unwrap());

                match op {
                    Rule::sub_op => {
                        expr = OperandExpr::Difference(Box::new(expr), Box::new(operand));
                    }
                    Rule::sum_op => {
                        expr = OperandExpr::Sum(Box::new(expr), Box::new(operand));
                    }
                    _ => unreachable!(),
                }
            }

            expr
        }
        _ => unreachable!(),
    }
}

pub fn parse_state(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> StateExpr {
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
                        StateExpr::AND(inner.map(|pair| parse_state(pair, pratt)).collect())
                    }
                    Rule::orExpr => {
                        StateExpr::OR(inner.map(|pair| parse_state(pair, pratt)).collect())
                    }
                    _ => unreachable!(),
                }
            }
            Rule::notExpr => StateExpr::NOT(Box::new(parse_state(
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
                    Rule::leq_op => StateExpr::LEQ(first, second),
                    Rule::geq_op => StateExpr::GEQ(first, second),
                    Rule::eq_op => StateExpr::EQ(first, second),
                    Rule::lt_op => StateExpr::LT(first, second),
                    Rule::gt_op => StateExpr::GT(first, second),
                    _ => unreachable!(),
                }
            }
            Rule::locExpr => StateExpr::Location(comp_var_from_variable_pair(
                pair.into_inner().next().unwrap(),
            )),
            Rule::bool_true => StateExpr::Bool(true),
            Rule::bool_false => StateExpr::Bool(false),
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

fn parse_query(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> QueryExpr {
    pratt
        .map_primary(|pair| {
            let query = match pair.as_rule() {
                Rule::refinement => {
                    let mut pairs = pair.into_inner();
                    let system1 = parse_system(pairs.next().unwrap(), pratt);
                    let system2 = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpr::Refinement(system1, system2)
                }
                Rule::consistency => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpr::Consistency(system)
                }
                Rule::reachability => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let to = parse_state(pairs.next_back().unwrap(), pratt);
                    let from = pairs.next().map(|pair| parse_state(pair, pratt));

                    QueryExpr::Reachability { system, from, to }
                }
                Rule::implementation => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpr::Implementation(system)
                }
                Rule::determinism => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpr::Determinism(system)
                }
                Rule::specification => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    QueryExpr::Specification(system)
                }
                Rule::getComponent => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpr::GetComponent(SaveExpr { system, name })
                }
                Rule::prune => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpr::Prune(SaveExpr { system, name })
                }
                Rule::bisim => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap(), pratt);
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpr::BisimMinim(SaveExpr { system, name })
                }
                _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
            };
            query
        })
        .parse(pair.into_inner())
}

fn parse_queries(pair: pest::iterators::Pair<Rule>, pratt: &PrattParser<Rule>) -> Vec<QueryExpr> {
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

    for res in &result {
        println!("{:?}", res);
    }

    Ok(result.into_iter().map(|_it| todo!()).collect())
}

pub fn parse_to_system_expr(input: &str) -> Result<SystemExpr, String> {
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
