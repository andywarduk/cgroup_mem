pub mod stats;

use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf, num::ParseIntError, fmt::Display,
};

use self::stats::{STATS, StatType};

#[derive(Debug, Clone)]
pub struct CGroup {
    path: PathBuf,
    error: Option<String>,
    stat: usize,
    children: Vec<CGroup>,
}

impl CGroup {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            error: None,
            stat: 0,
            children: Vec::new(),
        }
    }

    fn new_error(path: PathBuf, msg: String) -> Self {
        Self {
            path,
            error: Some(msg),
            stat: 0,
            children: Vec::new(),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn stat(&self) -> usize {
        self.stat
    }

    pub fn children(&self) -> &Vec<CGroup> {
        &self.children
    }

    pub fn error(&self) -> &Option<String> {
        &self.error
    }
}

#[derive(Clone, Copy)]
pub enum SortOrder {
    NameAsc,
    NameDsc,
    SizeAsc,
    SizeDsc,
}

pub fn load_cgroups(stat: usize, sort: SortOrder) -> Vec<CGroup> {
    let mut path_buf = PathBuf::new();
    path_buf.push("/sys/fs/cgroup");

    let root = PathBuf::new();

    let processor = get_stat_processor(stat);

    match load_cgroup_rec(path_buf, &root, sort, stat, &*processor) {
        Ok(cgroup) => {
            if cgroup.stat == 0 {
                cgroup.children
            } else {
                vec![cgroup]
            }
        },
        Err(e) => vec![CGroup::new_error(root, e.to_string())],
    }
}

fn load_cgroup_rec(abs_path: PathBuf, rel_path: &PathBuf, sort: SortOrder, stat: usize, processor: &dyn StatProcessor) -> io::Result<CGroup> {
    let mut cgroup = CGroup::new(rel_path.clone());

    // Recurse in to sub directories first
    let dir = abs_path.read_dir()?;

    dir.for_each(|file| {
        if let Ok(file) = file {
            let fname = file.file_name();

            if let Ok(ftype) = file.file_type() {
                if ftype.is_dir() {
                    let mut sub_rel_path = rel_path.clone();
                    sub_rel_path.push(fname);

                    match load_cgroup_rec(file.path(), &sub_rel_path, sort, stat, processor) {
                        Ok(sub_cgroup) => cgroup.children.push(sub_cgroup),
                        Err(e) => cgroup
                            .children
                            .push(CGroup::new_error(sub_rel_path, e.to_string())),
                    }
                }
            }
        }
    });

    // Get the statistic for this cgroup
    match processor.get_stat(&abs_path) {
        Ok(stat) => cgroup.stat = stat,
        Err(e) => cgroup.error = Some(e.to_string()),
    }

    match STATS[stat].stat_type() {
        StatType::Qty => {
            // Non-cumulative quantity
            let child_sum: usize = cgroup.children.iter().map(|c| c.stat).sum();

            if child_sum > 0 {
                if cgroup.stat > 0 {
                    // Add self quantity
                    let mut sub_rel_path = rel_path.clone();
                    sub_rel_path.push("<self>");
                    let mut cg_self = CGroup::new(sub_rel_path);
                    cg_self.stat = cgroup.stat;
                    cgroup.children.push(cg_self);
                }

                cgroup.stat += child_sum;
            }
        }
        StatType::MemQtyCumul => {
            // Cumulative quantity
            if !cgroup.children.is_empty() {
                // Add a <self> node for difference in memory between the sum of the children and this
                let child_sum: usize = cgroup.children.iter().map(|c| c.stat).sum();
                
                if child_sum < cgroup.stat {
                    // Add self quantity
                    let mut sub_rel_path = rel_path.clone();
                    sub_rel_path.push("<self>");
                    let mut cg_self = CGroup::new(sub_rel_path);
                    cg_self.stat = cgroup.stat - child_sum;
                    cgroup.children.push(cg_self);
                }
            }        
        }
    }

    // Sort the children
    match sort {
        SortOrder::NameAsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path)),
        SortOrder::NameDsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path).reverse()),
        SortOrder::SizeAsc => cgroup.children.sort_by(|a, b| a.stat.cmp(&b.stat)),
        SortOrder::SizeDsc => cgroup.children.sort_by(|a, b| a.stat.cmp(&b.stat).reverse()),
    }

    Ok(cgroup)
}

enum StatProcessorError {
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

trait StatProcessor {
    fn get_stat(&self, path: &PathBuf) -> Result<usize, StatProcessorError>;
}

fn get_stat_processor(stat: usize) -> Box<dyn StatProcessor> {
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

struct SingleValueProcessor {
    file: String,
}

impl SingleValueProcessor {
    fn new(file: &str) -> Self {
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

struct KeyedProcessor {
    file: String,
    match_string: String,
    ret_col: usize,
}

impl KeyedProcessor {
    fn new(file: &str, line_start: &str, ret_col: &str) -> Self {
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

struct CountProcessor {
    file: String,
}

impl CountProcessor {
    fn new(file: &str) -> Self {
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
