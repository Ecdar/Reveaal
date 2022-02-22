use crate::DBMLib::dbm::{Federation, Zone};
use crate::DataReader::parse_edge;
use crate::EdgeEval::updater::updater;
use crate::ModelObjects::component::Declarations;
use crate::ModelObjects::component::{
    Component, DeclarationProvider, DecoratedLocation, Location, LocationType, State, SyncType,
    Transition,
};
use crate::ModelObjects::max_bounds::MaxBounds;
use crate::System::local_consistency;
use crate::TransitionSystems::{LocationTuple, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Quotient {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    universal_location: Location,
    inconsistent_location: Location,
    decls: Declarations,
}

static INCONSISTENT_LOC_NAME: &str = "Inconsistent";
static UNIVERSAL_LOC_NAME: &str = "Universal";
impl Quotient {
    pub fn new(left: TransitionSystemPtr, right: TransitionSystemPtr) -> Box<Quotient> {
        let mut locations = HashMap::new();
        for loc_left in left.get_all_locations() {
            for loc_right in right.get_all_locations() {
                let location_type = if loc_left.is_initial() && loc_right.is_initial() {
                    LocationType::Initial
                } else {
                    LocationType::Normal
                };

                let id = loc_left.to_string().clone() + &loc_right.to_string();

                locations.insert(
                    (loc_left.to_string(), loc_right.to_string()),
                    Location {
                        id,
                        invariant: None,
                        location_type,
                        urgency: "".to_string(),
                    },
                );
            }
        }
        let universal_location = Location {
            id: UNIVERSAL_LOC_NAME.to_string(),
            invariant: None,
            location_type: LocationType::Universal,
            urgency: "".to_string(),
        };

        let inconsistent_location = Location {
            id: INCONSISTENT_LOC_NAME.to_string(),
            invariant: None,
            location_type: LocationType::Inconsistent,
            urgency: "".to_string(),
        };

        let inputs: HashSet<String> = left
            .get_input_actions()
            .union(&right.get_output_actions())
            .cloned()
            .collect();

        let output_dif: HashSet<String> = left
            .get_output_actions()
            .difference(&right.get_output_actions())
            .cloned()
            .collect();
        let input_dif: HashSet<String> = right
            .get_input_actions()
            .difference(&left.get_input_actions())
            .cloned()
            .collect();

        let outputs: HashSet<String> = output_dif.union(&input_dif).cloned().collect();

        Box::new(Quotient {
            left,
            right,
            inputs,
            outputs,
            universal_location,
            inconsistent_location,
            decls: Declarations::empty(),
        })
    }
}

impl TransitionSystem<'static> for Quotient {
    default_composition!();
    fn next_transitions<'b>(
        &'b self,
        location: &LocationTuple<'b>,
        action: &str,
        sync_type: &SyncType,
        index: &mut usize,
        dim: u32,
    ) -> Vec<Transition<'b>> {
        let mut transitions = vec![];
        let left = self
            .left
            .next_transitions(location, action, sync_type, index, dim);
        let right_index = *index;
        let right = self
            .right
            .next_transitions(location, action, sync_type, index, dim);
        let quotient_index = *index;
        *index += 1;

        println!("Index: {}", quotient_index);

        //Rules [universal] and [incosistent]
        if let Some(quotient_loc) = location.try_get_location(quotient_index) {
            let quotient_state = quotient_loc.get_location_type();
            if *quotient_state == LocationType::Inconsistent {
                transitions.push(Transition::new(dim));
                return transitions;
            } else if *quotient_state == LocationType::Universal {
                if self.inputs.contains(action) {
                    transitions.push(Transition::new(dim));
                }
                return transitions;
            }
        }

        //Reused target locations
        let mut universal_location = HashMap::new();
        universal_location.insert(quotient_index, &self.universal_location);
        let mut inconsistent_location = HashMap::new();
        inconsistent_location.insert(quotient_index, &self.inconsistent_location);

        //Rule 1
        if self.right.get_actions().contains(action) && self.left.get_actions().contains(action) {
            transitions.append(&mut Transition::combinations(&left, &right));
        }

        //Rule 2
        if self.right.get_actions().contains(action) && !self.left.get_actions().contains(action) {
            transitions.append(&mut right.clone());
        }

        if self.right.get_output_actions().contains(action) {
            //Rule 3
            let fed = Federation::new(right.iter().map(|t| &t.guard_zone).cloned().collect(), dim)
                .inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));

            //Rule 4
            let mut zones = vec![];
            for transition in &right {
                let mut zone = Zone::init(dim);
                for (&index, location) in &transition.target_locations {
                    let dec_loc = DecoratedLocation::create(
                        location,
                        self.right.get_components()[index].get_declarations(),
                    );
                    dec_loc.apply_invariant(&mut zone);
                }
                for (&index, updates) in &transition.updates {
                    //Assumes all updates are free operations i.e [x -> 0]
                    updater(
                        updates,
                        self.right.get_components()[index].get_declarations(),
                        &mut zone,
                    );
                }
                zones.push(zone);
            }
            let fed = Federation::new(zones, dim).inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));
        }

        //Rule 5
        if self.inputs.contains(action) || self.outputs.contains(action) {
            let mut zone = Zone::init(dim);
            let mut tmp_index = right_index;
            for _ in self.right.get_children() {
                let dec_loc = DecoratedLocation::create(
                    location.get_location(tmp_index),
                    location.get_decl(tmp_index),
                );
                dec_loc.apply_invariant(&mut zone);

                tmp_index += 1;
            }
            let fed = Federation::new(vec![zone], dim).inverse(dim);
            transitions.append(&mut create_transitions(
                fed,
                &universal_location,
                &HashMap::new(),
            ));
        }

        transitions
    }

    fn is_locally_consistent(&self, dimensions: u32) -> bool {
        local_consistency::is_least_consistent(self, dimensions)
    }

    fn get_all_locations<'b>(&'b self) -> Vec<LocationTuple<'b>> {
        let mut location_tuples = vec![];
        let left = self.left.get_all_locations();
        let right = self.right.get_all_locations();
        for loc1 in left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(loc1.clone(), loc2.clone()));
            }
        }
        let inconsistent = LocationTuple::simple(&self.inconsistent_location, &self.decls);
        let universal = LocationTuple::simple(&self.universal_location, &self.decls);

        location_tuples.push(inconsistent);
        location_tuples.push(universal);

        location_tuples
    }

    fn get_mut_children(&mut self) -> Vec<&mut TransitionSystemPtr> {
        vec![&mut self.left, &mut self.right]
    }

    fn get_children(&self) -> Vec<&TransitionSystemPtr> {
        vec![&self.left, &self.right]
    }
}

fn create_transitions<'a>(
    fed: Federation,
    target_locations: &HashMap<usize, &'a Location>,
    updates: &HashMap<usize, Vec<parse_edge::Update>>,
) -> Vec<Transition<'a>> {
    let mut transitions = vec![];
    for zone in fed.zones {
        transitions.push(Transition {
            guard_zone: zone,
            target_locations: target_locations.clone(),
            updates: updates.clone(),
        });
    }
    transitions
}
