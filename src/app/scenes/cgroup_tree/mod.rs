mod cgroup;

use std::{
    io,
    mem,
    time::{Instant, Duration},
};

use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use tui::{
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use cgroup::{load_cgroups, CGroup, SortOrder};

use crate::{
    app::{AppScene, PollResult},
    TermType,
};

use super::Scene;

pub struct CGroupTreeScene<'a> {
    debug: bool,
    tree_items: Vec<TreeItem<'a>>,
    tree_state: TreeState,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
    sort: SortOrder,
}

impl<'a> CGroupTreeScene<'a> {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            tree_items: Vec::new(),
            tree_state: TreeState::default(),
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

    fn build_tree_level(cgroups: Vec<CGroup>) -> Vec<TreeItem<'a>> {
        let mut tree_items = Vec::with_capacity(cgroups.len());

        for mut cg in cgroups {
            let items = mem::take(&mut cg.take_children());
            let text: Text = cg.into();
            let sub_nodes = Self::build_tree_level(items);
            tree_items.push(TreeItem::new(text, sub_nodes));
        }

        tree_items
    }

    /// Build tree
    fn build_tree(&mut self) {
        // Load cgroup information
        let cgroups = load_cgroups(self.sort);

        // Build tree items
        self.tree_items = Self::build_tree_level(cgroups);

        self.loads += 1;
    }

    fn frame_title(&self, base: &str) -> String {
        // Build block title
        let mut title = base.to_string();

        if self.debug {
            title += &format!(" ({} loads, {} draws)", self.loads, self.draws);
        }

        title
    }
}

impl<'a> Scene for CGroupTreeScene<'a> {
    fn reload(&mut self) {
        // Build the tree
        self.build_tree();

        // Calculate next refresh time
        self.next_refresh = Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    }

    /// Draws the cgroup tree scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        self.draws += 1;

        let title = self.frame_title("CGroup Memory Usage (press 'h' for help)");

        terminal.draw(|f| {
            // Get the size of the frame
            let size = f.size();

            // Create the block
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL);

            // Create the tree
            let tree = Tree::new(self.tree_items.clone())
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            // Draw the tree
            f.render_stateful_widget(tree, size, &mut self.tree_state);
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
                                    self.tree_state.key_left();
                                    PollResult::Redraw
                                }
                                KeyCode::Right => {
                                    self.tree_state.key_right();
                                    PollResult::Redraw
                                }
                                KeyCode::Down => {
                                    self.tree_state.key_down(&self.tree_items);
                                    PollResult::Redraw
                                }
                                KeyCode::Up => {
                                    self.tree_state.key_up(&self.tree_items);
                                    PollResult::Redraw
                                }
                                KeyCode::Home => {
                                    self.tree_state.select_first();
                                    PollResult::Redraw
                                }
                                KeyCode::End => {
                                    self.tree_state.select_last(&self.tree_items);
                                    PollResult::Redraw
                                }
                                KeyCode::Char('c') => {
                                    self.tree_state.close_all();
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
                                KeyCode::Char('h') => PollResult::Scene(AppScene::Help),
                                _ => PollResult::None,
                            }
                        }
                        Event::Mouse(mouse_event) => {
                            // Mouse event
                            match mouse_event.kind {
                                MouseEventKind::ScrollDown => {
                                    self.tree_state.key_down(&self.tree_items);
                                    PollResult::Redraw
                                }
                                MouseEventKind::ScrollUp => {
                                    self.tree_state.key_up(&self.tree_items);
                                    PollResult::Redraw
                                }
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
