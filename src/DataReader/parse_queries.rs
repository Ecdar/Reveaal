extern crate pest;
use crate::ModelObjects::representations::QueryExpression;
use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "DataReader/grammars/query_grammar.pest"]
pub struct QueryParser;

///This file handles parsing the queries based on the abstract syntax described in the .pest files in the grammar folder
///For clarification see documentation on pest crate

pub fn parse(edge_attribute_str: &str) -> Result<QueryExpression, Error<Rule>> {
    let mut pairs = QueryParser::parse(Rule::query, edge_attribute_str)
        .unwrap_or_else(|e| panic!("Could not parse as rule with error: {}", e));
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::query => Ok(build_query_from_pair(pair)),
        err => {
            panic!("Unable to match query string as rule: {:?}", err)
        }
    }
}

pub fn build_query_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let pair = pair.into_inner().next().unwrap();
    let pair_span = pair.as_span();

    //check if we have an empty pair
    if pair_span.start() == pair_span.end() {
        return QueryExpression::Bool(true);
    }

    match pair.as_rule() {
        Rule::refinement => build_refinement_from_pair(pair),
        Rule::consistency => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Consistency(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::implementation => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Implementation(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::determinism => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Determinism(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::specification => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Specification(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::logicFormulas => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_expression_from_pair(inner_pair)
        }
        unknown => panic!("Got unknown pair: {:?}", unknown),
    }
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Parentheses(Box::new(build_expression_from_pair(inner_pair)))
        }
        Rule::specificationFeature => build_specificationFeature_from_pair(pair),
        Rule::possibly => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Possibly(Box::new(build_boolExpr_from_pair(inner_pair)))
        }
        Rule::invariantly => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Invariantly(Box::new(build_boolExpr_from_pair(inner_pair)))
        }
        Rule::eventuallyAlways => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::EventuallyAlways(Box::new(build_boolExpr_from_pair(inner_pair)))
        }
        Rule::potentially => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Potentially(Box::new(build_boolExpr_from_pair(inner_pair)))
        }
        Rule::expr => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_expression_from_pair(inner_pair)
        }
        Rule::terms => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_expression_from_pair(inner_pair)
        }
        unknown => panic!("Got unknown pair: {:?}", unknown),
    }
}

fn build_refinement_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();
    let right_side_pair = inner_pair.next().unwrap();

    let lside = build_expression_from_pair(left_side_pair);
    let rside = build_expression_from_pair(right_side_pair);

    return QueryExpression::Refinement(Box::new(lside), Box::new(rside));
}

fn build_specificationFeature_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_expression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.next().unwrap();

            let lside = build_expression_from_pair(left_side_pair);
            let rside = build_expression_from_pair(right_side_pair);

            match operator_pair.as_str(){
                "//" => {QueryExpression::Quotient(Box::new(lside), Box::new(rside))},
                "&&" => {QueryExpression::Conjunction(Box::new(lside), Box::new(rside))},
                "||" => {QueryExpression::Composition(Box::new(lside), Box::new(rside))},
                unknown_operator => panic!("Got unknown specification feature operator: {}. Only able to match '//', '&&' or '||'", unknown_operator),
            }
        }
    }
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>() {
                QueryExpression::Bool(n)
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>() {
                QueryExpression::Int(n)
            } else {
                build_term_from_pair(inner_pair)
            }
        }
        Rule::var => build_var_from_pair(inner_pair),
        err => panic!("Unable to match: {:?} as rule atom or variable", err),
    }
}

fn build_var_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    let lside = QueryExpression::VarName(left_side_pair.as_str().trim().to_string());

    match inner_pair.next() {
        None => lside,
        Some(right_side_pair) => {
            let inner_right_pair = right_side_pair.into_inner().next().unwrap();
            let rside = build_expression_from_pair(inner_right_pair);

            QueryExpression::ComponentExpression(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_boolExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    match pair.as_rule() {
        Rule::boolExpr => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_boolExpr_from_pair(inner_pair)
        }
        Rule::andExpr => build_and_from_pair(pair),
        Rule::orExpr => build_or_from_pair(pair),
        Rule::compExpr => build_compareExpr_from_pair(pair),
        Rule::subExpr => build_subExpression_from_pair(pair),
        unknown => panic!("Expected and pair but got unknown pair: {:?}", unknown),
    }
}

fn build_subExpression_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    match pair.as_rule() {
        Rule::term => build_term_from_pair(pair),
        Rule::parenthesizedSubExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Parentheses(Box::new(build_subExpression_from_pair(inner_pair)))
        }
        Rule::notExpr => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Not(Box::new(build_subExpression_from_pair(inner_pair)))
        }
        Rule::subExpr => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_subExpression_from_pair(inner_pair)
        }
        Rule::boolExpr => build_boolExpr_from_pair(pair),
        unknown => panic!("Got unknown pair: {:?}", unknown),
    }
}

fn build_and_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_or_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_boolExpr_from_pair(left_side_pair);
            let rside = build_boolExpr_from_pair(right_side_pair);

            QueryExpression::AndOp(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_or_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_compareExpr_from_pair(left_side_pair),
        Some(right_side_pair) => {
            let lside = build_boolExpr_from_pair(left_side_pair);
            let rside = build_boolExpr_from_pair(right_side_pair);

            QueryExpression::OrOp(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_compareExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_subExpression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.next().unwrap();

            let lside = build_boolExpr_from_pair(left_side_pair);
            let rside = build_boolExpr_from_pair(right_side_pair);

            match operator_pair.as_str() {
                ">=" => QueryExpression::GreatEQ(Box::new(lside), Box::new(rside)),
                "<=" => QueryExpression::LessEQ(Box::new(lside), Box::new(rside)),
                "<" => QueryExpression::LessT(Box::new(lside), Box::new(rside)),
                ">" => QueryExpression::GreatT(Box::new(lside), Box::new(rside)),
                unknown_operator => panic!(
                    "Got unknown boolean operator: {}. Only able to match >=,<=,<,>",
                    unknown_operator
                ),
            }
        }
    }
}
