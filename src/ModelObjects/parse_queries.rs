extern crate pest;
use pest::error::Error;
use pest::Parser;
use super::super::ModelObjects::expression_representation::QueryExpression;
use super::super::ModelObjects::expression_representation::BoolExpression;
use serde::export::Option::Some;

#[derive(Parser)]
#[grammar = "ModelObjects/grammars/query_grammar.pest"]
pub struct QueryParser;



pub fn parse(edge_attribute_str : &str) -> Result<QueryExpression, Error<Rule>>{
    let mut pairs = QueryParser::parse(Rule::query, edge_attribute_str).unwrap_or_else(|e| panic!("Could not parse as rule with error: {}", e));
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::query => {
            Ok(build_query_from_pair(pair))
        }
        err => {panic!("Unable to match query string as rule: {:?}", err)}
    }
}

pub fn build_query_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression{
    let pair = pair.into_inner().next().unwrap();
    let pair_span = pair.as_span();

    //check if we have an empty pair
    if pair_span.start() == pair_span.end()  {
        return QueryExpression::Bool(true)
    }

    match pair.as_rule(){
        Rule::refinement => {
            build_refinement_from_pair(pair)
        },
        Rule::consistency => {
            QueryExpression::Consistency(Box::new(build_expression_from_pair(pair)))
        },
        Rule::implementation => {
            QueryExpression::Implementation(Box::new(build_expression_from_pair(pair)))
        },
        Rule::determinism => {
            QueryExpression::Determinism(Box::new(build_expression_from_pair(pair)))
        },
        Rule::specification => {
            QueryExpression::Specification(Box::new(build_expression_from_pair(pair)))
        },
        Rule::logicFormulas => {
            build_expression_from_pair(pair.into_inner().next().unwrap())
        },
        unknown => panic!("Got unknown pair: {:?}", unknown)
    }
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression{
    match pair.as_rule(){
        Rule::term => {
            build_term_from_pair(pair)
        },
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            QueryExpression::Parentheses(Box::new(build_expression_from_pair(inner_pair)))
        },
        Rule::specificationFeature => {
            build_specificationFeature_from_pair(pair)
        },
        unknown => panic!("Got unknown pair: {:?}", unknown)
    }
}

fn build_refinement_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();
    let right_side_pair = inner_pair.next().unwrap();

    let lside = build_expression_from_pair(left_side_pair);
    let rside = build_expression_from_pair(right_side_pair);

    return QueryExpression::Refinement(Box::new(lside), Box::new(rside))
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

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> QueryExpression{
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule(){
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>(){
                QueryExpression::Bool(n)
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>(){
                QueryExpression::Int(n)
            } else{
                build_term_from_pair(inner_pair)
            }
        },
        Rule::var => {
            build_var_from_pair(pair)
        },
        err => panic!("Unable to match: {:?} as rule atom or variable", err),
    }
}

fn build__bool_term_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule(){
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>(){
                BoolExpression::Bool(n)
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>(){
                BoolExpression::Int(n)
            } else{
                build__bool_term_from_pair(inner_pair)
            }
        },
        Rule::variable => {
            BoolExpression::VarName(inner_pair.as_str().trim().to_string())
        },
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
            let rside = build_subExpression_from_pair(right_side_pair);
            
            QueryExpression::ComponentExpression(Box::new(lside), Box::new(rside))
        }
    }
}

fn build_subExpression_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    match pair.as_rule(){
        Rule::term => {
            build__bool_term_from_pair(pair)
        },
        Rule::parenthesizedSubExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            BoolExpression::Parentheses(Box::new(build_subExpression_from_pair(inner_pair)))
        },
        Rule::notExpr => {
            let inner_pair = pair.into_inner().next().unwrap();
            BoolExpression::Not(Box::new(build_subExpression_from_pair(inner_pair)))
        },
        Rule::compExpr => {
            build_compareExpr_from_pair(pair)
        },
        unknown => panic!("Got unknown pair: {:?}", unknown)
    }
}

fn build_and_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
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

fn build_or_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
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

fn build_compareExpr_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
    let mut inner_pair = pair.into_inner();
    let left_side_pair = inner_pair.next().unwrap();

    match inner_pair.next() {
        None => build_expression_from_pair(left_side_pair),
        Some(operator_pair) => {
            let right_side_pair = inner_pair.next().unwrap();

            let lside = build_expression_from_pair(left_side_pair);
            let rside = build_expression_from_pair(right_side_pair);
            
            match operator_pair.as_str(){
                ">=" => {BoolExpression::GreatEQ(Box::new(lside), Box::new(rside))},
                "<=" => {BoolExpression::LessEQ(Box::new(lside), Box::new(rside))},
                "<" => {BoolExpression::LessT(Box::new(lside), Box::new(rside))},
                ">" => {BoolExpression::GreatT(Box::new(lside), Box::new(rside))},
                unknown_operator => panic!("Got unknown boolean operator: {}. Only able to match >=,<=,<,>", unknown_operator),
            }
        }
    }    
}