use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use super::{FileProcessor, FileProcessorError};

#[derive(Default)]
pub struct KeyedProcessor {
    file: Option<String>,
    match_col: usize,
    match_val: String,
    ret_col: usize,
}

impl KeyedProcessor {
    pub fn new(match_col: usize, match_val: &str, ret_col: usize) -> Self {
        Self {
            file: None,
            match_col,
            match_val: match_val.into(),
            ret_col,
        }
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.to_string())
    }
}

impl FileProcessor for KeyedProcessor {
    fn get_value(&self, path: &Path) -> Result<String, FileProcessorError> {
        let mut path = path.to_path_buf();

        if let Some(file) = &self.file {
            path.push(file);
        }

        let file = File::open(path)?;

        let buf_reader = io::BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;

            let columns: Vec<&str> = line.split_whitespace().collect();

            if self.match_col < columns.len() && columns[self.match_col - 1] == self.match_val {
                if self.ret_col > columns.len() {
                    return Err(FileProcessorError::ValueNotFound);
                } else {
                    return Ok(columns[self.ret_col - 1].to_string());
                }
            }
        }

        Err(FileProcessorError::ValueNotFound)
    }
}
