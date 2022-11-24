use std::cmp;
use std::io::Stdout;
use std::path::Path;

use tui::backend::CrosstermBackend;
use tui::layout::Constraint;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Cell, Paragraph, Row, Table, TableState};
use tui::Frame;

use crate::app::PollResult;
use crate::cgroup::stats::{ProcStatType, STATS};
use crate::file_proc::FileProcessorError;
use crate::formatters::format_mem_qty;
use crate::proc::{load_procs, Proc, ProcSortOrder};

#[derive(Default)]
pub struct ProcsTable<'a> {
    error: Option<String>,
    procs: Vec<Proc>,
    header: Row<'a>,
    widths: Vec<Constraint>,
    items: Vec<Row<'a>>,
    state: TableState,
    page_size: u16,
}

impl<'a> ProcsTable<'a> {
    /// Build table
    pub fn build_table(
        &mut self,
        cgroup2fs: &Path,
        cgroup: &Path,
        threads: bool,
        include_children: bool,
        stat: usize,
        sort: ProcSortOrder,
    ) {
        // Get currently selected PID
        let old_selected_pid = self.selected().map(|i| self.procs[i].pid);

        // Load process information
        match load_procs(cgroup2fs, cgroup, include_children, threads, stat, sort) {
            Ok(procs) => {
                self.procs = procs;
                self.error = None;
            }
            Err(e) => {
                self.procs = Vec::new();
                self.error = Some(e.to_string());
            }
        }

        // Build table cells
        self.build_table_cells(threads, stat, sort);

        // Re-select PID if we had one and it's still there
        if let Some(old_pid) = old_selected_pid {
            self.state
                .select(self.procs.iter().position(|p| p.pid == old_pid));
        } else {
            self.state.select(None);
        }
    }

    fn build_table_cells(&mut self, threads: bool, stat: usize, sort: ProcSortOrder) {
        let mut header_cells = Vec::new();
        let mut widths = Vec::new();

        // PID/TID column
        let mut text = if threads {
            "TID".to_string()
        } else {
            "PID".to_string()
        };

        match sort {
            ProcSortOrder::PidAsc => text += " ▼",
            ProcSortOrder::PidDsc => text += " ▲",
            _ => (),
        }

        // Calculate max PID length
        let pid_len = cmp::max(
            text.chars().count(),
            self.procs
                .iter()
                .map(|p| format!("{}", p.pid).len())
                .max()
                .unwrap_or(0),
        );

        header_cells.push(Cell::from(format!("{:>1$}", text, pid_len)));
        widths.push(Constraint::Length(pid_len as u16));

        // Stat column
        let mut stat_spans: Vec<Span> = Vec::new();
        let mut stat_len = 0;

        if STATS[stat].proc_stat_type() != ProcStatType::None {
            let mut text: String = STATS[stat].proc_short_desc().into();

            match sort {
                ProcSortOrder::StatAsc => text += " ▼",
                ProcSortOrder::StatDsc => text += " ▲",
                _ => (),
            }

            // Calculate stat spans
            stat_spans = self
                .procs
                .iter()
                .map(|proc| match &proc.stat {
                    Ok(value) => format_mem_qty(*value),
                    Err(e) => {
                        let msg = match e {
                            FileProcessorError::ValueNotFound => "<None>",
                            _ => "<Error>",
                        };
                        Span::styled(msg, Style::default().fg(Color::Red))
                    }
                })
                .collect();

            // Calculate max stat length
            stat_len = cmp::max(
                text.chars().count(),
                stat_spans.iter().map(|s| s.width()).max().unwrap_or(0),
            );

            header_cells.push(Cell::from(format!("{:>1$}", text, stat_len)));
            widths.push(Constraint::Length(cmp::max(7, stat_len as u16)));
        }

        // Command column
        let mut text = "Command".to_string();

        match sort {
            ProcSortOrder::CmdAsc => text += " ▼",
            ProcSortOrder::CmdDsc => text += " ▲",
            _ => (),
        }

        // Calculate max command length
        let cmd_len = cmp::max(
            text.chars().count(),
            self.procs.iter().map(|p| p.cmd.len()).max().unwrap_or(0),
        );

        header_cells.push(Cell::from(text));
        widths.push(Constraint::Length(cmd_len as u16));

        // Build header
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Blue))
            .height(1);

        // Build body
        let body_rows = self
            .procs
            .iter()
            .enumerate()
            .map(|(i, proc)| {
                let mut cells = Vec::new();

                cells.push(Cell::from(format!("{:>1$}", proc.pid, pid_len)));

                if STATS[stat].proc_stat_type() != ProcStatType::None {
                    let span = &stat_spans[i];
                    let pad_len = stat_len - span.width();
                    let mut spans = Vec::new();

                    if pad_len > 0 {
                        spans.push(Span::from(format!("{:>1$}", "", pad_len)))
                    }
                    spans.push(span.clone());

                    cells.push(Cell::from(Spans::from(spans)));
                }

                cells.push(Cell::from(proc.cmd.clone()));

                Row::new(cells)
            })
            .collect();

        self.header = header;
        self.widths = widths;
        self.items = body_rows;
    }

    pub fn render(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, block: Block) {
        // Get the size of the frame
        let size = frame.size();

        // Calculate number of rows in a page
        self.page_size = block.inner(size).height;

        if self.page_size > 0 {
            // Take one off for the heading row
            self.page_size -= 1;
        }

        if let Some(error) = &self.error {
            // Display error message
            let para = Paragraph::new(vec![
                Spans::from(Span::raw("Failed to load processes:")),
                Spans::from(Span::raw(error)),
            ]);

            frame.render_widget(para, size);
        } else {
            // Display process table
            let table = Table::new(self.items.clone())
                .header(self.header.clone())
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .widths(&self.widths);

            // Draw the table
            frame.render_stateful_widget(table, size, &mut self.state);
        }
    }

    #[must_use]
    pub fn up(&mut self) -> PollResult {
        self.move_by(-1)
    }

    #[must_use]
    pub fn down(&mut self) -> PollResult {
        self.move_by(1)
    }

    #[must_use]
    pub fn pgup(&mut self) -> PollResult {
        self.move_by(-(self.page_size as isize))
    }

    #[must_use]
    pub fn pgdown(&mut self) -> PollResult {
        self.move_by(self.page_size as isize)
    }

    #[must_use]
    pub fn home(&mut self) -> PollResult {
        self.move_to(1)
    }

    #[must_use]
    pub fn end(&mut self) -> PollResult {
        self.move_to(-1)
    }

    #[must_use]
    fn move_by(&mut self, amount: isize) -> PollResult {
        if amount == 0 || self.items.is_empty() {
            return None;
        }

        if let Some(cur_row) = self.state.selected() {
            // Have a row selected already - adjust
            let new_row = if amount > 0 {
                // Moving down
                cmp::min(cur_row + amount as usize, self.items.len() - 1)
            } else {
                // Moving up
                let amount = (-amount) as usize;

                if cur_row < amount {
                    0
                } else {
                    cur_row - amount
                }
            };

            if cur_row != new_row {
                self.state.select(Some(new_row));
                Some(vec![])
            } else {
                None
            }
        } else {
            // No row selected yet
            self.move_to(amount)
        }
    }

    #[must_use]
    fn move_to(&mut self, new_row: isize) -> PollResult {
        if self.items.is_empty() {
            return None;
        }

        let new_row = if new_row < 0 {
            let adjust = (-new_row) as usize;

            if adjust > self.items.len() {
                0
            } else {
                self.items.len() - adjust
            }
        } else {
            cmp::min((new_row - 1) as usize, self.items.len() - 1)
        };

        self.state.select(Some(new_row));

        Some(vec![])
    }

    pub fn reset(&mut self) {
        self.state = TableState::default();
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}
