mod table;

use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{Block, Borders};

use self::table::ProcsTable;
use super::Scene;
use crate::{
    app::{Action, AppScene, PollResult},
    cgroup::{
        stats::{ProcStatType, STATS},
        CGroupSortOrder,
    },
    proc::ProcSortOrder,
    TermType,
};

pub struct ProcsScene<'a> {
    debug: bool,
    cgroup2fs: &'a Path,
    cgroup: PathBuf,
    sort: ProcSortOrder,
    proc_sort: ProcSortOrder,
    stat: usize,
    threads: bool,
    include_children: bool,
    table: ProcsTable<'a>,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
}

impl<'a> ProcsScene<'a> {
    /// Creates a new process scene
    pub fn new(cgroup2fs: &'a Path, debug: bool) -> Self {
        Self {
            debug,
            cgroup2fs,
            cgroup: PathBuf::new(),
            sort: ProcSortOrder::CmdAsc,
            proc_sort: ProcSortOrder::CmdAsc,
            stat: 0,
            threads: false,
            include_children: false,
            table: Default::default(),
            next_refresh: Instant::now(),
            draws: 0,
            loads: 0,
        }
    }

    /// Sets the cgroup to display
    pub fn set_cgroup(&mut self, mut path: PathBuf) {
        if path.file_name() == Some(OsStr::new("<self>")) {
            path.pop();
        }

        self.cgroup = path;

        self.table.reset();
    }

    /// Sets the statistic to display
    pub fn set_stat(&mut self, stat: usize) {
        self.stat = stat;
        self.resolve_sort();
    }

    /// Sets the sort order to use
    pub fn set_sort(&mut self, sort: ProcSortOrder) {
        self.proc_sort = sort;
        self.resolve_sort();
    }

    /// Sets the sort order to use
    pub fn set_cgroup_sort(&mut self, sort: CGroupSortOrder) {
        match sort {
            CGroupSortOrder::NameAsc => self.proc_sort = ProcSortOrder::CmdAsc,
            CGroupSortOrder::NameDsc => self.proc_sort = ProcSortOrder::CmdDsc,
            CGroupSortOrder::StatAsc => self.proc_sort = ProcSortOrder::StatAsc,
            CGroupSortOrder::StatDsc => self.proc_sort = ProcSortOrder::StatDsc,
        }
        self.resolve_sort();
    }

    /// Set display mode
    pub fn set_mode(&mut self, threads: bool, include_children: bool) {
        self.threads = threads;
        self.include_children = include_children;
    }

    fn sort_pid(&mut self) -> PollResult {
        let new_sort = match self.sort {
            ProcSortOrder::PidAsc => ProcSortOrder::PidDsc,
            _ => ProcSortOrder::PidAsc,
        };

        Some(vec![Action::ProcSort(new_sort), Action::Reload])
    }

    fn sort_name(&mut self) -> PollResult {
        let new_sort = match self.sort {
            ProcSortOrder::CmdAsc => ProcSortOrder::CmdDsc,
            _ => ProcSortOrder::CmdAsc,
        };

        Some(vec![Action::ProcSort(new_sort), Action::Reload])
    }

    fn sort_stat(&mut self) -> PollResult {
        let new_sort = match self.sort {
            ProcSortOrder::StatAsc => ProcSortOrder::StatDsc,
            _ => ProcSortOrder::StatAsc,
        };

        Some(vec![Action::ProcSort(new_sort), Action::Reload])
    }

    fn resolve_sort(&mut self) {
        self.sort = if STATS[self.stat].proc_stat_type() == ProcStatType::None {
            match self.proc_sort {
                ProcSortOrder::StatAsc => ProcSortOrder::PidAsc,
                ProcSortOrder::StatDsc => ProcSortOrder::PidDsc,
                s => s,
            }
        } else {
            self.proc_sort
        }
    }

    fn next_stat(&self, up: bool) -> PollResult {
        let mut new_stat = self.stat;

        loop {
            new_stat = if up {
                (new_stat + 1) % STATS.len()
            } else if new_stat == 0 {
                STATS.len() - 1
            } else {
                new_stat - 1
            };

            if STATS[new_stat].proc_stat_type() != ProcStatType::None {
                break;
            }
        }

        Some(vec![Action::Stat(new_stat), Action::Reload])
    }
}

impl<'a> Scene for ProcsScene<'a> {
    /// Reloads the process scene
    fn reload(&mut self) {
        // Build the tree
        self.table.build_table(
            self.cgroup2fs,
            &self.cgroup,
            self.threads,
            self.include_children,
            self.stat,
            self.sort,
        );
        self.loads += 1;

        // Calculate next refresh time
        self.next_refresh = Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    }

    /// Draws the process scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        self.draws += 1;

        terminal.draw(|f| {
            // Create the title
            let mut cgroup_str = self.cgroup.to_string_lossy();

            if cgroup_str == "" {
                cgroup_str = "/".into();
            }

            let ptype = match (self.threads, self.include_children) {
                (false, false) => "Processes",
                (false, true) => "Hierarchy Processes",
                (true, false) => "Threads",
                (true, true) => "Hierarchy Threads",
            };

            let mut title = format!("{} for {}", ptype, cgroup_str);

            if self.debug {
                title += &format!(
                    " ({} loads, {} draws, {:?})",
                    self.loads,
                    self.draws,
                    self.table.selected()
                );
            }

            // Create the block
            let block = Block::default().title(title).borders(Borders::ALL);

            // Draw the table
            self.table.render(f, block);
        })?;

        Ok(())
    }

    /// Key event
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q')
            | KeyCode::Esc
            | KeyCode::Char('p')
            | KeyCode::Char('t')
            | KeyCode::Char('P')
            | KeyCode::Char('T') => Some(vec![Action::Scene(AppScene::CGroupTree)]),
            KeyCode::Up => self.table.up(),
            KeyCode::Down => self.table.down(),
            KeyCode::PageUp => self.table.pgup(),
            KeyCode::PageDown => self.table.pgdown(),
            KeyCode::Home => self.table.home(),
            KeyCode::End => self.table.end(),
            KeyCode::Char('i') => self.sort_pid(),
            KeyCode::Char('n') => self.sort_name(),
            KeyCode::Char('s') => self.sort_stat(),
            KeyCode::Char('[') => self.next_stat(false),
            KeyCode::Char(']') => self.next_stat(true),
            KeyCode::Char('a') => Some(vec![
                Action::ProcMode(!self.threads, self.include_children),
                Action::Reload,
            ]),
            KeyCode::Char('c') => Some(vec![
                Action::ProcMode(self.threads, !self.include_children),
                Action::Reload,
            ]),
            KeyCode::Char('h') => Some(vec![Action::Scene(AppScene::ProcsHelp)]),
            KeyCode::Char('r') => Some(vec![Action::Reload]),
            _ => None,
        }
    }

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
    }
}
