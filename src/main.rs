#[deny(missing_docs)]

mod app;
mod cgroup;
mod file_proc;
mod formatters;
mod proc;

use std::io;

use clap::Parser;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::CrosstermBackend,
    Terminal,
};

use app::App;

type TermType = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<(), io::Error> {
    // Parse command line arguments
    let args = Args::parse();

    // Set up terminal
    match setup_terminal() {
        Ok(mut terminal) => {
            // Run the application
            let mut app = App::new(&mut terminal, &args);

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

fn setup_terminal() -> Result<TermType, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn restore_terminal(terminal: Option<&mut TermType>) -> Result<(), io::Error> {
    disable_raw_mode()?;

    if let Some(terminal) = terminal {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
   /// Enable debug mode
   #[clap(short = 'd', long = "debug", action)]
   debug: bool,
}
