pub mod count;
pub mod keyed;
pub mod single_value;

use std::{
    fmt::Display,
    io,
    num::ParseIntError,
    path::Path,
};

use self::{count::CountProcessor, keyed::KeyedProcessor, single_value::SingleValueProcessor};

pub trait FileProcessor {
    fn get_value(&self, path: &Path) -> Result<String, FileProcessorError>;
}

impl dyn FileProcessor + '_ {
    pub fn get_stat(&self, path: &Path) -> Result<usize, FileProcessorError> {
        let value = self.get_value(path)?;
        Ok(value.parse::<usize>()?)
    }
}

pub enum FileProcessorError {
    IoError(io::Error),
    ValueNotFound,
    ParseError(ParseIntError),
}

impl Display for FileProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileProcessorError::IoError(e) => write!(f, "{}", e),
            FileProcessorError::ValueNotFound => write!(f, "No value found"),
            FileProcessorError::ParseError(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for FileProcessorError {
    fn from(e: io::Error) -> Self {
        FileProcessorError::IoError(e)
    }
}

impl From<ParseIntError> for FileProcessorError {
    fn from(e: ParseIntError) -> Self {
        FileProcessorError::ParseError(e)
    }
}

pub fn get_file_processor(def: &str) -> Option<Box<dyn FileProcessor>> {
    let split: Vec<&str> = def.split('/').collect();

    let result: Option<Box<dyn FileProcessor>> = if split.len() == 1 && !split[0].is_empty() {
        let mut proc = SingleValueProcessor::new();
        proc.set_file(split[0]);
        Some(Box::new(proc))
    } else if split.len() == 3 && split[1].starts_with('=') {
        let mut proc = KeyedProcessor::new(&split[1][1..], split[2]);
        proc.set_file(split[0]);
        Some(Box::new(proc))
    } else if split.len() == 2 && split[1] == "#" {
        let mut proc = CountProcessor::new();
        proc.set_file(split[0]);
        Some(Box::new(proc))
    } else {
        None
    };

    result
}
