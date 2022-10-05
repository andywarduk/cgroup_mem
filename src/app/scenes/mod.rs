use std::io;

use crate::TermType;
use super::PollResult;

pub mod cgroup_tree;
pub mod help;

pub trait Scene {
    fn reload(&mut self);
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error>;
    fn poll(&mut self) -> Result<PollResult, io::Error>;
}
