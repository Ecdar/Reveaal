use std::collections::{HashMap, VecDeque};

use crate::{
    DBMLib::dbm::Federation, ModelObjects::statepair::StatePair,
    TransitionSystems::transition_system::LocationID,
};

pub type PassedStateList = PassedStateListFed;
type PassedStateListFed = HashMap<(LocationID, LocationID), Federation>;
type PassedStateListVec = HashMap<(LocationID, LocationID), Vec<Federation>>;
pub struct WaitingStateList {
    queue: VecDeque<StatePair>,
    map: HashMap<(LocationID, LocationID), VecDeque<Federation>>,
}

const BREADTH_FIRST: bool = false;

pub trait PassedStateListExt {
    fn put(&mut self, pair: StatePair);
    fn has(&self, pair: &StatePair) -> bool;
    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<Federation>;
}

impl PassedStateListExt for PassedStateListVec {
    fn put(&mut self, pair: StatePair) {
        let (loc1, loc2, fed) = (pair.locations1.id, pair.locations2.id, pair.zone);
        let key = (loc1, loc2);
        if let Some(vec) = self.get_mut(&key) {
            vec.push(fed);
        } else {
            self.insert(key, vec![fed]);
        };
    }

    fn has(&self, pair: &StatePair) -> bool {
        //let pair = pair.clone();
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            &pair.zone,
        );
        let key = (loc1, loc2);
        match self.get(&key) {
            Some(vec) => vec.iter().any(|f| fed.is_subset_eq(f)),
            None => false,
        }
    }

    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<Federation> {
        match self.get(key) {
            Some(vec) => vec.clone(),
            None => vec![],
        }
    }
}

impl PassedStateListExt for WaitingStateList {
    fn put(&mut self, pair: StatePair) {
        if BREADTH_FIRST {
            self.queue.push_back(pair.clone());
        } else {
            self.queue.push_front(pair.clone());
        }
        let (loc1, loc2, fed) = (pair.locations1.id, pair.locations2.id, pair.zone);
        let key = (loc1, loc2);
        if let Some(vec) = self.map.get_mut(&key) {
            if BREADTH_FIRST {
                vec.push_back(fed);
            } else {
                vec.push_front(fed);
            }
        } else {
            self.map.insert(key, vec![fed].into());
        };
    }
    fn has(&self, pair: &StatePair) -> bool {
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            &pair.zone,
        );
        let key = (loc1, loc2);
        match self.map.get(&key) {
            Some(vec) => vec.iter().any(|f| fed.is_subset_eq(f)),
            None => false,
        }
    }
    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<Federation> {
        match self.map.get(key) {
            Some(vec) => vec.clone().into(),
            None => vec![],
        }
    }
}

impl WaitingStateList {
    pub fn new() -> Self {
        WaitingStateList {
            queue: VecDeque::new(),
            map: HashMap::new(),
        }
    }

    pub fn pop(&mut self) -> Option<StatePair> {
        let pair = self.queue.pop_front()?;
        let key = (pair.locations1.id.clone(), pair.locations2.id.clone());

        if let Some(vec) = self.map.get_mut(&key) {
            vec.pop_front().unwrap();
        };

        Some(pair)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
impl PassedStateListExt for PassedStateListFed {
    fn put(&mut self, pair: StatePair) {
        let (loc1, loc2, fed) = (pair.locations1.id, pair.locations2.id, pair.zone);
        let key = (loc1, loc2);
        if let Some(f) = self.get_mut(&key) {
            f.add_fed(&fed);
            f.reduce(true);
        } else {
            self.insert(key, fed);
        };
    }

    fn has(&self, pair: &StatePair) -> bool {
        //let pair = pair.clone();
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            &pair.zone,
        );
        let key = (loc1, loc2);
        match self.get(&key) {
            Some(f) => fed.is_subset_eq(f),
            None => false,
        }
    }

    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<Federation> {
        match self.get(key) {
            Some(f) => vec![f.clone()],
            None => vec![],
        }
    }
}
