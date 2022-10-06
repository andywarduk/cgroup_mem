use std::{
    path::PathBuf,
    fs::File,
    io::{self, BufRead}
};

use super::{StatProcessor, StatProcessorError};

pub struct SingleValueProcessor {
    file: String,
}

impl SingleValueProcessor {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.into(),
        }
    }
}

impl StatProcessor for SingleValueProcessor {
    fn get_stat(&self, path: &PathBuf) -> Result<usize, StatProcessorError> {
        let mut path = path.clone();
        path.push(&self.file);

        let file = File::open(path)?;

        match io::BufReader::new(file)
            .lines()
            .next()
        {
            None => Err(StatProcessorError::ValueNotFound)?,
            Some(Err(e)) => Err(e)?,
            Some(Ok(line)) => Ok(line.parse::<usize>()?),
        }    
    }
}
