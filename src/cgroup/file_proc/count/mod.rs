use std::{
    path::PathBuf,
    fs::File,
    io::{self, BufRead}
};

use super::{StatProcessor, StatProcessorError};

pub struct CountProcessor {
    file: String,
}

impl CountProcessor {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.into(),
        }
    }
}

impl StatProcessor for CountProcessor {
    fn get_stat(&self, path: &PathBuf) -> Result<usize, StatProcessorError> {
        let mut path = path.clone();
        path.push(&self.file);

        let file = File::open(path)?;

        let buf_reader = io::BufReader::new(file);

        Ok(buf_reader.lines().count())
    }
}
