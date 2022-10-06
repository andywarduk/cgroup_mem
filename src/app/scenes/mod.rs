use std::io;

use super::PollResult;
use crate::TermType;

pub mod cgroup_tree;
pub mod stat_choose;
pub mod help;

pub trait Scene {
    fn reload(&mut self);
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error>;
    fn poll(&mut self) -> Result<PollResult, io::Error>;
}
