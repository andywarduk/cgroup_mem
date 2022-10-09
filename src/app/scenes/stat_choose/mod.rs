use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use tui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::{
    app::{AppScene, PollResult, SceneChangeParm},
    cgroup::stats::STATS,
    TermType,
};

use super::Scene;

pub struct StatChooseScene<'a> {
    items: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a> StatChooseScene<'a> {
    pub fn new() -> Self {
        // Build list items
        let items = STATS
            .iter()
            .map(|stat| ListItem::new(stat.desc()))
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
                PollResult::Redraw
            } else {
                PollResult::None
            }
        } else {
            self.state.select(Some(self.items.len() - 1));
            PollResult::Redraw
        }
    }

    #[must_use]
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

    #[must_use]
    fn select(&mut self) -> PollResult {
        if let Some(selected) = self.state.selected() {
            PollResult::SceneParms(
                AppScene::CGroupTree,
                vec![SceneChangeParm::Stat(selected)],
            )
        } else {
            PollResult::None
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

            // Create the block
            let block = Block::default().title("Displayed Statistic").borders(Borders::ALL);

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
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q')
            | KeyCode::Char('h')
            | KeyCode::Esc => PollResult::Scene(AppScene::CGroupTree),
            KeyCode::Down => self.down(),
            KeyCode::Up => self.up(),
            KeyCode::Enter | KeyCode::Char(' ') => self.select(),
            _ => PollResult::None,
        }
    }
}
