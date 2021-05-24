use crate::DBMLib::dbm::Zone;
use crate::ModelObjects::component::State;

#[derive(Clone)]
pub struct StatePair<'a> {
    pub states1: Vec<State<'a>>,
    pub states2: Vec<State<'a>>,
    pub zone: Zone,
}

impl<'b> StatePair<'b> {
    pub fn create<'a>(states1: Vec<State<'a>>, states2: Vec<State<'a>>) -> StatePair<'a> {
        let mut dimensions = 1;
        for state in &states1 {
            dimensions += state.get_dimensions();
        }

        for state in &states2 {
            dimensions += state.get_dimensions();
        }

        let zone = Zone::init(dimensions);

        StatePair {
            states1,
            states2,
            zone,
        }
    }

    pub fn get_states1(&self) -> &Vec<State<'b>> {
        &self.states1
    }

    pub fn get_states2(&self) -> &Vec<State<'b>> {
        &self.states2
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_states(
        &mut self,
        is_states1: bool,
    ) -> (&mut Vec<State<'b>>, &mut Vec<State<'b>>) {
        if is_states1 {
            (&mut self.states1, &mut self.states2)
        } else {
            (&mut self.states2, &mut self.states1)
        }
    }
}
