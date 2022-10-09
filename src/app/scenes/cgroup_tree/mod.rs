mod tree;

use std::{
    io,
    path::Path,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{Block, Borders};

use crate::{
    app::{Action, AppScene, PollResult},
    cgroup::{
        stats::{StatType, STATS},
        SortOrder,
    },
    TermType,
};

use self::tree::CGroupTree;

use super::Scene;

pub struct CGroupTreeScene<'a> {
    debug: bool,
    cgroup2fs: &'a Path,
    tree: CGroupTree<'a>,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
    sort: SortOrder,
    stat: usize,
}

impl<'a> CGroupTreeScene<'a> {
    /// Creates a new cgroup tree scene
    pub fn new(cgroup2fs: &'a Path, debug: bool) -> Self {
        Self {
            debug,
            cgroup2fs,
            tree: Default::default(),
            next_refresh: Instant::now(),
            draws: 0,
            loads: 0,
            sort: SortOrder::NameAsc,
            stat: 0,
        }
    }

    /// Sets the statistic to view
    pub fn set_stat(&mut self, stat: usize) {
        self.stat = stat
    }

    /// Sets the sort order to use
    pub fn set_sort(&mut self, sort: SortOrder) {
        self.sort = sort;
    }

    fn sort_name(&mut self) -> PollResult {
        let new_sort = match self.sort {
            SortOrder::NameAsc => SortOrder::NameDsc,
            _ => SortOrder::NameAsc,
        };

        Some(vec![Action::Sort(new_sort), Action::Reload])
    }

    fn sort_stat(&mut self) -> PollResult {
        let new_sort = match self.sort {
            SortOrder::SizeAsc => SortOrder::SizeDsc,
            _ => SortOrder::SizeAsc,
        };

        Some(vec![Action::Sort(new_sort), Action::Reload])
    }

    fn next_stat(&self, up: bool) -> PollResult {
        let new_stat = if up {
            (self.stat + 1) % STATS.len()
        } else if self.stat == 0 {
            STATS.len() - 1
        } else {
            self.stat - 1
        };

        Some(vec![Action::Stat(new_stat), Action::Reload])
    }

    fn procs(&mut self, threads: bool) -> PollResult {
        self.tree.cgroup().map(|cgroup| {
            vec![
                Action::ProcCGroup(cgroup.path().clone()),
                Action::ProcThreads(threads),
                Action::Scene(AppScene::Procs),
            ]
        })
    }
}

impl<'a> Scene for CGroupTreeScene<'a> {
    fn reload(&mut self) {
        // Build the tree
        self.tree.build_tree(self.cgroup2fs, self.stat, self.sort);
        self.loads += 1;

        // Calculate next refresh time
        self.next_refresh = Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    }

    /// Draws the cgroup tree scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        self.draws += 1;

        // Build block title
        let qty_desc = match STATS[self.stat].stat_type() {
            StatType::MemQtyCumul => "Memory Usage",
            StatType::Qty => "Count",
        };

        let mut title = format!("CGroup {} {} (press 'h' for help)", STATS[self.stat].short_desc(), qty_desc);

        if self.debug {
            title += &format!(" ({} loads, {} draws, {:?})", self.loads, self.draws, self.tree.selected());
        }

        terminal.draw(|f| {
            // Create the block
            let block = Block::default().title(title).borders(Borders::ALL);

            // Create the tree
            self.tree.render(f, block);
        })?;

        Ok(())
    }

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
    }

    /// Key event
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(vec![Action::Exit]),
            KeyCode::Left => self.tree.left(),
            KeyCode::Right => self.tree.right(),
            KeyCode::Down => self.tree.down(),
            KeyCode::Up => self.tree.up(),
            KeyCode::Home => self.tree.first(),
            KeyCode::End => self.tree.last(),
            KeyCode::Char('c') => self.tree.close_all(),
            KeyCode::Char('r') => Some(vec![Action::Reload]),
            KeyCode::Char('n') => self.sort_name(),
            KeyCode::Char('s') => self.sort_stat(),
            KeyCode::Char('p') => self.procs(false),
            KeyCode::Char('t') => self.procs(true),
            KeyCode::Char('z') => Some(vec![Action::Scene(AppScene::StatChoose)]),
            KeyCode::Char('[') => self.next_stat(false),
            KeyCode::Char(']') => self.next_stat(true),
            KeyCode::Char('h') => Some(vec![Action::Scene(AppScene::CgroupTreeHelp)]),
            _ => PollResult::None,
        }
    }
}
