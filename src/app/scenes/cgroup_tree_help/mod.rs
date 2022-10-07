use std::io;

use crossterm::event::{self, Event, KeyCode, MouseEventKind};

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{AppScene, PollResult},
    TermType,
};

use super::Scene;

pub struct CGroupTreeHelpScene {
    help_scroll: u16,
}

impl CGroupTreeHelpScene {
    pub fn new(_debug: bool) -> Self {
        Self { help_scroll: 0 }
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
                        KeyCode::Down => self.scroll_help_down(),
                        KeyCode::Up => self.scroll_help_up(),
                        _ => PollResult::None,
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Mouse event
                    match mouse_event.kind {
                        MouseEventKind::ScrollDown => self.scroll_help_down(),
                        MouseEventKind::ScrollUp => self.scroll_help_up(),
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
