mod table;

use std::{
    ffi::OsStr,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{Block, Borders};

use crate::{
    app::{AppScene, PollResult, SceneChangeParm},
    cgroup::SortOrder,
    TermType,
};

use self::table::ProcsTable;

use super::Scene;

pub struct ProcsScene<'a> {
    debug: bool,
    cgroup: PathBuf,
    sort: SortOrder,
    stat: usize,
    threads: bool,
    table: ProcsTable<'a>,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
}

impl<'a> ProcsScene<'a> {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            cgroup: PathBuf::new(),
            sort: SortOrder::NameAsc,
            stat: 0,
            threads: false,
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
    }

    /// Sets the sort order to use
    pub fn set_sort(&mut self, sort: SortOrder) {
        self.sort = sort;
    }

    pub fn set_threads(&mut self, threads: bool) {
        self.threads = threads
    }

    fn sort_name(&mut self) -> PollResult {
        let new_sort = match self.sort {
            SortOrder::NameAsc => SortOrder::NameDsc,
            _ => SortOrder::NameAsc,
        };

        PollResult::SceneParms(
            AppScene::Procs,
            vec![SceneChangeParm::Sort(new_sort)],
        )
    }

    fn sort_stat(&mut self) -> PollResult {
        let new_sort = match self.sort {
            SortOrder::SizeAsc => SortOrder::SizeDsc,
            _ => SortOrder::SizeAsc,
        };

        PollResult::SceneParms(
            AppScene::Procs,
            vec![SceneChangeParm::Sort(new_sort)],
        )
    }
}

impl<'a> Scene for ProcsScene<'a> {
    /// Reloads the process scene
    fn reload(&mut self) {
        // Build the tree
        self.table.build_table(&self.cgroup, self.threads, self.stat, self.sort);
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

            let ptype = if self.threads { "Threads" } else { "Processes" };

            let mut title = format!("{} for {}", ptype, cgroup_str);

            if self.debug {
                title += &format!(" ({} loads, {} draws, {:?})", self.loads, self.draws, self.table.selected());
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
            | KeyCode::Char('t') => PollResult::Scene(AppScene::CGroupTree),
            KeyCode::Up => self.table.up(),
            KeyCode::Down => self.table.down(),
            KeyCode::PageUp => self.table.pgup(),
            KeyCode::PageDown => self.table.pgdown(),
            KeyCode::Home => self.table.home(),
            KeyCode::End => self.table.end(),
            KeyCode::Char('n') => self.sort_name(),
            KeyCode::Char('s') => self.sort_stat(),
            KeyCode::Char('h') => PollResult::Scene(AppScene::ProcsHelp),
            KeyCode::Char('r') => PollResult::Reload,
            _ => PollResult::None,
        }
    }

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
    }
}
