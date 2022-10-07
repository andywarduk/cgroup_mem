use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use super::{FileProcessor, FileProcessorError};

#[derive(Default)]
pub struct CountProcessor {
    file: Option<String>,
}

impl CountProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.to_string())
    }
}

impl FileProcessor for CountProcessor {
    fn get_value(&self, path: &PathBuf) -> Result<String, FileProcessorError> {
        let mut path = path.clone();

        if let Some(file) = &self.file {
            path.push(file);
        }

        let file = File::open(path)?;

        let buf_reader = io::BufReader::new(file);

        Ok(buf_reader.lines().count().to_string())
    }
}
