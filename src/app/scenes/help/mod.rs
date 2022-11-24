use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph};

use super::Scene;
use crate::app::{Action, AppScene, PollResult};
use crate::TermType;

enum HelpLine<'a> {
    Line(&'a str),
    Key(&'a str, &'a str),
}

#[derive(Default)]
pub struct HelpScene<'a> {
    lines: Vec<HelpLine<'a>>,
    max_key: usize,
    changed: bool,
    cur_scroll_x: u16,
    max_scroll_x: u16,
    cur_scroll_y: u16,
    max_scroll_y: u16,
}

impl<'a> HelpScene<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_line(&mut self, line: &'a str) {
        self.lines.push(HelpLine::Line(line));
        self.changed = true;
    }

    pub fn add_key(&mut self, key: &'a str, desc: &'a str) {
        self.lines.push(HelpLine::Key(key, desc));
        self.max_key = std::cmp::max(self.max_key, key.len());
        self.changed = true;
    }

    #[must_use]
    fn scroll_help_up(&mut self) -> PollResult {
        if self.cur_scroll_y > 0 {
            self.cur_scroll_y -= 1;
            Some(vec![])
        } else {
            None
        }
    }

    #[must_use]
    fn scroll_help_down(&mut self) -> PollResult {
        if self.cur_scroll_y < self.max_scroll_y {
            self.cur_scroll_y += 1;
            Some(vec![])
        } else {
            None
        }
    }

    #[must_use]
    fn scroll_help_left(&mut self) -> PollResult {
        if self.cur_scroll_x > 0 {
            self.cur_scroll_x -= 1;
            Some(vec![])
        } else {
            None
        }
    }

    #[must_use]
    fn scroll_help_right(&mut self) -> PollResult {
        if self.cur_scroll_x < self.max_scroll_x {
            self.cur_scroll_x += 1;
            Some(vec![])
        } else {
            None
        }
    }
}

impl<'a> Scene for HelpScene<'a> {
    /// Reloads the help scene
    fn reload(&mut self) {}

    /// Draws the help scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        terminal.draw(|f| {
            // Get the size of the frame
            let size = f.size();

            // Create block
            let block = Block::default().title("Help").borders(Borders::ALL);

            // Create text
            let text: Vec<Spans<'a>> = self
                .lines
                .iter()
                .map(|line| match &line {
                    HelpLine::Line(line) => Spans::from(Span::<'a>::raw(*line)),
                    HelpLine::Key(key, desc) => Spans::from(vec![
                        Span::styled(
                            format!("  {:<width$}  ", key, width = self.max_key),
                            Style::default().fg(Color::Red),
                        ),
                        Span::<'a>::raw(*desc),
                    ]),
                })
                .collect();

            // Work out scroll bounds
            let inner_rect = block.inner(size);

            let lines = text.len() as u16;
            let height = inner_rect.height;

            self.max_scroll_y = lines.saturating_sub(height);

            if self.cur_scroll_y > self.max_scroll_y {
                self.cur_scroll_y = self.max_scroll_y;
            }

            let max_width = text.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
            let width = inner_rect.width;

            self.max_scroll_x = max_width.saturating_sub(width);

            if self.cur_scroll_x > self.max_scroll_x {
                self.cur_scroll_x = self.max_scroll_x;
            }

            // Create the paragraph
            let para = Paragraph::new(text)
                .block(block)
                .scroll((self.cur_scroll_y, self.cur_scroll_x));

            // Draw the paragraph
            f.render_widget(para, size);
        })?;

        Ok(())
    }

    /// Key event
    #[must_use]
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                Some(vec![Action::Scene(AppScene::CGroupTree)])
            }
            KeyCode::Down => self.scroll_help_down(),
            KeyCode::Up => self.scroll_help_up(),
            KeyCode::Left => self.scroll_help_left(),
            KeyCode::Right => self.scroll_help_right(),
            _ => None,
        }
    }
}
