use std::{
    path::PathBuf,
    fs::File,
    io::{self, BufRead}
};

use super::{StatProcessor, StatProcessorError};

pub struct KeyedProcessor {
    file: String,
    match_string: String,
    ret_col: usize,
}

impl KeyedProcessor {
    pub fn new(file: &str, line_start: &str, ret_col: &str) -> Self {
        Self {
            file: file.into(),
            match_string: line_start.into(),
            ret_col: ret_col.parse::<usize>().unwrap(),
        }
    }
}

impl StatProcessor for KeyedProcessor {
    fn get_stat(&self, path: &PathBuf) -> Result<usize, StatProcessorError> {
        let mut path = path.clone();
        path.push(&self.file);

        let file = File::open(path)?;

        let buf_reader = io::BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;

            let columns: Vec<&str> = line.split_whitespace().collect();

            if columns[0].starts_with(&self.match_string) {
                if self.ret_col > columns.len() {
                    return Err(StatProcessorError::ValueNotFound);
                } else {
                    return Ok(columns[self.ret_col - 1].parse::<usize>()?);
                }
            }
        }

        Err(StatProcessorError::ValueNotFound)
    }
}
