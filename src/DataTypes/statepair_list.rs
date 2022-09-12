use std::collections::{HashMap, VecDeque};

use edbm::zones::OwnedFederation;

use crate::{ModelObjects::statepair::StatePair, TransitionSystems::LocationID};

pub type PassedStateList = PassedStateListFed;
type PassedStateListFed = HashMap<(LocationID, LocationID), OwnedFederation>;
type PassedStateListVec = HashMap<(LocationID, LocationID), Vec<OwnedFederation>>;

pub type WaitingStateList = DepthFirstWaitingStateList;
pub struct DepthFirstWaitingStateList {
    queue: VecDeque<StatePair>,
    map: HashMap<(LocationID, LocationID), VecDeque<OwnedFederation>>,
}

pub trait PassedStateListExt {
    fn put(&mut self, pair: StatePair);
    fn has(&self, pair: &StatePair) -> bool;
    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<&OwnedFederation>;
}

impl PassedStateListExt for PassedStateListVec {
    fn put(&mut self, mut pair: StatePair) {
        let fed = pair.take_zone();
        let (loc1, loc2) = (pair.locations1.id, pair.locations2.id);
        let key = (loc1, loc2);
        if let Some(vec) = self.get_mut(&key) {
            vec.push(fed);
        } else {
            self.insert(key, vec![fed]);
        };
    }

    fn has(&self, pair: &StatePair) -> bool {
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            pair.ref_zone(),
        );
        let key = (loc1, loc2);
        match self.get(&key) {
            Some(vec) => vec.iter().any(|f| fed.subset_eq(f)),
            None => false,
        }
    }

    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<&OwnedFederation> {
        match self.get(key) {
            Some(vec) => vec.iter().collect(),
            None => panic!("No zones for key: {:?}", key),
        }
    }
}

impl PassedStateListExt for DepthFirstWaitingStateList {
    fn put(&mut self, mut pair: StatePair) {
        self.queue.push_front(pair.clone());
        let fed = pair.take_zone();
        let (loc1, loc2) = (pair.locations1.id, pair.locations2.id);
        let key = (loc1, loc2);
        if let Some(vec) = self.map.get_mut(&key) {
            vec.push_front(fed);
        } else {
            self.map.insert(key, vec![fed].into());
        };
    }
    fn has(&self, pair: &StatePair) -> bool {
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            pair.ref_zone(),
        );
        let key = (loc1, loc2);
        match self.map.get(&key) {
            Some(vec) => vec.iter().any(|f| fed.subset_eq(f)),
            None => false,
        }
    }
    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<&OwnedFederation> {
        match self.map.get(key) {
            Some(vec) => vec.iter().collect(),
            None => panic!("No zones for key: {:?}", key),
        }
    }
}

#[allow(clippy::new_without_default)]
impl DepthFirstWaitingStateList {
    pub fn new() -> Self {
        DepthFirstWaitingStateList {
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
}
impl PassedStateListExt for PassedStateListFed {
    fn put(&mut self, mut pair: StatePair) {
        let mut fed = pair.take_zone();
        let (loc1, loc2) = (pair.locations1.id, pair.locations2.id);
        let key = (loc1, loc2);

        if let Some(f) = self.get(&key) {
            fed = fed.union(f).expensive_reduce();
        }
        self.insert(key, fed);
    }

    fn has(&self, pair: &StatePair) -> bool {
        let (loc1, loc2, fed) = (
            pair.locations1.id.clone(),
            pair.locations2.id.clone(),
            pair.ref_zone(),
        );
        let key = (loc1, loc2);
        match self.get(&key) {
            Some(f) => fed.subset_eq(f),
            None => false,
        }
    }

    fn zones(&self, key: &(LocationID, LocationID)) -> Vec<&OwnedFederation> {
        match self.get(key) {
            Some(f) => vec![f],
            None => panic!("No zones for key: {:?}", key),
        }
    }
}
