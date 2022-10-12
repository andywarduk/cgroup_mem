pub mod stats;

use std::{
    io::{self, BufReader, BufRead},
    fs::File,
    path::{Path, PathBuf},
};

use crate::file_proc::{get_file_processor, FileProcessor};

use self::stats::{StatType, STATS};

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CGroupSortOrder {
    NameAsc,
    NameDsc,
    StatAsc,
    StatDsc,
}

pub fn load_cgroups(cgroup2fs: &Path, stat: usize, sort: CGroupSortOrder) -> Vec<CGroup> {
    let rel_path = PathBuf::new();

    let processor = get_file_processor(STATS[stat].def()).unwrap();

    match load_cgroup_rec(cgroup2fs.to_path_buf(), &rel_path, sort, stat, &*processor) {
        Ok(cgroup) => {
            if cgroup.error.is_some() && !cgroup.children.is_empty() {
                // Handle case where this is no file in the root directory
                cgroup.children
            } else {
                vec![cgroup]
            }
        }
        Err(e) => vec![CGroup::new_error(rel_path, e.to_string())],
    }
}

fn load_cgroup_rec(
    abs_path: PathBuf,
    rel_path: &Path,
    sort: CGroupSortOrder,
    stat: usize,
    processor: &dyn FileProcessor,
) -> io::Result<CGroup> {
    let mut cgroup = CGroup::new(rel_path.to_path_buf());

    // Recurse in to sub directories first
    let dir = abs_path.read_dir()?;

    dir.for_each(|file| {
        if let Ok(file) = file {
            let fname = file.file_name();

            if let Ok(ftype) = file.file_type() {
                if ftype.is_dir() {
                    let mut sub_rel_path = rel_path.to_path_buf();
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
        Err(e) => {
            cgroup.error = Some(e.to_string());

            if let Ok(has_controller) = cgroup_has_memory_controller(&abs_path) {
                if !has_controller {
                    cgroup.error = Some("No memory controller".into());
                }
            }
        }
    }

    match STATS[stat].stat_type() {
        StatType::Qty => {
            // Non-cumulative quantity
            let child_sum: usize = cgroup.children.iter().map(|c| c.stat).sum();

            if child_sum > 0 {
                if cgroup.stat > 0 {
                    // Add self quantity
                    let mut sub_rel_path = rel_path.to_path_buf();
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
                    let mut sub_rel_path = rel_path.to_path_buf();
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
        CGroupSortOrder::NameAsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path)),
        CGroupSortOrder::NameDsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path).reverse()),
        CGroupSortOrder::StatAsc => cgroup.children.sort_by(|a, b| a.stat.cmp(&b.stat)),
        CGroupSortOrder::StatDsc => cgroup.children.sort_by(|a, b| a.stat.cmp(&b.stat).reverse()),
    }

    Ok(cgroup)
}

fn cgroup_has_memory_controller(path: &Path) -> io::Result<bool> {
    let mut path = path.to_path_buf();
    path.push("cgroup.controllers");

    let file = File::open(path)?;

    match BufReader::new(file).lines().next() {
        None => Ok(false),
        Some(Err(e)) => Err(e)?,
        Some(Ok(line)) => {
            Ok(line
                .split_whitespace()
                .any(|s| s == "memory")
            )
        }
    }
}
