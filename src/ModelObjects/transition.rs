use crate::component::{Automaton, DeclarationProvider};
use crate::DataReader::parse_edge;
use crate::EdgeEval::updater::CompiledUpdate;
use crate::ModelObjects::edge::Edge;
use crate::ModelObjects::representations::BoolExpression;
use crate::ModelObjects::state::State;
use crate::TransitionSystems::{CompositionType, LocationTree, TransitionID};
use edbm::util::constraints::ClockIndex;
use edbm::zones::OwnedFederation;
use std::collections::HashMap;
use std::fmt;

/// Represents a single transition from taking edges in multiple components
#[derive(Debug, Clone)]
pub struct Transition {
    /// The ID of the transition, based on the edges it is created from.
    pub id: TransitionID,
    pub guard_zone: OwnedFederation,
    pub target_locations: LocationTree,
    pub updates: Vec<CompiledUpdate>,
}

impl Transition {
    /// Create a new transition not based on an edge with no identifier
    pub fn new(target_locations: &LocationTree, dim: ClockIndex) -> Transition {
        Transition {
            id: TransitionID::None,
            guard_zone: OwnedFederation::universe(dim),
            target_locations: target_locations.clone(),
            updates: vec![],
        }
    }

    pub fn from(automaton: &Automaton, edge: &Edge, dim: ClockIndex) -> Transition {
        //let (comp, edge) = edges;

        let target_loc_name = &edge.target_location;
        let target_loc = automaton.get_location_by_name(target_loc_name);
        let target_locations = LocationTree::simple(target_loc, automaton.get_declarations(), dim);

        let mut compiled_updates = vec![];
        if let Some(updates) = edge.get_update() {
            compiled_updates.extend(
                updates
                    .iter()
                    .map(|update| CompiledUpdate::compile(update, automaton.get_declarations())),
            );
        }

        Transition {
            id: TransitionID::Simple(edge.id.clone()),
            guard_zone: Transition::combine_edge_guards(&vec![(automaton, edge)], dim),
            target_locations,
            updates: compiled_updates,
        }
    }

    pub fn use_transition(&self, state: &mut State) -> bool {
        let mut zone = state.take_zone();
        zone = self.apply_guards(zone);
        if !zone.is_empty() {
            zone = self.apply_updates(zone).up();
            self.move_locations(&mut state.decorated_locations);
            zone = state.decorated_locations.apply_invariants(zone);
            if !zone.is_empty() {
                state.set_zone(zone);
                return true;
            }
        }
        state.set_zone(zone);
        false
    }

    /// Returns the resulting [`State`] when using a transition in the given [`State`]
    pub fn use_transition_alt(&self, state: &State) -> Option<State> {
        let mut state = state.to_owned();
        match self.use_transition(&mut state) {
            true => Some(state),
            false => None,
        }
    }

    pub fn combinations(
        left: &Vec<Transition>,
        right: &Vec<Transition>,
        comp: CompositionType,
    ) -> Vec<Transition> {
        let mut out: Vec<Transition> = vec![];
        for l in left {
            for r in right {
                let target_locations =
                    LocationTree::compose(&l.target_locations, &r.target_locations, comp);

                let guard_zone = l.guard_zone.clone().intersection(&r.guard_zone);

                let mut updates = l.updates.clone();
                updates.append(&mut r.updates.clone());

                out.push(Transition {
                    id: match comp {
                        CompositionType::Conjunction => TransitionID::Conjunction(
                            Box::new(l.id.clone()),
                            Box::new(r.id.clone()),
                        ),
                        CompositionType::Composition => TransitionID::Composition(
                            Box::new(l.id.clone()),
                            Box::new(r.id.clone()),
                        ),
                        _ => unreachable!("Invalid composition type {:?}", comp),
                    },
                    guard_zone,
                    target_locations,
                    updates,
                });
            }
        }

        out
    }

    pub fn apply_updates(&self, mut fed: OwnedFederation) -> OwnedFederation {
        for update in &self.updates {
            fed = update.apply(fed);
        }

        fed
    }

    pub fn inverse_apply_updates(&self, mut fed: OwnedFederation) -> OwnedFederation {
        for update in &self.updates {
            fed = update.apply_as_guard(fed);
        }
        for update in &self.updates {
            fed = update.apply_as_free(fed);
        }

        fed
    }

    pub fn get_allowed_federation(&self) -> OwnedFederation {
        let mut fed = match self.target_locations.get_invariants() {
            Some(fed) => fed.clone(),
            None => OwnedFederation::universe(self.guard_zone.dim()),
        };
        fed = self.inverse_apply_updates(fed);
        self.apply_guards(fed)
    }

    pub fn apply_guards(&self, zone: OwnedFederation) -> OwnedFederation {
        zone.intersection(&self.guard_zone)
    }

    pub fn move_locations(&self, locations: &mut LocationTree) {
        *locations = self.target_locations.clone();
    }

    pub fn combine_edge_guards(
        edges: &Vec<(&Automaton, &Edge)>,
        dim: ClockIndex,
    ) -> OwnedFederation {
        let mut fed = OwnedFederation::universe(dim);
        for (comp, edge) in edges {
            fed = edge.apply_guard(comp.get_declarations(), fed);
        }
        fed
    }

    pub fn get_renamed_guard_expression(
        &self,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<BoolExpression> {
        BoolExpression::from_disjunction(&self.guard_zone.minimal_constraints(), naming)
    }

    pub fn get_renamed_updates(
        &self,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<Vec<parse_edge::Update>> {
        let updates: Vec<_> = self.updates.iter().map(|u| u.as_update(naming)).collect();

        if updates.is_empty() {
            None
        } else {
            Some(updates)
        }
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Transition{{{} to {} where {} [{}]}}",
            self.guard_zone,
            self.target_locations.id,
            self.target_locations
                .get_invariants()
                .map(|f| format!("invariant is {}", f))
                .unwrap_or_else(|| "no invariant".to_string()),
            self.updates
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }
}
