mod table;

use std::{
    ffi::OsStr,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, MouseEventKind};
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

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
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

    /// Event poll loop
    fn poll(&mut self) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = if let Some(poll_duration) = self.time_to_refresh() {
                if event::poll(poll_duration)? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            // A key was pressed
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
                                KeyCode::Char('n') => {
                                    let new_sort = match self.sort {
                                        SortOrder::SizeAsc => SortOrder::NameDsc,
                                        _ => SortOrder::NameAsc,
                                    };

                                    PollResult::SceneParms(
                                        AppScene::Procs,
                                        vec![SceneChangeParm::NewSort(new_sort)],
                                    )
                                }
                                KeyCode::Char('s') => {
                                    let new_sort = match self.sort {
                                        SortOrder::SizeAsc => SortOrder::SizeDsc,
                                        _ => SortOrder::SizeAsc,
                                    };

                                    PollResult::SceneParms(
                                        AppScene::Procs,
                                        vec![SceneChangeParm::NewSort(new_sort)],
                                    )
                                }
                                KeyCode::Char('h') => PollResult::Scene(AppScene::ProcsHelp),
                                KeyCode::Char('r') => PollResult::Reload,
                                _ => PollResult::None,
                            }
                        }
                        Event::Mouse(mouse_event) => {
                            // Mouse event
                            match mouse_event.kind {
                                MouseEventKind::ScrollDown => self.table.down(),
                                MouseEventKind::ScrollUp => self.table.up(),
                                // TODO MouseEventKind::Up() => {
                                //     mouse_event.column
                                //     mouse_event.row
                                // }
                                _ => PollResult::None,
                            }
                        }
                        Event::Resize(_, _) => {
                            // Break out to redraw
                            PollResult::Redraw
                        }
                        _ => {
                            // All other events are ignored
                            PollResult::None
                        }
                    }
                } else {
                    // No event in the timeout period
                    PollResult::Reload
                }
            } else {
                // No time left
                PollResult::Reload
            }
        }

        Ok(result)
    }
}
