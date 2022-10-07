use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use super::{FileProcessor, FileProcessorError};

#[derive(Default)]
pub struct KeyedProcessor {
    file: Option<String>,
    match_string: String,
    ret_col: usize,
}

impl KeyedProcessor {
    pub fn new(line_start: &str, ret_col: &str) -> Self {
        Self {
            file: None,
            match_string: line_start.into(),
            ret_col: ret_col.parse::<usize>().unwrap(),
        }
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.to_string())
    }
}

impl FileProcessor for KeyedProcessor {
    fn get_value(&self, path: &PathBuf) -> Result<String, FileProcessorError> {
        let mut path = path.clone();

        if let Some(file) = &self.file {
            path.push(file);
        }

        let file = File::open(path)?;

        let buf_reader = io::BufReader::new(file);

        for line in buf_reader.lines() {
            let line = line?;

            let columns: Vec<&str> = line.split_whitespace().collect();

            if columns[0].starts_with(&self.match_string) {
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
