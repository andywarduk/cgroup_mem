use std::io;

use crossterm::event::{self, Event, KeyCode, MouseEventKind};

use tui::{
    style::{Style, Modifier},
    widgets::{Block, Borders, List, ListState, ListItem},
};

use crate::{
    app::{AppScene, PollResult, SceneChangeParm},
    TermType, cgroup::stats::STATS,
};

use super::Scene;

pub struct StatChooseScene<'a> {
    items: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a> StatChooseScene<'a> {
    pub fn new(_debug: bool) -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    fn up(&mut self) -> PollResult {
        if let Some(cur) = self.state.selected() {
            if cur > 0 {
                self.state.select(Some(cur - 1));
                PollResult::Redraw
            } else {
                PollResult::None
            }
        } else {
            self.state.select(Some(self.items.len() - 1));
            PollResult::Redraw
        }
    }

    fn down(&mut self) -> PollResult {
        if let Some(cur) = self.state.selected() {
            if cur < self.items.len() - 1 {
                self.state.select(Some(cur + 1));
                PollResult::Redraw
            } else {
                PollResult::None
            }
        } else {
            self.state.select(Some(0));
            PollResult::Redraw
        }
    }
}

impl<'a> Scene for StatChooseScene<'a> {
    /// Reloads the scene
    fn reload(&mut self) {}

    /// Draws the stat choose scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        terminal.draw(|f| {
            // Get the size of the frame
            let size = f.size();

            // Build list items
            self.items = STATS.iter().map(|stat| {
                ListItem::new(stat.desc())
            }).collect();

            // Create the list
            let list = List::new(self.items.clone())
                .block(Block::default()
                .title("Displayed Statistic")
                .borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            // Draw the paragraph
            f.render_stateful_widget(list, size, &mut self.state);
        })?;

        Ok(())
    }

    /// Event poll loop
    fn poll(&mut self) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = match event::read()? {
                Event::Key(key_event) => {
                    // A key was pressed
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                             PollResult::Scene(AppScene::CGroupTree)
                        }
                        KeyCode::Down => self.down(),
                        KeyCode::Up => self.up(),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(selected) = self.state.selected() {
                                PollResult::SceneParms(AppScene::CGroupTree, vec![
                                    SceneChangeParm::Stat(selected)
                                ])
                            } else {
                                PollResult::None
                            }
                        }
                        _ => PollResult::None,
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Mouse event
                    match mouse_event.kind {
                        MouseEventKind::ScrollDown => self.down(),
                        MouseEventKind::ScrollUp => self.up(),
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
        }

        Ok(result)
    }
}
