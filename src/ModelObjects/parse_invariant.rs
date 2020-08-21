extern crate pest;
use pest::error::Error;
use pest::Parser;
use super::super::ModelObjects::representations::BoolExpression;
use serde::export::Option::Some;

#[derive(Parser)]  
#[grammar = "ModelObjects/grammars/invariant_grammar.pest"]
pub struct InvariantParser;



pub fn parse(edge_attribute_str : &str) -> Result<BoolExpression, Error<Rule>>{
    let mut pairs = InvariantParser::parse(Rule::invariant, edge_attribute_str).unwrap_or_else(|e| panic!("Could not parse as rule with error: {}", e));
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::invariant => {
            Ok(build_invariant_from_pair(pair))
        }
        err => {panic!("Unable to match invariant string as rule: {:?}", err)}
    }
}

pub fn build_invariant_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule(){
        Rule::andExpr =>{
            let pair_span = pair.as_span();

            //check if we have an empty pair
            if pair_span.start() == pair_span.end()  {
                return BoolExpression::Bool(true)
            }

            let mut inner_pairs = pair.into_inner();
            let inner_pair = inner_pairs.next().unwrap();

            return build_expression_from_pair(inner_pair)
        }
        _ => panic!("Unable to match: {:?} as rule, guard", pair)
    }
}

fn build_expression_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
    match pair.as_rule(){
        Rule::term => {
            build_term_from_pair(pair)
        },
        Rule::parenthesizedExp => {
            let inner_pair = pair.into_inner().next().unwrap();
            BoolExpression::Parentheses(Box::new(build_expression_from_pair(inner_pair)))
        },
        Rule::andExpr => {
            build_and_from_pair(pair)
        },
        Rule::orExpr => {
            build_or_from_pair(pair)
        },
        Rule::compareExpr => {
            build_compareExpr_from_pair(pair)
        },
        Rule::terms => {
            build_expression_from_pair(pair.into_inner().next().unwrap())
        },
        unknown => panic!("Got unknown pair: {:?}", unknown)
    }
}

fn build_term_from_pair(pair: pest::iterators::Pair<Rule>) -> BoolExpression{
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule(){
        Rule::atom => {
            if let Ok(n) = inner_pair.as_str().trim().parse::<bool>(){
                BoolExpression::Bool(n)
            } else if let Ok(n) = inner_pair.as_str().trim().parse::<i32>(){
                BoolExpression::Int(n)
            } else{
                build_term_from_pair(inner_pair)
            }
        },
        Rule::variable => {
            BoolExpression::VarName(inner_pair.as_str().trim().to_string())
        },
        err => panic!("Unable to match: {:?} as rule atom or variable", err),
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