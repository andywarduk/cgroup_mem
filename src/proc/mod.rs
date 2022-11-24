use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{
    cgroup::stats::{ProcStatType, STATS},
    file_proc::{get_file_processor, FileProcessor, FileProcessorError, SingleValueProcessor},
};

pub struct Proc {
    pub pid: usize,
    pub cmd: String,
    pub stat: Result<usize, FileProcessorError>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcSortOrder {
    PidAsc,
    PidDsc,
    StatAsc,
    StatDsc,
    CmdAsc,
    CmdDsc,
}

pub fn load_procs(
    cgroup2fs: &Path,
    cgroup: &Path,
    include_children: bool,
    threads: bool,
    stat: usize,
    sort: ProcSortOrder,
) -> io::Result<Vec<Proc>> {
    // Get PID list
    let mut path = cgroup2fs.to_path_buf();
    path.extend(cgroup);

    let pids = load_pids(path.as_path(), threads, include_children)?;

    // Create file processor for getting command line / comm
    let file_processor = SingleValueProcessor::default();

    // Create the stats processor (if required)
    let stat_processor = get_file_processor(STATS[stat].proc_def());
    let stat_type = STATS[stat].proc_stat_type();

    let mut procs: Vec<Proc> = pids
        .into_iter()
        .map(|pid| {
            // Build /proc path
            let proc_path = PathBuf::from(format!("/proc/{}", pid));

            // Get command line
            let cmd = match file_processor.get_value(&proc_path.join("cmdline")) {
                Ok(string) => string
                    .chars()
                    .map(|c| if c == '\x00' { ' ' } else { c })
                    .collect(),
                Err(_) => match file_processor.get_value(&proc_path.join("comm")) {
                    Ok(string) => format!("[{}]", string),
                    Err(_) => "<Unknown>".into(),
                },
            };

            // Get stat
            let stat = if let Some(processor) = &stat_processor {
                let mut value = processor.get_stat(&proc_path);

                match stat_type {
                    ProcStatType::MemQtyKb => {
                        value = match value {
                            Ok(value) => Ok(value * 1024),
                            v => v,
                        }
                    }
                    _ => panic!("Unexpected stat type"),
                }

                value
            } else {
                Ok(0)
            };

            Proc { pid, cmd, stat }
        })
        .collect();

    // Sort the processes
    match sort {
        ProcSortOrder::PidAsc => procs.sort_by(|a, b| a.pid.cmp(&b.pid)),
        ProcSortOrder::PidDsc => procs.sort_by(|a, b| a.pid.cmp(&b.pid).reverse()),
        ProcSortOrder::CmdAsc => procs.sort_by(|a, b| a.cmd.cmp(&b.cmd)),
        ProcSortOrder::CmdDsc => procs.sort_by(|a, b| a.cmd.cmp(&b.cmd).reverse()),
        ProcSortOrder::StatAsc => {
            procs.sort_by(|a, b| {
                a.stat
                    .as_ref()
                    .unwrap_or(&0)
                    .cmp(b.stat.as_ref().unwrap_or(&0))
            });
        }
        ProcSortOrder::StatDsc => {
            procs.sort_by(|a, b| {
                a.stat
                    .as_ref()
                    .unwrap_or(&0)
                    .cmp(b.stat.as_ref().unwrap_or(&0))
                    .reverse()
            });
        }
    }

    Ok(procs)
}

fn load_pids(cgroup_path: &Path, threads: bool, include_children: bool) -> io::Result<Vec<usize>> {
    let mut path = cgroup_path.to_path_buf();

    // Get PIDs for the passed cgroup
    if threads {
        path.push("cgroup.threads");
    } else {
        path.push("cgroup.procs");
    }

    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut pids = buf_reader
        .lines()
        .map(|line| {
            let line = line?;

            match line.parse::<usize>() {
                Ok(n) => Ok(n),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
        })
        .collect::<io::Result<Vec<usize>>>()?;

    // Recurse in to child cgroups
    if include_children {
        for child_pids in cgroup_path
            .read_dir()?
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|e| e.is_dir())
            .map(|e| load_pids(&e, threads, true))
            .filter_map(|e| e.ok())
        {
            pids.extend(child_pids);
        }
    }

    Ok(pids)
}
