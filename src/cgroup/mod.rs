pub mod stats;

use std::{
    fs::File,
    io::{self, BufRead},
    iter::successors,
    path::PathBuf, num::ParseIntError, fmt::Display,
};

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

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

    pub fn children(&self) -> &Vec<CGroup> {
        &self.children
    }
}

impl<'a> From<&CGroup> for Text<'a> {
    fn from(cgroup: &CGroup) -> Self {
        let filename = cgroup.path.file_name();

        let pathstr = match filename {
            Some(f) => {
                f.to_string_lossy().into()
            }
            None => "/".to_string(),
        };

        let path = Span::styled(pathstr, Style::default().add_modifier(Modifier::BOLD));

        Text::from(Spans::from(match &cgroup.error {
            Some(msg) => {
                vec![
                    path,
                    Span::raw(": "),
                    Span::styled(msg.clone(), Style::default().fg(Color::Red)),
                ]
            }
            None => {
                let mut spans = format_size(cgroup.stat);
                spans.push(Span::raw(": "));
                spans.push(path);
                spans
            }
        }))
    }
}

#[derive(Clone, Copy)]
pub enum SortOrder {
    NameAsc,
    NameDsc,
    SizeAsc,
    SizeDsc,
}

pub fn load_cgroups(stat: &str, sort: SortOrder) -> Vec<CGroup> {
    let mut path_buf = PathBuf::new();
    path_buf.push("/sys/fs/cgroup");

    let root = PathBuf::new();

    let processor = get_stat_processor(stat);

    match load_cgroup_rec(path_buf, &root, sort, &*processor) {
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

fn load_cgroup_rec(abs_path: PathBuf, rel_path: &PathBuf, sort: SortOrder, processor: &dyn StatProcessor) -> io::Result<CGroup> {
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

                    match load_cgroup_rec(file.path(), &sub_rel_path, sort, processor) {
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

    if !cgroup.children.is_empty() {
        // Add a <self> node for difference in memory between the sum of the children and this
        let child_sum: usize = cgroup.children.iter().map(|c| c.stat).sum();
        
        if child_sum < cgroup.stat {
            let mut sub_rel_path = rel_path.clone();
            sub_rel_path.push("<self>");
            let mut cg_self = CGroup::new(sub_rel_path);
            cg_self.stat = cgroup.stat - child_sum;
            cgroup.children.push(cg_self);
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

fn get_stat_processor(stat: &str) -> Box<dyn StatProcessor> {
    let split: Vec<&str> = stat.split('/').collect();

    let result: Box<dyn StatProcessor> = if split.len() == 1 {
        Box::new(SingleValueProcessor::new(split[0]))
    } else if split.len() == 3 && split[1].starts_with('=') {
        Box::new(KeyedProcessor::new(split[0], &split[1][1..], split[2]))
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

const POWERS: [&str; 7] = ["b", "k", "M", "G", "T", "P", "E"];
const COLOURS: [Color; 7] = [
    Color::LightGreen,
    Color::LightBlue,
    Color::LightYellow,
    Color::LightRed,
    Color::LightRed,
    Color::LightRed,
    Color::LightRed,
];

fn format_size(size: usize) -> Vec<Span<'static>> {
    let mut fsize = size as f64;
    let mut power = 0;

    while power < 6 && fsize >= 1024_f64 {
        power += 1;
        fsize /= 1024_f64;
    }

    let style = Style::default().fg(COLOURS[power]);

    let dp = if power > 1 {
        let digits = successors(Some(fsize), |&n| (n >= 10_f64).then_some(n / 10_f64)).count();
        4 - digits
    } else {
        0
    };

    vec![Span::styled(format!("{:>5.*} {}", dp, fsize, POWERS[power]), style)]
}
