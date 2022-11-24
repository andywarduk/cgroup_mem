#![deny(missing_docs)]

//! CGroup memory statistics display

mod app;
mod cgroup;
mod file_proc;
mod formatters;
mod proc;

use std::io;

use clap::Parser;
use crossterm::cursor::MoveTo;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    Clear,
    ClearType,
    EnterAlternateScreen,
    LeaveAlternateScreen,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::app::App;
use crate::cgroup::get_cgroup2_mount_point;
use crate::cgroup::stats::STATS;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Enable debug mode
    #[clap(short = 'd', long = "debug", action)]
    debug: bool,

    /// List available statistics
    #[clap(short = 'l', long = "list", action)]
    list_stats: bool,

    /// Initial statistic to display
    #[clap(short = 's', long = "stat", default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=(STATS.len() as i64)))]
    stat: u16,
}

fn main() -> Result<(), io::Error> {
    // Parse command line arguments
    let args = Args::parse();

    if args.list_stats {
        list_stats();
        return Ok(());
    }

    // Try and find path to the cgroup2 mount in /proc/mounts
    let cgroup2fs = match get_cgroup2_mount_point() {
        Some(path) => path,
        None => {
            eprintln!("Unable to find the mount point for the cgroup2 file system");
            std::process::exit(1);
        }
    };

    // Set up terminal
    match setup_terminal() {
        Ok(mut terminal) => {
            // Run the application
            let mut app = App::new(
                &mut terminal,
                &cgroup2fs,
                (args.stat - 1) as usize,
                args.debug,
            );

            let res = app.run();

            // Restore terminal
            restore_terminal(Some(&mut terminal))?;

            res
        }
        Err(e) => {
            restore_terminal(None)?;
            Err(e)
        }
    }
}

type TermType = Terminal<CrosstermBackend<io::Stdout>>;

fn setup_terminal() -> Result<TermType, io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        Clear(ClearType::All)
    )?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn restore_terminal(terminal: Option<&mut TermType>) -> Result<(), io::Error> {
    disable_raw_mode()?;

    if let Some(terminal) = terminal {
        execute!(
            terminal.backend_mut(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        terminal.show_cursor()?;
    }

    Ok(())
}

fn list_stats() {
    println!("Available statistics:");

    for (i, s) in STATS.iter().enumerate() {
        println!("  {:>2}: {}", i + 1, s.desc());
    }
}
