use crate::to_result;
use anyhow::Result;
use pest::iterators::{Pair, Pairs};
use pest::RuleType;

pub trait TryNextable<'i, R> {
    fn try_next(&mut self) -> Result<Pair<'i, R>>;
}

impl<'i, R: RuleType> TryNextable<'i, R> for Pairs<'i, R> {
    fn try_next(&mut self) -> Result<Pair<'i, R>> {
        to_result!(self.next())
    }
}
