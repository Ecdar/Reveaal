use crate::to_result;
use crate::DBMLib::dbm::Federation;
use crate::EdgeEval::constraint_applyer::{apply_constraint, apply_constraints_to_state};
use crate::ModelObjects::component::{
    Component, DeclarationProvider, Declarations, Edge, Location, LocationType, SyncType,
    Transition,
};
use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::system_declarations::{SystemDeclarations, SystemSpecification};
use crate::System::save_component::combine_components;
use crate::TransitionSystems::{CompiledComponent, LocationTuple};
use crate::TransitionSystems::{TransitionSystem, TransitionSystemPtr};
use anyhow::{bail, Result};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub fn prune_system(ts: TransitionSystemPtr, dim: u32) -> Result<TransitionSystemPtr> {
    let inputs = ts.get_input_actions();
    let outputs = ts.get_output_actions();
    let comp = combine_components(&ts)?;

    if !ts.precheck_sys_rep()? {
        panic!("Trying to prune transitions system which is not least consistent");
    }

    let mut input_map: HashMap<String, Vec<String>> = HashMap::new();
    input_map.insert(comp.get_name().clone(), inputs.iter().cloned().collect());

    let sys_decl = SystemDeclarations {
        name: "".to_string(),
        declarations: SystemSpecification {
            components: vec![comp.get_name().clone()],
            input_actions: input_map,
            output_actions: HashMap::new(),
        },
    };

    Ok(prune(&comp, dim, inputs, outputs, &sys_decl)?)
}

struct PruneContext {
    comp: Component,
    inconsistent_locs: Vec<String>,
    inconsistent_parts: HashMap<String, Federation>,
    passed_pairs: Vec<(String, Federation)>,
    dim: u32,
}

impl PruneContext {
    fn decl(&self) -> &Declarations {
        self.comp.get_declarations()
    }

    fn get_loc(&self, name: &str) -> Result<&Location> {
        self.comp.get_location_by_name(name)
    }

    fn try_get_incons(&self, name: &str) -> Option<&Federation> {
        self.inconsistent_parts.get(name)
    }

    fn get_incons(&self, name: &str) -> Result<&Federation> {
        self.inconsistent_parts.get(name).ok_or_else(|| {
            anyhow::format_err!(
                "Pruned component doesnt have incosistent location named {}",
                name
            )
        })
    }

    fn remove_edge(&mut self, edge: &Edge) {
        if let Some(index) = self.comp.edges.iter().position(|e| *e == *edge) {
            println!("Removing {}", edge);
            self.comp.edges.remove(index);
            self.comp.create_edge_io_split();
        }
    }

    fn update_edge_guard(&mut self, edge: &Edge, guard_fed: &Federation) -> Result<()> {
        if let Some(index) = self.comp.edges.iter().position(|e| *e == *edge) {
            let guard = guard_fed.as_boolexpression(Some(&self.decl().clocks));

            println!(
                "Updating {} with guard {}",
                edge,
                guard.as_ref().unwrap_or(&BoolExpression::Bool(true))
            );
            self.comp.edges.get_mut(index).unwrap().guard = guard;
            self.comp.create_edge_io_split();
        }
        Ok(())
    }

    fn finish(self) -> (Component, HashMap<String, Federation>) {
        (self.comp, self.inconsistent_parts)
    }
}

pub fn prune(
    comp: &Component,
    dim: u32,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    decl: &SystemDeclarations,
) -> Result<Box<CompiledComponent>> {
    let mut new_comp = comp.clone();
    new_comp.create_edge_io_split();
    let inconsistent_locs: Vec<_> = new_comp
        .locations
        .iter()
        .filter(|l| is_immediately_inconsistent(l, comp, dim))
        .map(|l| l.id.clone())
        .collect();
    let inconsistent_parts: HashMap<String, Federation> = inconsistent_locs
        .iter()
        .map(|id| (id.clone(), Federation::full(dim)))
        .collect();

    println!("Inconsistent locs: {:?}", inconsistent_locs);

    let mut context = PruneContext {
        comp: new_comp,
        inconsistent_locs,
        inconsistent_parts,
        passed_pairs: vec![],
        dim,
    };

    while let Some(target_loc) = context.inconsistent_locs.pop() {
        // TODO: If is initial

        //Handle edges
        for edge in comp
            .edges
            .iter()
            .filter(|e| e.target_location == target_loc)
            .cloned()
        {
            if *edge.get_sync_type() == SyncType::Input {
                handle_input(&edge, &mut context)?;
            } else
            // If output
            {
                handle_output(&edge, &mut context)?;
            }
        }

        println!(
            "Step {}",
            context
                .inconsistent_parts
                .iter()
                .map(|(loc, fed)| format!("{loc}: {fed}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let (mut new_comp, incons_parts) = context.finish();
    add_inconsistent_parts_to_invariants(&mut new_comp, incons_parts, dim)?;

    println!(
        "Pruned component from {} edges to {} edges",
        comp.get_edges().len(),
        new_comp.get_edges().len()
    );

    CompiledComponent::compile_with_actions(new_comp, inputs, outputs, dim)
}

fn add_inconsistent_parts_to_invariants(
    comp: &mut Component,
    incons_parts: HashMap<String, Federation>,
    dim: u32,
) -> Result<()> {
    let decls = comp.get_declarations().clone();
    for location in &mut comp.locations {
        if let Some(incons) = incons_parts.get(&location.id) {
            // get invariant
            let mut invariant_fed = Federation::full(dim);
            if let Some(inv) = location.get_invariant() {
                apply_constraints_to_state(inv, &decls, &mut invariant_fed)?;
            }
            // Remove inconsistent part
            invariant_fed.subtract(incons);
            // Set the new invariant
            location.invariant = invariant_fed.as_boolexpression(Some(&decls.clocks));
        }
    }

    Ok(())
}

fn handle_input(edge: &Edge, context: &mut PruneContext) -> Result<()> {
    let target_loc = context.get_loc(edge.get_target_location())?.clone();
    let mut inconsistent_part = context.get_incons(target_loc.get_id())?.clone();

    // apply target invariant
    if let Some(inv) = target_loc.get_invariant() {
        apply_constraints_to_state(inv, context.decl(), &mut inconsistent_part)?;
    }
    // apply updates as guard
    if let Some(updates) = edge.get_update() {
        for update in updates {
            update
                .compiled(context.decl())?
                .apply_as_guard(&mut inconsistent_part);
        }
    }

    if inconsistent_part.is_empty() {
        return Ok(());
    }

    let inconsistent_part = back_exploration_on_transition(edge, inconsistent_part, context)?;

    if !inconsistent_part.is_empty() {
        // in the next step, we need to check whether there is output transitions that could lead us away from the inconsistent state
        // such a transition needs to
        // a) have the same source as e
        // b) not be a selfloop
        // c) be an output
        // d) not lead to the inconsistent part of a state itself
        let copy = inconsistent_part.clone();
        let mut inconsistent_part = predt_of_all_outputs(&target_loc, inconsistent_part, context)?;

        if copy == inconsistent_part {
            inconsistent_part.down();
            // apply source invariant

            let source_loc = context.get_loc(edge.get_source_location())?;
            if let Some(inv) = source_loc.get_invariant() {
                apply_constraints_to_state(inv, context.decl(), &mut inconsistent_part)?;
            }
        }

        process_source_location(edge.get_source_location(), &mut inconsistent_part, context);
    }

    remove_transition_if_unsat(edge, context)
}

fn remove_transition_if_unsat(edge: &Edge, context: &mut PruneContext) -> Result<()> {
    let mut edge_fed = Federation::full(context.dim);
    // apply target invariant
    let target_loc = context.get_loc(edge.get_target_location())?;
    if let Some(inv) = target_loc.get_invariant() {
        apply_constraints_to_state(inv, context.decl(), &mut edge_fed)?;
    }

    // Subtract target inconsistent part
    if let Some(incons) = context.try_get_incons(edge.get_target_location()) {
        edge_fed.subtract(incons);
    }

    if let Some(updates) = edge.get_update() {
        // apply updates as guard
        for update in updates {
            update
                .compiled(context.decl())?
                .apply_as_guard(&mut edge_fed);
        }

        // apply updates as free
        if !edge_fed.is_empty() {
            for update in updates {
                update
                    .compiled(context.decl())?
                    .apply_as_free(&mut edge_fed);
            }
        }
    }

    // Apply guards
    if let Some(guard) = edge.get_guard() {
        apply_constraints_to_state(guard, context.decl(), &mut edge_fed)?;
    }

    // Apply source invariant
    let source_loc = context.get_loc(edge.get_source_location())?;
    if let Some(inv) = source_loc.get_invariant() {
        apply_constraints_to_state(inv, context.decl(), &mut edge_fed)?;
    }

    // Subtract source inconsistent part
    if let Some(incons) = context.try_get_incons(edge.get_source_location()) {
        edge_fed.subtract(incons);
    }

    if edge_fed.is_empty() {
        context.remove_edge(edge);
    }

    Ok(())
}

fn process_source_location(
    source_loc: &String,
    inconsistent_part: &Federation,
    context: &mut PruneContext,
) {
    if !inconsistent_part.is_empty() {
        match context.inconsistent_parts.get_mut(source_loc) {
            Some(part) => part.add_fed(&inconsistent_part),
            None => {
                context
                    .inconsistent_parts
                    .insert(source_loc.clone(), inconsistent_part.clone());
            }
        };

        if !context
            .passed_pairs
            .iter()
            .any(|(id, fed)| *id == *source_loc && *fed == *inconsistent_part)
        {
            context
                .passed_pairs
                .push((source_loc.clone(), inconsistent_part.clone()));
            context.inconsistent_locs.push(source_loc.clone());
        }
    }
}

fn predt_of_all_outputs(
    source_loc: &Location,
    inconsistent_part: Federation,
    context: &mut PruneContext,
) -> Result<Federation> {
    let mut incons_fed = inconsistent_part;
    for other_edge in context
        .comp
        .get_edges()
        .iter()
        .filter(|e| e.source_location == source_loc.id && e.sync_type == SyncType::Output)
    {
        let target_loc = context.get_loc(other_edge.get_target_location())?;

        let mut saving_fed = Federation::full(context.dim);
        // apply target invariant
        if let Some(inv) = target_loc.get_invariant() {
            apply_constraints_to_state(inv, context.decl(), &mut saving_fed)?;
        }

        // remove the parts of the target transition that are inconsistent.
        if context
            .inconsistent_locs
            .iter()
            .any(|id| *id == target_loc.id)
        {
            saving_fed.subtract(context.get_incons(other_edge.get_target_location())?)
        }

        // apply updates via free
        if !saving_fed.is_empty() {
            if let Some(updates) = other_edge.get_update() {
                for update in updates {
                    update
                        .compiled(context.decl())?
                        .apply_as_free(&mut saving_fed);
                }
            }
        }

        // apply edge guard
        if let Some(guard) = other_edge.get_guard() {
            apply_constraints_to_state(guard, context.decl(), &mut saving_fed)?;
        }

        // apply source invariant
        if let Some(inv) = source_loc.get_invariant() {
            apply_constraints_to_state(inv, context.decl(), &mut saving_fed)?;
        }

        // do temporal predecessor avoiding saving fed
        let mut predt_fed = incons_fed.clone();
        predt_fed.predt(&saving_fed);
        incons_fed += predt_fed;
    }

    Ok(incons_fed)
}

fn back_exploration_on_transition(
    edge: &Edge,
    mut inconsistent_part: Federation,
    context: &mut PruneContext,
) -> Result<Federation> {
    // apply updates via free
    if let Some(updates) = edge.get_update() {
        for update in updates {
            update
                .compiled(context.decl())?
                .apply_as_free(&mut inconsistent_part);
        }
    }

    // apply edge guard
    if let Some(guard) = edge.get_guard() {
        apply_constraints_to_state(guard, context.decl(), &mut inconsistent_part)?;
    }

    // apply source invariant
    let source = context.get_loc(edge.get_source_location())?;
    if let Some(inv) = source.get_invariant() {
        apply_constraints_to_state(inv, context.decl(), &mut inconsistent_part)?;
    }

    Ok(inconsistent_part)
}

fn handle_output(edge: &Edge, context: &mut PruneContext) -> Result<()> {
    let target_incons = context.get_incons(edge.get_target_location())?;
    if target_incons.is_full() {
        // Fully inconsistent target
        context.remove_edge(edge);
    } else {
        // Partially inconsistent target
        let mut incons_after_reset = target_incons.clone();
        if let Some(updates) = edge.get_update() {
            // TODO: this is different from J-ecdar
            // apply updates as guard
            for update in updates {
                update
                    .compiled(context.decl())?
                    .apply_as_guard(&mut incons_after_reset);
            }

            // apply updates as free
            if !incons_after_reset.is_empty() {
                for update in updates {
                    update
                        .compiled(context.decl())?
                        .apply_as_free(&mut incons_after_reset);
                }
            }
        }
        let mut guard_fed = Federation::full(context.dim);
        // Apply guards
        if let Some(guard) = edge.get_guard() {
            apply_constraints_to_state(guard, context.decl(), &mut guard_fed)?;
        }
        guard_fed.subtract(&incons_after_reset);

        if guard_fed.is_empty() {
            context.remove_edge(edge);
        } else {
            context.update_edge_guard(edge, &guard_fed)?;
        }
    }

    // get source invariant
    let mut source_invariant = Federation::full(context.dim);
    let source_loc = context.get_loc(edge.get_source_location())?;
    if let Some(inv) = source_loc.get_invariant() {
        apply_constraints_to_state(inv, context.decl(), &mut source_invariant)?;
    }

    if source_invariant.can_delay_indefinitely() {
        // Source is not inconsistent, nothing more to do
    } else {
        let mut fed_that_saves_us = Federation::empty(context.dim);
        for other_edge in context
            .comp
            .edges
            .iter()
            .filter(|e| {
                e.source_location == edge.source_location && *e.get_sync_type() == SyncType::Output
            })
            .cloned()
        {
            // calculate and backtrack the part that is NOT inconsistent

            // get target invariant
            let mut good_part = Federation::full(context.dim);
            let target_loc = context.get_loc(other_edge.get_target_location())?;
            if let Some(inv) = target_loc.get_invariant() {
                apply_constraints_to_state(inv, context.decl(), &mut good_part)?;
            }

            // If target is inconsistent we must avoid that part
            if let Some(incons) = context.try_get_incons(other_edge.get_target_location()) {
                good_part.subtract(incons);
            }

            if let Some(updates) = other_edge.get_update() {
                // TODO: this is different from J-ecdar
                // apply updates as guard
                for update in updates {
                    update
                        .compiled(context.decl())?
                        .apply_as_guard(&mut good_part);
                }

                // apply updates as free
                if !good_part.is_empty() {
                    for update in updates {
                        update
                            .compiled(context.decl())?
                            .apply_as_free(&mut good_part);
                    }
                }
            }

            // Apply guards
            if let Some(guard) = other_edge.get_guard() {
                apply_constraints_to_state(guard, context.decl(), &mut good_part)?;
            }

            good_part.down(); // We are allowed to delay into outputs
            good_part.intersect(&source_invariant);

            fed_that_saves_us += good_part;
        }

        let new_incon_part = source_invariant - fed_that_saves_us;
        process_source_location(edge.get_source_location(), &new_incon_part, context);
    }
    Ok(())
}

fn is_immediately_inconsistent(location: &Location, comp: &Component, dimensions: u32) -> bool {
    //MUST: check if this works, if so refactor

    // let loc = LocationTuple::simple(location, &comp.declarations, dimensions)?;
    // return Ok(loc.is_inconsistent());

    location.location_type == LocationType::Inconsistent
}
