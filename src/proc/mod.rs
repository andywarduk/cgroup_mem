use std::{
    fs::File,
    io::{self, BufReader, BufRead},
    path::PathBuf,
};

use crate::{
    cgroup::{
        cgroup_fs_path,
        stats::{STATS, ProcStatType},
        SortOrder
    },
    file_proc::{single_value::SingleValueProcessor, FileProcessor, get_file_processor, FileProcessorError}
};

pub struct Proc {
    pub pid: usize,
    pub cmd: String,
    pub stat: Result<usize, FileProcessorError>,
}

pub fn load_procs(cgroup: &PathBuf, threads: bool, stat: usize, sort: SortOrder) -> io::Result<Vec<Proc>> {
    // Get PID list
    let mut path = cgroup_fs_path();
    path.extend(cgroup);

    let pids = load_pids(path, threads)?;

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
                    .map(|c| if c == '\x00' {' '} else {c})
                    .collect(),
                Err(_) => match file_processor.get_value(&proc_path.join("comm")) {
                    Ok(string) => format!("[{}]", string),
                    Err(_) => "<Unknown>".into()
                }
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
        SortOrder::NameAsc => procs.sort_by(|a, b| a.cmd.cmp(&b.cmd)),
        SortOrder::NameDsc => procs.sort_by(|a, b| a.cmd.cmp(&b.cmd).reverse()),
        SortOrder::SizeAsc => {
            if stat_processor.is_none() {
                procs.sort_by(|a, b| a.pid.cmp(&b.pid));
            } else {
                procs.sort_by(|a, b| a.stat.as_ref().unwrap_or(&0).cmp(b.stat.as_ref().unwrap_or(&0)));
            }
        }
        SortOrder::SizeDsc => {
            if stat_processor.is_none() {
                procs.sort_by(|a, b| a.pid.cmp(&b.pid).reverse());
            } else {
                procs.sort_by(|a, b| a.stat.as_ref().unwrap_or(&0).cmp(b.stat.as_ref().unwrap_or(&0)).reverse());
            }
        }
    }

    Ok(procs)
}

fn load_pids(mut path: PathBuf, threads: bool) -> io::Result<Vec<usize>> {
    if threads {
        path.push("cgroup.threads");
    } else {
        path.push("cgroup.procs");
    }

    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    buf_reader
        .lines()
        .map(|line| {
            let line = line?;
            match line.parse::<usize>() {
                Ok(n) => Ok(n),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
        })
        .collect()
}
