use crate::ModelObjects::expression_representation::BoolExpression;
use crate::ModelObjects::component;
use crate::ModelObjects::expression_representation::BoolExpression::Bool;
use super::super::DBMLib::lib;
use pest::state;
use crate::DBMLib::lib::rs_dbm_satisfies_i_EQUAL_j_bounds;

pub fn apply_constraints_to_state_pair(constraint: &BoolExpression, state_pair : &mut component::StatePair, is_state1 : bool) -> BoolExpression{
    match constraint {
        BoolExpression::AndOp(left, right) => {
            let left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            if let BoolExpression::Bool(val) = left{
                if !val {
                    return BoolExpression::Bool(false)
                }
            }
            let right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val && right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard && expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        },
        BoolExpression::OrOp(left, right) => {
            let left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            let right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val || right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard || expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        },
        BoolExpression::LessEQ(left, right) => {
            let computed_left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            let computed_right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {

                            let dim =state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatEQ(left, right) => {
            let computed_left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            let computed_right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(state_pair.get_zone(), dim, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::LessT(left, right) => {
            let computed_left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            let computed_right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            // int <= clock
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatT(left, right) => {
            let computed_left = apply_constraints_to_state_pair(&**left, state_pair, is_state1);
            let computed_right = apply_constraints_to_state_pair(&**right, state_pair, is_state1);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = state_pair.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(state_pair.get_zone(), dim, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::Parentheses(expr) => {
            return apply_constraints_to_state_pair(constraint, state_pair, is_state1)
        },
        BoolExpression::VarName(name) => {
           let state =  if is_state1 {
                state_pair.get_state1()
            } else {
               state_pair.get_state2()
           };
            if let Some(clock_index) = state.get_declarations().get_clocks().get(name.as_str()) {
                return BoolExpression::Clock(*clock_index)
            }
            if let Some(val) = state.get_declarations().get_ints().get(name.as_str()) {
                return BoolExpression::Int(*val)
            }
        },
        BoolExpression::Bool(val) => {
            return BoolExpression::Bool(*val)
        },
        BoolExpression::Int(val) => {
            return BoolExpression::Int(*val)
        },
        BoolExpression::Clock(index) => {
            return BoolExpression::Clock(*index)
        }
    }

    panic!("not implemented")
}


pub fn apply_constraints_to_state(guard : &BoolExpression, state : & component::State, zone : &mut [i32], dimensions : &u32) -> BoolExpression{
    match guard {
        BoolExpression::AndOp(left, right) => {
            let left = apply_constraints_to_state(&**left, state, zone, dimensions);
            if let BoolExpression::Bool(val) = left{
                if !val {
                    return BoolExpression::Bool(false)
                }
            }
            let right = apply_constraints_to_state(&**right, state, zone, dimensions);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val && right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard && expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard && expression")
                }
            }
        },
        BoolExpression::OrOp(left, right) => {
            let left = apply_constraints_to_state(&**left, state, zone, dimensions);
            let right = apply_constraints_to_state(&**right, state, zone, dimensions);

            match left {
                BoolExpression::Bool(left_val) => {
                    match right {
                        BoolExpression::Bool(right_val) => {
                            return BoolExpression::Bool(left_val || right_val)
                        },
                        _ => {
                            panic!("expected bool in apply guard || expression")
                        }
                    }
                },
                _ => {
                    panic!("expected bool in apply guard || expression")
                }
            }
        },
        BoolExpression::LessEQ(left, right) => {
            let computed_left = apply_constraints_to_state(&**left, state, zone, dimensions);
            let computed_right = apply_constraints_to_state(&**right, state, zone, dimensions);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {

                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatEQ(left, right) => {
            let computed_left = apply_constraints_to_state(&**left, state, zone, dimensions);
            let computed_right = apply_constraints_to_state(&**right, state, zone, dimensions);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LTE_constraint(zone, dim, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::LessT(left, right) => {
            let computed_left = apply_constraints_to_state(&**left, state, zone, dimensions);
            let computed_right = apply_constraints_to_state(&**right, state, zone, dimensions);

            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, left_index, right_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, left_index, 0, right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            // int <= clock
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, 0, right_index, -1 * left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val <= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::GreatT(left, right) => {
            let computed_left = apply_constraints_to_state(&**left, state, zone, dimensions);
            let computed_right = apply_constraints_to_state(&**right, state, zone, dimensions);
            match computed_left {
                BoolExpression::Clock(left_index) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, right_index, left_index, 0);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, 0, left_index, -1 * right_val);
                            return BoolExpression::Bool(result)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                BoolExpression::Int(left_val) => {
                    match computed_right {
                        BoolExpression::Clock(right_index) => {
                            let dim = *state.get_dimensions();
                            let result = lib::rs_dbm_add_LT_constraint(zone, dim, right_index, 0, left_val);
                            return BoolExpression::Bool(result)
                        },
                        BoolExpression::Int(right_val) => {
                            return BoolExpression::Bool(left_val >= right_val)
                        },
                        _ => {
                            panic!("invalid type in LEQ expression in guard")
                        }
                    }
                },
                _ => {
                    panic!("invalid type in LEQ expression in guard")
                }
            }
        },
        BoolExpression::Parentheses(expr) => {
            return apply_constraints_to_state(guard, state, zone, dimensions)
        },
        BoolExpression::VarName(name) => {
            if let Some(clock_index) = state.get_declarations().get_clocks().get(name.as_str()) {
                return BoolExpression::Clock(*clock_index)
            }
            if let Some(val) = state.get_declarations().get_ints().get(name.as_str()) {
                return BoolExpression::Int(*val)
            }
        },
        BoolExpression::Bool(val) => {
            return BoolExpression::Bool(*val)
        },
        BoolExpression::Int(val) => {
            return BoolExpression::Int(*val)
        },
        BoolExpression::Clock(index) => {
            return BoolExpression::Clock(*index)
        }
    }

    panic!("not implemented")
}
