mod count;
mod keyed;
mod single_value;

use std::{fmt::Display, io, num::ParseIntError, path::Path};

pub use self::{count::CountProcessor, keyed::KeyedProcessor, single_value::SingleValueProcessor};

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

    // Sanity check
    if split.is_empty() || split[0].is_empty() {
        return None;
    }

    if split.len() == 1 {
        // Format is "filename" for single value processor
        let mut proc = SingleValueProcessor::new();
        proc.set_file(split[0]);
        return Some(Box::new(proc));
    }

    match split[1] {
        "=" => {
            // Format is "filename/=/<matchcol>/<string>/<retcol>" for keyed processor
            // Columns are counted from 1
            if split.len() != 5 || split[3].is_empty() {
                return None;
            }

            let match_col = split[2].parse::<usize>();
            let ret_col = split[4].parse::<usize>();

            if match_col.is_err() || ret_col.is_err() {
                return None;
            }

            let match_col = match_col.unwrap();
            let ret_col = ret_col.unwrap();

            if match_col == 0 || ret_col == 0 {
                return None;
            }

            let mut proc = KeyedProcessor::new(match_col, split[3], ret_col);
            proc.set_file(split[0]);
            Some(Box::new(proc))
        }
        "#" => {
            // Format is "filename/#" for line count processor
            if split.len() != 2 {
                return None;
            }

            let mut proc = CountProcessor::new();
            proc.set_file(split[0]);
            Some(Box::new(proc))
        }
        _ => None,
    }
}
