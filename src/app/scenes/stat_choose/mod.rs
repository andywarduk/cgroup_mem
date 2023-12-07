use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use super::Scene;
use crate::app::{Action, AppScene, PollResult};
use crate::cgroup::stats::STATS;
use crate::TermType;

pub struct StatChooseScene<'a> {
    items: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a> StatChooseScene<'a> {
    pub fn new() -> Self {
        // Build list items
        let items = STATS
            .iter()
            .enumerate()
            .map(|(i, stat)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(" {:>2} ", i + 1),
                        Style::default().add_modifier(Modifier::DIM),
                    ),
                    Span::from(stat.desc()),
                ]))
            })
            .collect();

        Self {
            items,
            state: ListState::default(),
        }
    }

    pub fn set_stat(&mut self, stat: usize) {
        self.state.select(Some(stat));
    }

    #[must_use]
    fn up(&mut self) -> PollResult {
        if let Some(cur) = self.state.selected() {
            if cur > 0 {
                self.state.select(Some(cur - 1));
                Some(vec![])
            } else {
                None
            }
        } else {
            self.state.select(Some(self.items.len() - 1));
            Some(vec![])
        }
    }

    #[must_use]
    fn down(&mut self) -> PollResult {
        if let Some(cur) = self.state.selected() {
            if cur < self.items.len() - 1 {
                self.state.select(Some(cur + 1));
                Some(vec![])
            } else {
                None
            }
        } else {
            self.state.select(Some(0));
            Some(vec![])
        }
    }

    #[must_use]
    fn select(&mut self) -> PollResult {
        self.state
            .selected()
            .map(|selected| vec![Action::Stat(selected), Action::Scene(AppScene::CGroupTree)])
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

            // Create the block
            let block = Block::default()
                .title("Displayed Statistic")
                .borders(Borders::ALL);

            // Create the list
            let list = List::new(self.items.clone())
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            // Draw the paragraph
            f.render_stateful_widget(list, size, &mut self.state);
        })?;

        Ok(())
    }

    /// Key events
    #[must_use]
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                Some(vec![Action::Scene(AppScene::CGroupTree)])
            }
            KeyCode::Down => self.down(),
            KeyCode::Up => self.up(),
            KeyCode::Enter | KeyCode::Char(' ') => self.select(),
            _ => None,
        }
    }
}
