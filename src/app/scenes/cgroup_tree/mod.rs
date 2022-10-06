mod cgroup;
mod tree;

use std::{
    io,
    time::{Instant, Duration},
};

use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use tui::widgets::{Block, Borders};

use cgroup::SortOrder;

use crate::{
    app::{AppScene, PollResult},
    TermType,
};

use self::tree::CGroupTree;

use super::Scene;

pub struct CGroupTreeScene<'a> {
    debug: bool,
    tree: CGroupTree<'a>,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
    sort: SortOrder,
}

impl<'a> CGroupTreeScene<'a> {
    /// Creates a new cgroup tree scene
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            tree: Default::default(),
            next_refresh: Instant::now(),
            draws: 0,
            loads: 0,
            sort: SortOrder::NameAsc,
        }
    }

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
    }

    fn frame_title(&self, base: &str) -> String {
        // Build block title
        let mut title = base.to_string();

        if self.debug {
            title += &format!(" ({} loads, {} draws, {:?})", self.loads, self.draws, self.tree.selected());
        }

        title
    }
}

impl<'a> Scene for CGroupTreeScene<'a> {
    fn reload(&mut self) {
        // Build the tree
        self.tree.build_tree(self.sort);
        self.loads += 1;

        // Calculate next refresh time
        self.next_refresh = Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    }

    /// Draws the cgroup tree scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        self.draws += 1;

        let title = self.frame_title("CGroup Memory Usage (press 'h' for help)");

        terminal.draw(|f| {
            // Create the block
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL);

            // Create the tree
            self.tree.render(f, block);
        })?;

        Ok(())
    }

    fn poll(&mut self) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = if let Some(poll_duration) = self.time_to_refresh() {
                if event::poll(poll_duration)? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            // A key was pressed
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => PollResult::Exit,
                                KeyCode::Left => {
                                    self.tree.left();
                                    PollResult::Redraw
                                }
                                KeyCode::Right => {
                                    self.tree.right();
                                    PollResult::Redraw
                                }
                                KeyCode::Down => {
                                    self.tree.down();
                                    PollResult::Redraw
                                }
                                KeyCode::Up => {
                                    self.tree.up();
                                    PollResult::Redraw
                                }
                                KeyCode::Home => {
                                    self.tree.first();
                                    PollResult::Redraw
                                }
                                KeyCode::End => {
                                    self.tree.last();
                                    PollResult::Redraw
                                }
                                KeyCode::Char('c') => {
                                    self.tree.close_all();
                                    PollResult::Redraw
                                }
                                KeyCode::Char('r') => PollResult::Reload,
                                KeyCode::Char('n') => {
                                    match self.sort {
                                        SortOrder::NameAsc => self.sort = SortOrder::NameDsc,
                                        _ => self.sort = SortOrder::NameAsc,
                                    }
                                    PollResult::Reload
                                }
                                KeyCode::Char('s') => {
                                    match self.sort {
                                        SortOrder::SizeAsc => self.sort = SortOrder::SizeDsc,
                                        _ => self.sort = SortOrder::SizeAsc,
                                    }
                                    PollResult::Reload
                                }
                                KeyCode::Char('p') => {
                                    println!("{:?}", self.tree.cgroup());
                                    PollResult::None
                                }
                                KeyCode::Char('h') => PollResult::Scene(AppScene::Help),
                                _ => PollResult::None,
                            }
                        }
                        Event::Mouse(mouse_event) => {
                            // Mouse event
                            match mouse_event.kind {
                                MouseEventKind::ScrollDown => {
                                    self.tree.down();
                                    PollResult::Redraw
                                }
                                MouseEventKind::ScrollUp => {
                                    self.tree.up();
                                    PollResult::Redraw
                                }
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
