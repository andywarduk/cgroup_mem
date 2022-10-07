use std::io;

use super::PollResult;

use crate::TermType;

pub mod cgroup_tree;
pub mod cgroup_tree_help;
pub mod procs;
pub mod procs_help;
pub mod stat_choose;

pub trait Scene {
    fn reload(&mut self);
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error>;
    fn poll(&mut self) -> Result<PollResult, io::Error>;
}
