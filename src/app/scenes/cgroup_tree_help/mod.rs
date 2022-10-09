use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{Action, AppScene, PollResult},
    TermType,
};

use super::Scene;

pub struct CGroupTreeHelpScene {
    help_scroll: u16,
}

impl CGroupTreeHelpScene {
    pub fn new() -> Self {
        Self { help_scroll: 0 }
    }

    fn scroll_help_up(&mut self) -> PollResult {
        if self.help_scroll > 0 {
            self.help_scroll -= 1;
            Some(vec![])
        } else {
            None
        }
    }

    fn scroll_help_down(&mut self) -> PollResult {
        if self.help_scroll < u16::MAX {
            self.help_scroll += 1;
            Some(vec![])
        } else {
            None
        }
    }
}

impl Scene for CGroupTreeHelpScene {
    /// Reloads the help scene
    fn reload(&mut self) {}

    /// Draws the help scene
    fn draw(&mut self, terminal: &mut TermType) -> Result<(), io::Error> {
        terminal.draw(|f| {
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
                    Span::raw(description),
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
            add_key(&mut text, "s".into(), "Sort by memory usage. Pressing again toggles ascending / descending sort order.".into());
            add_key(&mut text, "c".into(), "Collapse all expanded nodes.".into());
            add_key(&mut text, "z".into(), "Select statistic to show.".into());
            add_key(&mut text, "[".into(), "Move to previous statistic.".into());
            add_key(&mut text, "]".into(), "Move to next statistic.".into());
            add_key(&mut text, "p".into(), "Show processes for the selected cgroup.".into());
            add_key(&mut text, "t".into(), "Show threads for the selected cgroup.".into());
            add_key(&mut text, "r".into(), "Refresh the list.".into());
            add_key(&mut text, "h".into(), "Shows this help screen.".into());
            add_key(&mut text, "Esc / q".into(), "Exit the program.".into());

            add_line(&mut text, "".into());
            add_line(&mut text, "Press q, h or Esc to exit help".into());

            // Create the paragraph
            let para = Paragraph::new(text)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .scroll((self.help_scroll, 0));

            // Draw the paragraph
            f.render_widget(para, size);
        })?;

        Ok(())
    }

    /// Key event
    fn key_event(&mut self, key_event: KeyEvent) -> PollResult {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                Some(vec![Action::Scene(AppScene::CGroupTree)])
            }
            KeyCode::Down => self.scroll_help_down(),
            KeyCode::Up => self.scroll_help_up(),
            _ => None,
        }
    }
}
