use crate::DataReader::component_loader::ComponentLoader;
use crate::ModelObjects::component::Component;
use crate::ModelObjects::queries::Query;
use crate::ModelObjects::representations::QueryExpression;
use crate::System::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    ReachabilityExecutor, RefinementExecutor,
};
use crate::System::extract_state::get_state;
use std::cmp::max;

use crate::TransitionSystems::{
    CompiledComponent, Composition, Conjunction, Quotient, TransitionSystemPtr,
};

use crate::component::State;
use crate::ProtobufServer::services::query_request::settings::ReduceClocksLevel;
use crate::System::pruning;
use edbm::util::constraints::ClockIndex;
use log::debug;
use simple_error::bail;
use std::error::Error;
use std::ops::Range;

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    component_loader: &'a mut (dyn ComponentLoader + 'static),
) -> Result<Box<dyn ExecutableQuery + 'a>, Box<dyn Error>> {
    let mut dim: ClockIndex = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => {
                let mut quotient_index = None;
                let mut left = get_system_recipe(left_side, component_loader, &mut dim, &mut quotient_index);
                let mut right = get_system_recipe(right_side, component_loader, &mut dim, &mut quotient_index);
                //let height = max(left.height(), right.height()) + 1;
                if let Some(x) = &component_loader.get_settings().reduce_clocks_level {
                    match x { //TODO: Find dem clocks and parse
                        ReduceClocksLevel::Level(y) if *y >= 0 => {
                            //let heights = Heights::new(height, (*y) as usize);
                            let clocks: Vec<ClockReductionInstruction> = vec![]; // Found clocks
                            right.reduce_clocks(clocks.clone(), Some(left.reduce_clocks(clocks, None)));
                            //dim -= NUMBER OF CLOCKS
                        },
                        ReduceClocksLevel::All(true) =>{
                            //let heights = Heights::new(height, height);
                            let clocks: Vec<ClockReductionInstruction> = vec![]; // Found clocks
                            right.reduce_clocks(clocks.clone(), Some(left.reduce_clocks(clocks, None)));
                            //dim -= NUMBER OF CLOCKS
                        },
                        _ => (),
                    };
                }
                Ok(Box::new(RefinementExecutor {
                sys1: left.compile(dim)?,
                sys2: right.compile(dim)?,
            }))},
            QueryExpression::Reachability(automata, start, end) => {
                let machine = get_system_recipe(automata, component_loader, &mut dim, &mut None);
                let transition_system = machine.clone().compile(dim)?;

                validate_reachability_input(&machine, end)?;

                let start_state: State = if let Some(state) = start.as_ref() {
                    validate_reachability_input(&machine, state)?;
                    let state = get_state(state, &machine, &transition_system)?;
                    if state.get_location().id.is_partial_location() {
                        return Err("Start state is a partial state, which it must not be".into())
                    }
                    state
                }
                else {
                    match transition_system.get_initial_state() {
                        Some(state)=> state,
                        None => return Err("No start state in the transition system".into())
                    }
                };

                let end_state: State = get_state(end, &machine, &transition_system)?;

                Ok(Box::new(ReachabilityExecutor {
                    transition_system,
                    start_state,
                    end_state,
                }))
            },
            QueryExpression::Consistency(query_expression) => Ok(Box::new(ConsistencyExecutor {
                recipe: get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut None
                ),
                dim
            })),
            QueryExpression::Determinism(query_expression) => {
                Ok(Box::new(DeterminismExecutor {
                    system: get_system_recipe(
                        query_expression,
                        component_loader,
                        &mut dim,
                        &mut None,
                    ).compile(dim)?,
                }))
            },
            QueryExpression::GetComponent(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    Ok(Box::new(
                        GetComponentExecutor {
                            system: get_system_recipe(query_expression, component_loader, &mut dim, &mut None).compile(dim)?,
                            comp_name: comp_name.clone(),
                            component_loader,
                        }
                    ))
                }else{
                    bail!("Unexpected expression type")
                }
            }
            ,
            QueryExpression::Prune(save_as_expression) => {
                if let QueryExpression::SaveAs(query_expression, comp_name) = save_as_expression.as_ref() {
                    Ok(Box::new(
                        GetComponentExecutor {
                            system: pruning::prune_system(get_system_recipe(query_expression, component_loader, &mut dim, &mut None).compile(dim)?, dim),
                            comp_name: comp_name.clone(),
                            component_loader
                        }
                    ))
                }else{
                    bail!("Unexpected expression type")
                }
            }
            ,
            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => bail!("Not yet setup to handle {:?}", query),
        }
    } else {
        bail!("No query was supplied for extraction")
    }
}

#[derive(Clone)]
pub enum SystemRecipe {
    Composition(Box<SystemRecipe>, Box<SystemRecipe>),
    Conjunction(Box<SystemRecipe>, Box<SystemRecipe>),
    Quotient(Box<SystemRecipe>, Box<SystemRecipe>, ClockIndex),
    Component(Box<Component>),
}

impl SystemRecipe {
    pub fn compile(self, dim: ClockIndex) -> Result<TransitionSystemPtr, String> {
        match self {
            SystemRecipe::Composition(left, right) => {
                Composition::new(left.compile(dim)?, right.compile(dim)?, dim + 1)
            }
            SystemRecipe::Conjunction(left, right) => {
                Conjunction::new(left.compile(dim)?, right.compile(dim)?, dim + 1)
            }
            SystemRecipe::Quotient(left, right, clock_index) => Quotient::new(
                left.compile(dim)?,
                right.compile(dim)?,
                clock_index,
                dim + 1,
            ),
            SystemRecipe::Component(comp) => match CompiledComponent::compile(*comp, dim + 1) {
                Ok(comp) => Ok(comp),
                Err(err) => Err(err),
            },
        }
    }

    /// Gets the height of the `SystemRecipe` tree
    fn height(&self) -> usize {
        match self {
            SystemRecipe::Composition(l, r)
            | SystemRecipe::Conjunction(l, r)
            | SystemRecipe::Quotient(l, r, _) => max(l.height(), r.height()),
            SystemRecipe::Component(_) => 1,
        }
    }

    /// Gets the count `Components`s in the `SystemRecipe`
    pub fn count_component(&self) -> usize {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                left.count_component() + right.count_component()
            }
            SystemRecipe::Component(_) => 1,
        }
    }

    ///Applies the clock-reduction
    pub(crate) fn reduce_clocks(
        &mut self,
        clock_instruction: Vec<ClockReductionInstruction>,
        offset: Option<usize>,
    ) -> usize {
        let mut adjust: Option<usize> = offset;

        for (range, comp) in self
            .get_components()
            .into_iter()
            .filter_map(move |c| Some((c.get_clock_range()?, c)))
        {
            let clocks: Vec<&ClockReductionInstruction> = clock_instruction
                .iter()
                .filter(move |r| r.is_in_range(&range))
                .collect();
            let len = clocks.len();
            let offset = if let Some(u) = adjust {
                comp.decrement_dim(u, clocks.iter().flat_map(|r| r.get_indices()).collect());
                u
            } else {
                0
            };
            comp.reduce_clocks(clocks);
            adjust = Some(offset + len);
        }
        adjust.unwrap_or(0)
    }

    /// Gets all components in `SystemRecipe`
    fn get_components(&mut self) -> Vec<&mut Component> {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                let mut o = left.get_components();
                o.extend(right.get_components());
                o
            }
            SystemRecipe::Component(c) => vec![c],
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClocks {
        clock_index: ClockIndex,
        clock_indices: Vec<ClockIndex>,
    },
}

impl ClockReductionInstruction {
    ///Checks the index/indices is within a given range
    pub(crate) fn is_in_range(&self, range: &Range<usize>) -> bool {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index } => range.contains(clock_index),
            ClockReductionInstruction::ReplaceClocks { clock_indices, .. } => {
                clock_indices.iter().any(|c| range.contains(c))
            }
        }
    }

    ///Gets the index/indices
    fn get_indices(&self) -> Vec<usize> {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index } => vec![*clock_index],
            ClockReductionInstruction::ReplaceClocks { clock_indices, .. } => clock_indices.clone(),
        }
    }
}

pub fn get_system_recipe(
    side: &QueryExpression,
    component_loader: &mut dyn ComponentLoader,
    clock_index: &mut ClockIndex,
    quotient_index: &mut Option<ClockIndex>,
) -> Box<SystemRecipe> {
    match side {
        QueryExpression::Parentheses(expression) => {
            get_system_recipe(expression, component_loader, clock_index, quotient_index)
        }
        QueryExpression::Composition(left, right) => Box::new(SystemRecipe::Composition(
            get_system_recipe(left, component_loader, clock_index, quotient_index),
            get_system_recipe(right, component_loader, clock_index, quotient_index),
        )),
        QueryExpression::Conjunction(left, right) => Box::new(SystemRecipe::Conjunction(
            get_system_recipe(left, component_loader, clock_index, quotient_index),
            get_system_recipe(right, component_loader, clock_index, quotient_index),
        )),
        QueryExpression::Quotient(left, right) => {
            let left = get_system_recipe(left, component_loader, clock_index, quotient_index);
            let right = get_system_recipe(right, component_loader, clock_index, quotient_index);

            let q_index = match quotient_index {
                Some(q_i) => *q_i,
                None => {
                    *clock_index += 1;
                    debug!("Quotient clock index: {}", *clock_index);

                    quotient_index.replace(*clock_index);
                    quotient_index.unwrap()
                }
            };

            Box::new(SystemRecipe::Quotient(left, right, q_index))
        }
        QueryExpression::VarName(name) => {
            let mut component = component_loader.get_component(name).clone();
            component.set_clock_indices(clock_index);
            debug!("{} Clocks: {:?}", name, component.declarations.clocks);

            Box::new(SystemRecipe::Component(Box::new(component)))
        }
        QueryExpression::SaveAs(comp, _) => {
            get_system_recipe(comp, component_loader, clock_index, &mut None)
        }
        _ => panic!("Got unexpected query side: {:?}", side),
    }
}

fn validate_reachability_input(
    machine: &SystemRecipe,
    state: &QueryExpression,
) -> Result<(), String> {
    if let QueryExpression::State(loc_names, _) = state {
        if loc_names.len() != machine.count_component() {
            return Err(
                "The number of automata does not match the number of locations".to_string(),
            );
        }
    } else {
        return Err(format!(
            "Expected QueryExpression::State but got {:?}",
            state
        ));
    }

    Ok(())
}
