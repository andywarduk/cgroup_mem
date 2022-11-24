use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use super::{FileProcessor, FileProcessorError};

#[derive(Default)]
pub struct SingleValueProcessor {
    file: Option<String>,
}

impl SingleValueProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.to_string())
    }
}

impl FileProcessor for SingleValueProcessor {
    fn get_value(&self, path: &Path) -> Result<String, FileProcessorError> {
        let mut path = path.to_path_buf();

        if let Some(file) = &self.file {
            path.push(file);
        }

        let file = File::open(path)?;

        match io::BufReader::new(file).lines().next() {
            None => Err(FileProcessorError::ValueNotFound)?,
            Some(Err(e)) => Err(e)?,
            Some(Ok(line)) => Ok(line),
        }
    }
}
