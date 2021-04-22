extern crate pest;
use crate::ModelObjects::representations::BoolExpression;
use pest::error::Error;
use pest::Parser;
use serde::export::Option::Some;
use serde::Deserialize;

///This file handles parsing the edges based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate
#[derive(Parser)]
#[grammar = "ModelObjects/grammars/edge_grammar.pest"]
pub struct EdgeParser;

#[derive(Debug, Clone, Deserialize)]
pub enum EdgeAttribute {
    Updates(Vec<Update>),
    Guard(BoolExpression),
}

#[derive(Debug, Clone, Deserialize, std::cmp::PartialEq)]
pub struct Update {
    variable: String,
    expression: BoolExpression,
}

#[allow(dead_code)]
impl Update {
    pub fn get_expression(&self) -> &BoolExpression {
        &self.expression
    }

    pub fn get_variable_name(&self) -> &str {
        return self.variable.as_str();
    }
}

pub fn parse(edge_attribute_str: &str) -> Result<EdgeAttribute, Error<Rule>> {
    let mut pairs = EdgeParser::parse(Rule::edgeAttribute, edge_attribute_str)
        .unwrap_or_else(|e| panic!("Could not parse as rule with error: {}", e));
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::edgeAttribute => Ok(build_edgeAttribute_from_pair(pair)),
        err => {
            panic!("Unable to match edgeAttribute string as rule: {:?}", err)
        }
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

            return EdgeAttribute::Guard(build_expression_from_pair(inner_pair));
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

            return EdgeAttribute::Updates(updates);
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

    return updates;
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

    return update;
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        // Rule::negation => {
        //     build_negation_from_pair(pair)
        // },
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            BoolExpression::Parentheses(Box::new(build_expression_from_pair(inner_pair)))
        }
        // Rule::numNegation => {
        //     let inner_pair = pair.into_inner().next().unwrap();
        //     BoolExpression::Negate(Box::new(build_expression_from_pair(inner_pair)))
        // },
        Rule::and => build_and_from_pair(pair),
        Rule::or => build_or_from_pair(pair),
        Rule::compareExpr => build_compareExpr_from_pair(pair),
        // Rule::sub_add => {
        //     build_sub_add_from_pair(pair)
        // },
        // Rule::mult_div_mod => {
        //     build_mult_div_mod_from_pair(pair)
        // },
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

// fn build_sub_add_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
//     let mut inner_pair = pair.into_inner();
//     let left_side_pair = inner_pair.next().unwrap();

//     match inner_pair.next() {
//         None => build_expression_from_pair(left_side_pair),
//         Some(operator_pair) => {
//             let right_side_pair = inner_pair.next().unwrap();

//             let lside = build_expression_from_pair(left_side_pair);
//             let rside = build_expression_from_pair(right_side_pair);

//             match operator_pair.as_str(){
//                 "+" => {BoolExpression::Add(Box::new(lside), Box::new(rside))},
//                 "-" => {BoolExpression::Subtract(Box::new(lside), Box::new(rside))},
//                 unknown_operator => panic!("Got unknown arithmetic operator: {}. Only able to handle +,-,/,*,%", unknown_operator),
//             }
//         }
//     }
// }

// fn build_mult_div_mod_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
//     let mut inner_pair = pair.into_inner();
//     let left_side_pair = inner_pair.next().unwrap();

//     match inner_pair.next() {
//         None => build_expression_from_pair(left_side_pair),
//         Some(operator_pair) => {
//             let right_side_pair = inner_pair.next().unwrap();

//             let lside = build_expression_from_pair(left_side_pair);
//             let rside = build_expression_from_pair(right_side_pair);

//             match operator_pair.as_str(){
//                 "/" => {BoolExpression::Divide(Box::new(lside), Box::new(rside))},
//                 "*" => {BoolExpression::Mul(Box::new(lside), Box::new(rside))},
//                 "%" => {BoolExpression::Modulo(Box::new(lside), Box::new(rside))}
//                 unknown_operator => panic!("Got unknown arithmetic operator: {}. Only able to handle +,-,/,*,%", unknown_operator),
//             }
//         }
//     }
// }

// fn build_negation_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
//     BoolExpression::Not(Box::new(build_expression_from_pair(pair.into_inner().next().unwrap())))
// }
