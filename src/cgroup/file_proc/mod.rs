mod single_value;
mod keyed;
mod count;

use std::{
    path::PathBuf,
    io,
    num::ParseIntError,
    fmt::Display
};

use self::{single_value::SingleValueProcessor, keyed::KeyedProcessor, count::CountProcessor};

use super::stats::STATS;

pub trait StatProcessor {
    fn get_stat(&self, path: &PathBuf) -> Result<usize, StatProcessorError>;
}

pub enum StatProcessorError {
    IoError(io::Error),
    ValueNotFound,
    ParseError(ParseIntError),
}

impl Display for StatProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatProcessorError::IoError(e) => write!(f, "{}", e),
            StatProcessorError::ValueNotFound => write!(f, "No value found"),
            StatProcessorError::ParseError(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for StatProcessorError {
    fn from(e: io::Error) -> Self {
        StatProcessorError::IoError(e)
    }
}

impl From<ParseIntError> for StatProcessorError {
    fn from(e: ParseIntError) -> Self {
        StatProcessorError::ParseError(e)
    }
}

pub fn get_stat_processor(stat: usize) -> Box<dyn StatProcessor> {
    let split: Vec<&str> = STATS[stat].def().split('/').collect();

    let result: Box<dyn StatProcessor> = if split.len() == 1 {
        Box::new(SingleValueProcessor::new(split[0]))
    } else if split.len() == 3 && split[1].starts_with('=') {
        Box::new(KeyedProcessor::new(split[0], &split[1][1..], split[2]))
    } else if split.len() == 2 && split[1] == "#" {
        Box::new(CountProcessor::new(split[0]))
    } else {
        panic!("Unrecognised stat processor {}", stat);
    };

    result
}
