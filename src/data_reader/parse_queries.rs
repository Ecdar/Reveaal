extern crate pest;

use crate::model_objects::expressions::{
    ComponentVariable, OperandExpression, QueryExpression, SaveExpression, StateExpression,
    SystemExpression,
};
use crate::model_objects::Query;

use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;

#[derive(Parser)]
#[grammar = "data_reader/grammars/query_grammar.pest"]
pub struct QueryParser;

lazy_static! {
    static ref PRATT: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::qoutient_op, Assoc::Left))
        .op(Op::infix(Rule::composition_op, Assoc::Left))
        .op(Op::infix(Rule::conjunction_op, Assoc::Left));
}

//This file handles parsing the queries based on the abstract syntax described in the .pest files in the grammar folder
//For clarification see documentation on pest crate

pub fn parse_system(pair: pest::iterators::Pair<Rule>) -> SystemExpression {
    PRATT
        .map_primary(|pair| match pair.as_rule() {
            Rule::expr => parse_system(pair),
            Rule::component => {
                let mut pairs = pair.into_inner();
                let comp_name = pairs.next().unwrap().as_str().to_string();
                let special_id = pairs.next().map(|it| it.as_str().to_string());

                SystemExpression::Component(comp_name, special_id)
            }
            Rule::variable_name => SystemExpression::Component(pair.as_str().to_string(), None),

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

fn parse_state(pair: pest::iterators::Pair<Rule>) -> StateExpression {
    PRATT
        .map_primary(|pair| match pair.as_rule() {
            Rule::andExpr | Rule::orExpr => {
                let rule = pair.as_rule();

                let mut inner = pair.into_inner();
                let len = inner.clone().count();
                if len == 1 {
                    return parse_state(inner.next().unwrap());
                }
                match rule {
                    Rule::andExpr => StateExpression::AND(inner.map(parse_state).collect()),
                    Rule::orExpr => StateExpression::OR(inner.map(parse_state).collect()),
                    _ => unreachable!(),
                }
            }
            Rule::notExpr => {
                StateExpression::NOT(Box::new(parse_state(pair.into_inner().next().unwrap())))
            }
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

fn parse_query(pair: pest::iterators::Pair<Rule>) -> QueryExpression {
    PRATT
        .map_primary(|pair| {
            let query = match pair.as_rule() {
                Rule::refinement => {
                    let mut pairs = pair.into_inner();
                    let system1 = parse_system(pairs.next().unwrap());
                    let system2 = parse_system(pairs.next().unwrap());
                    QueryExpression::Refinement(system1, system2)
                }
                Rule::consistency => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    QueryExpression::Consistency(system)
                }
                Rule::reachability => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    let to = parse_state(pairs.next_back().unwrap());
                    let from = pairs.next().map(parse_state);

                    QueryExpression::Reachability { system, from, to }
                }
                Rule::implementation => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    QueryExpression::Implementation(system)
                }
                Rule::determinism => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    QueryExpression::Determinism(system)
                }
                Rule::specification => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    QueryExpression::Specification(system)
                }
                Rule::getComponent => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::GetComponent(SaveExpression { system, name })
                }
                Rule::prune => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::Prune(SaveExpression { system, name })
                }
                Rule::bisim => {
                    let mut pairs = pair.into_inner();
                    let system = parse_system(pairs.next().unwrap());
                    let name = pairs.next().map(|it| it.as_str().to_string());
                    QueryExpression::BisimMinim(SaveExpression { system, name })
                }
                Rule::syntax => {
                    QueryExpression::Syntax(parse_system(pair.into_inner().next().unwrap()))
                }
                _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
            };
            query
        })
        .parse(pair.into_inner())
}

fn parse_queries(pair: pest::iterators::Pair<Rule>) -> Vec<QueryExpression> {
    match pair.as_rule() {
        Rule::queryList => {
            let inner = pair.into_inner();
            inner.map(parse_query).collect()
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
    parse_to_expression_tree(input)
        .expect("Parsing failed")
        .into_iter()
        .map(|q| Query {
            query: Option::from(q),
            comment: "".to_string(),
        })
        .collect()
}

pub fn parse_to_expression_tree(input: &str) -> Result<Vec<QueryExpression>, String> {
    let mut pairs = match QueryParser::parse(Rule::queries, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_queries(pairs.next().unwrap());

    Ok(result)
}

pub fn parse_to_system_expr(input: &str) -> Result<SystemExpression, String> {
    let mut pairs = match QueryParser::parse(Rule::expr, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_system(pairs.next().unwrap());

    Ok(result)
}

pub fn parse_to_state_expr(input: &str) -> Result<StateExpression, String> {
    let mut pairs = match QueryParser::parse(Rule::state, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let result = parse_state(pairs.next().unwrap());

    Ok(result)
}

// which is the parse_queries::parse_to_expression_tree() function.
#[cfg(test)]
pub mod tests {
    use crate::extract_system_rep::create_executable_query;
    use crate::model_objects::expressions::{StateExpression, SystemExpression};
    use crate::parse_queries::parse_to_state_expr;
    use crate::system::system_recipe::{get_system_recipe, SystemRecipe};
    use crate::transition_systems::TransitionSystem;
    use crate::{parse_queries, system, xml_parser, JsonProjectLoader, XmlProjectLoader};
    use edbm::util::constraints::ClockIndex;
    use test_case::test_case;

    // These tests ensure the parser/grammar can parse strings of the reachability syntax,
    // Currently only reachability-focused queries are tested

    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";

    /// Helper function which converts a string to an option<box<BoolExpression>> by replacing ',' with "&&" and using the invariant parser.
    fn string_to_state_expr(string: &str) -> StateExpression {
        parse_to_state_expr(string).unwrap()
    }

    /// Helper function to create a transition system and a machine (system recipe)
    pub fn create_system_recipe_and_machine(
        model: SystemExpression,
        folder_path: &str,
    ) -> (Box<SystemRecipe>, Box<dyn TransitionSystem>) {
        let mut comp_loader = if xml_parser::is_xml_project(folder_path) {
            XmlProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        } else {
            JsonProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        }
        .to_comp_loader();
        let mut dim: ClockIndex = 0;
        let mut quotient_index = None;
        let machine =
            get_system_recipe(&model, &mut (*comp_loader), &mut dim, &mut quotient_index).unwrap();
        //TODO:: - unwrap might not be the best way to handle this
        let system = machine.clone().compile(dim).unwrap();
        (machine, system)
    }

    #[test_case("reachability: Hi @ Hi.L1 && Hi.y<3 -> Hi.L2 && Hi.y<2"; "only 1 machine, start/end location and clock restriction")]
    #[test_case("reachability: Hi[1] && Hi[2] @ Hi[1].L1 && Hi[2].L1 && Hi[1].y<3 -> Hi[1].L2 && Hi[1].y<2"; "2 machine, start/end location and clock restriction")]
    fn query_grammar_test_valid_queries(parser_input: &str) {
        // This tests that the grammar accepts this string, and does not panic:
        assert!(super::parse_to_expression_tree(parser_input).is_ok());
    }

    #[test_case("reachability: Hi @ L1 && Hi.y<3 -> L2 && Hi.y<2"; "No component prefix on location")]
    #[test_case("reachability: Hi @ Hi.L1 && y<3 -> Hi.L2 && y<2"; "No component prefix on clock")]
    fn query_grammar_test_invalid_queries(parser_input: &str) {
        // This tests that the grammar does NOT accept this string and panics:
        assert!(super::parse_to_expression_tree(parser_input).is_err());
    }

    // These tests check that the parser only accepts clock variable arguments with existing clock variables.
    // i.e. check that the variables exist in the model.
    // The model/sample used is samples/json/EcdarUniversity/adm2.json
    // This model/sample contains the clock variables "x" and "y".
    // And locations "L20", "L21" ... "L23".
    #[test_case("Adm2.L20 && Adm2.u>1";
    "The clock variable u in the state does not exist in the model")]
    #[test_case("Adm2.L20 && Adm2.uwu>2";
    "The clock variable uwu in the state does not exist in the model")]
    fn query_parser_checks_invalid_clock_variables(clock_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);

        let (machine, system) = create_system_recipe_and_machine(mock_model, FOLDER_PATH);

        let mock_state = string_to_state_expr(clock_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_err());
    }

    #[test_case("true";
    "State gets parsed as partial")]
    #[test_case("Adm2.L20 && Adm2.x>1";
    "The clock variable x in state exists in the model")]
    #[test_case("Adm2.L20 && Adm2.y<1";
    "The clock variable y in state exists in the model")]
    fn query_parser_checks_valid_clock_variables(clock_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) = create_system_recipe_and_machine(mock_model, FOLDER_PATH);

        let mock_state = string_to_state_expr(clock_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_ok());
    }
    #[test_case("Adm2.L19";
    "The location L19 in the state does not exist in the model")]
    #[test_case("Adm2.NOTCORRECTNAME";
    "The location NOTCORRECTNAME in the state does not exist in the model")]
    fn query_parser_checks_invalid_location(location_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) = create_system_recipe_and_machine(mock_model, FOLDER_PATH);

        let mock_state = string_to_state_expr(location_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_err());
    }

    #[test_case("Adm2.L20";
    "The location L20 in the state exists in the model")]
    #[test_case("Adm2.L23";
    "The location L23 in the state exists in the model")]
    fn query_parser_checks_valid_locations(location_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) = create_system_recipe_and_machine(mock_model, FOLDER_PATH);

        let mock_state = string_to_state_expr(location_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_ok());
    }

    //These tests check the parsers validity checks, like an equal amount of parameters
    /*
     #[test_case("reachability: Adm2 -> [L21, _](); [L20]()";
     "Amount of machine and amount of location args does not match: 1 machine, 2 start-loc-args")]
     #[test_case("reachability: Adm2 -> [L21](); [L20, _]()";
     "Amount of machine and amount of location args does not match: 1 machine, 2 end-loc-args")]
     #[test_case("reachability: Adm2 || Machine -> [L21](); [L20]()";
     "Amount of machine and amount of location args does not match: 2 machines, 1 loc-arg")]
    */
    #[test_case("reachability: Adm2 @ Adm2.L21 && Adm2.L20 -> Adm2.L20";
    "Amount of machine and amount of location args does not match: 1 machine, 2 start-loc-args")]
    #[test_case("reachability: Adm2 @ Adm2.L21 -> Adm2.L20 && Adm2.L21";
    "Amount of machine and amount of location args does not match: 1 machine, 2 end-loc-args")]
    #[test_case("reachability: Adm2 || Machine @ Adm2.L21 -> Adm2.L20";
    "Amount of machine and amount of location args does not match: 2 machines, 1 loc-arg")]
    // The amount of locations given as parameters must be the same as the amount of machines.
    fn invalid_amount_of_location_and_machine_args(parser_input: &str) {
        let folder_path = "samples/json/EcdarUniversity".to_string();
        let mut comp_loader = if xml_parser::is_xml_project(&folder_path) {
            XmlProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        } else {
            JsonProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        }
        .to_comp_loader();
        // Make query:
        let q = parse_queries::parse_to_query(parser_input);
        let queries = q.first().unwrap();

        // Runs the "validate_reachability" function from extract_system_rep, which we wish to test.
        assert!(create_executable_query(queries, &mut *comp_loader).is_err());
    }
    #[test_case("reachability: Adm2 @ Adm2.L21 -> Adm2.L20";
    "Matching amount of locations and machines: 1 machine, 1 loc")]
    #[test_case("reachability: Adm2 || Machine @ Adm2.L21 && Machine.L4 -> Adm2.L20 && Machine.L5";
    "Matching amount of locations and machines: 2 machines, 2 loc args")]
    // The amount of locations given as parameters must be the same as the amount of machines.
    fn valid_amount_of_location_and_machine_args(parser_input: &str) {
        let folder_path = "samples/json/EcdarUniversity".to_string();
        let mut comp_loader = if xml_parser::is_xml_project(&folder_path) {
            XmlProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        } else {
            JsonProjectLoader::new_loader(folder_path, crate::DEFAULT_SETTINGS)
        }
        .to_comp_loader();
        // Make query:
        let q = parse_queries::parse_to_query(parser_input);
        let queries = q.first().unwrap();

        // Runs the "validate_reachability" function from extract_system_rep, which we wish to test.
        assert!(create_executable_query(queries, &mut *comp_loader).is_ok());
    }
}
