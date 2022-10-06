use std::{
    fs::File,
    io::{self, BufRead},
    iter::successors,
    path::PathBuf,
};

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

#[derive(Debug, Clone)]
pub struct CGroup {
    path: PathBuf,
    error: Option<String>,
    memory: usize,
    children: Vec<CGroup>,
}

impl CGroup {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            error: None,
            memory: 0,
            children: Vec::new(),
        }
    }

    fn new_error(path: PathBuf, msg: String) -> Self {
        Self {
            path,
            error: Some(msg),
            memory: 0,
            children: Vec::new(),
        }
    }

    pub fn children(&self) -> &Vec<CGroup> {
        &self.children
    }
}

impl<'a> From<&CGroup> for Text<'a> {
    fn from(cgroup: &CGroup) -> Self {
        let pathstr: String = cgroup.path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into();

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
                let mut spans = format_size(cgroup.memory);
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

pub fn load_cgroups(sort: SortOrder) -> Vec<CGroup> {
    let mut path_buf = PathBuf::new();
    path_buf.push("/sys/fs/cgroup");

    let root = PathBuf::new();

    match load_cgroup_rec(path_buf, &root, sort) {
        Ok(cgroup) => cgroup.children,
        Err(e) => vec![CGroup::new_error(root, e.to_string())],
    }
}

fn load_cgroup_rec(abs_path: PathBuf, rel_path: &PathBuf, sort: SortOrder) -> io::Result<CGroup> {
    let mut cgroup = CGroup::new(rel_path.clone());

    let dir = abs_path.read_dir()?;

    dir.for_each(|file| {
        if let Ok(file) = file {
            let fname = file.file_name();

            if fname == *"memory.current" {
                match get_file_line(&file.path()) {
                    Ok(line) => match line.parse::<usize>() {
                        Ok(mem) => cgroup.memory = mem,
                        Err(e) => cgroup.error = Some(e.to_string()),
                    },
                    Err(e) => cgroup.error = Some(e.to_string()),
                }
            }

            if let Ok(ftype) = file.file_type() {
                if ftype.is_dir() {
                    let mut sub_rel_path = rel_path.clone();
                    sub_rel_path.push(fname);

                    match load_cgroup_rec(file.path(), &sub_rel_path, sort) {
                        Ok(sub_cgroup) => cgroup.children.push(sub_cgroup),
                        Err(e) => cgroup
                            .children
                            .push(CGroup::new_error(sub_rel_path, e.to_string())),
                    }
                }
            }
        }
    });

    match sort {
        SortOrder::NameAsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path)),
        SortOrder::NameDsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path).reverse()),
        SortOrder::SizeAsc => cgroup.children.sort_by(|a, b| a.memory.cmp(&b.memory)),
        SortOrder::SizeDsc => cgroup.children.sort_by(|a, b| a.memory.cmp(&b.memory).reverse()),
    }

    Ok(cgroup)
}

fn get_file_line(path: &PathBuf) -> io::Result<String> {
    let file = File::open(path)?;

    match io::BufReader::new(file)
        .lines()
        .next()
    {
        None => Err(io::ErrorKind::InvalidData)?,
        Some(Err(e)) => Err(e)?,
        Some(Ok(line)) => Ok(line),
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

    let digits = successors(Some(fsize), |&n| (n >= 10_f64).then_some(n / 10_f64)).count();
    let dp = 4 - digits;

    vec![Span::styled(format!("{:>5.*} {}", dp, fsize, POWERS[power]), style)]
}
