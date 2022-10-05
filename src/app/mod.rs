mod cgroup;

use std::{
    io,
    mem,
    time::{Duration, Instant},
};
use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use tui::{
    style::{Modifier, Style, Color},
    widgets::{Block, Borders, Paragraph},
    text::{Text, Span, Spans},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::TermType;
use cgroup::{SortOrder, CGroup, load_cgroups};

#[derive(PartialEq, Eq)]
enum PollResult {
    None,
    Redraw,
    Reload,
    Exit,
}

enum AppScene {
    CGroupTree,
    Help
}

pub struct App<'a> {
    scene: AppScene,
    debug: bool,
    terminal: &'a mut TermType,
    tree_items: Vec<TreeItem<'a>>,
    tree_state: TreeState,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
    sort: SortOrder,
    help_scroll: u16,
}

impl<'a> App<'a> {
    /// Creates the app
    pub fn new(terminal: &'a mut TermType, debug: bool) -> Self {
        Self {
            scene: AppScene::CGroupTree,
            debug,
            terminal,
            tree_items: Vec::new(),
            tree_state: TreeState::default(),
            next_refresh: Instant::now(),
            draws: 0,
            loads: 0,
            sort: SortOrder::NameAsc,
            help_scroll: 0,
        }
    }

    /// Main application loop
    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut reload = true;

        loop {
            if reload {
                // Build the tree
                self.build_tree();

                // Calculate next refresh time
                self.next_refresh = Instant::now().checked_add(Duration::from_secs(5)).unwrap();

                reload = false;
            }
            
            // Draw the scene
            self.draw()?;
    
            // Poll events
            match self.poll()? {
                PollResult::Exit => break,
                PollResult::Redraw => (),
                PollResult::Reload => reload = true,
                PollResult::None => unreachable!(),
            }
        }
    
        Ok(())
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

    /// Draws the scene
    fn draw(&mut self) -> Result<(), io::Error> {
        self.draws += 1;

        match self.scene {
            AppScene::CGroupTree => self.draw_cgroup_tree(),
            AppScene::Help => self.draw_help(),
        }
    }

    /// Draws the cgroup tree scene
    fn draw_cgroup_tree(&mut self) -> Result<(), io::Error> {
        let title = self.frame_title("CGroup Memory Usage (press 'h' for help)");

        self.terminal.draw(|f| {
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

    /// Draws the help scene
    fn draw_help(&mut self) -> Result<(), io::Error> {
        let title = self.frame_title("Help");

        self.terminal.draw(|f| {
            // Get the size of the frame
            let size = f.size();

            // Create the help text
            let mut text = Vec::new();

            let add_line = |text: &mut Vec<_>, string: String| {
                text.push(Spans::from(Span::raw(string)));
            };

            let add_key = |text: &mut Vec<_>, key: String, description: String| {
                text.push(Spans::from(vec![
                    Span::styled(format!("  {:<12}", key), Style::default().fg(Color::Red)),
                    Span::raw(" "),
                    Span::raw(description)
                ]));
            };

            add_line(&mut text, "Key bindings for cgroup memory display:".into());
            add_line(&mut text, "".into());

            add_key(&mut text, "Up Arrow".into(), "Move selection up.".into());
            add_key(&mut text, "Down Arrow".into(), "Move selection down.".into());
            add_key(&mut text, "Left Arrow".into(), "Collapse tree node if on a parent node or move to parent otherwise.".into());
            add_key(&mut text, "Right Arrow".into(), "Expand tree node if on a parent node.".into());
            add_key(&mut text, "Home".into(), "Move selection to the top.".into());
            add_key(&mut text, "End".into(), "Move selection to the end.".into());
            add_key(&mut text, "n".into(), "Sort by cgroup name. Pressing again toggles ascending / descending sort order.".into());
            add_key(&mut text, "s".into(), "Sort by cgroup memory usage. Pressing again toggles ascending / descending sort order.".into());
            add_key(&mut text, "h".into(), "Shows this help screen.".into());
            add_key(&mut text, "Esc / q".into(), "Exit the program.".into());

            add_line(&mut text, "".into());
            add_line(&mut text, "Press q, h or Esc to exit help".into());

            // Create the paragraph
            let para = Paragraph::new(text)
                .block(Block::default().title(title).borders(Borders::ALL))
                .scroll((self.help_scroll, 0));

            // Draw the paragraph
            f.render_widget(para, size);
        })?;

        Ok(())
    }

    fn frame_title(&self, base: &str) -> String {
        // Build block title
        let mut title = base.to_string();

        if self.debug {
            title += &format!(" ({} loads, {} draws)", self.loads, self.draws);
        }

        title
    }

    /// Polls for events
    fn poll(&mut self) -> Result<PollResult, io::Error> {
        match self.scene {
            AppScene::CGroupTree => self.poll_cgroup_tree(),
            AppScene::Help => self.poll_help(),
        }
    }

    fn poll_cgroup_tree(&mut self) -> Result<PollResult, io::Error> {
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
                                KeyCode::Char('h') => {
                                    self.scene = AppScene::Help;
                                    PollResult::Redraw
                                }
                                _ => PollResult::None
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
                                _ => PollResult::None
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

    fn poll_help(&mut self) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = match event::read()? {
                Event::Key(key_event) => {
                    // A key was pressed
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                            self.scene = AppScene::CGroupTree;
                            PollResult::Reload
                        }
                        KeyCode::Down => self.scroll_help_down(),
                        KeyCode::Up => self.scroll_help_up(),
                        _ => PollResult::None
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Mouse event
                    match mouse_event.kind {
                        MouseEventKind::ScrollDown => self.scroll_help_down(),
                        MouseEventKind::ScrollUp => self.scroll_help_up(),
                        _ => PollResult::None
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
        }

        Ok(result)
    }

    fn scroll_help_up(&mut self) -> PollResult {
        if self.help_scroll > 0 {
            self.help_scroll -= 1;
            PollResult::Redraw
        } else {
            PollResult::None
        }
    }

    fn scroll_help_down(&mut self) -> PollResult {
        if self.help_scroll < u16::MAX {
            self.help_scroll += 1;
            PollResult::Redraw
        } else {
            PollResult::None
        }
    }
}
