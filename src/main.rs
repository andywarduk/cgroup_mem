use std::{
    io::{self, BufRead},
    mem,
    time::{Duration, Instant},
    path::PathBuf,
    fs::File,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    style::{Modifier, Style, Color},
    Terminal,
    widgets::{Block, Borders},
    text::{Text, Span, Spans},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

type TermType = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<(), io::Error> {
    // Set up terminal
    match setup_terminal() {
        Ok(mut terminal) => {
            // Run the application
            let mut app = App::new(&mut terminal, true);

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

#[derive(PartialEq, Eq)]
enum PollResult {
    None,
    Redraw,
    Reload,
    Exit,
}

struct App<'a> {
    debug: bool,
    terminal: &'a mut TermType,
    tree_items: Vec<TreeItem<'a>>,
    tree_state: TreeState,
    next_refresh: Instant,
    draws: usize,
    loads: usize,
    sort: SortOrder,
}

impl<'a> App<'a> {
    /// Creates the app
    fn new(terminal: &'a mut TermType, debug: bool) -> Self {
        Self {
            debug,
            terminal,
            tree_items: Vec::new(),
            tree_state: TreeState::default(),
            next_refresh: Instant::now(),
            draws: 0,
            loads: 0,
            sort: SortOrder::NameAsc,
        }
    }

    /// Calculates the time left before the details should be reloaded, None returned if overdue
    fn time_to_refresh(&self) -> Option<Duration> {
        self.next_refresh.checked_duration_since(Instant::now())
    }

    fn build_tree_level(cgroups: Vec<CGroup>) -> Vec<TreeItem<'a>> {
        let mut tree_items = Vec::with_capacity(cgroups.len());

        for mut cg in cgroups {
            let items = mem::take(&mut cg.children);
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

        self.terminal.draw(|f| {
            // Get the size of the frame
            let size = f.size();

            // Build block title
            let mut title = "CGroup Memory".to_string();

            if self.debug {
                title += &format!(" ({} loads, {} draws)", self.loads, self.draws);
            }

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

    /// Polls for events
    fn poll(&mut self) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = if let Some(poll_duration) = self.time_to_refresh() {
                if event::poll(poll_duration)? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            // A key was pressed
                            match key_event.code {
                                KeyCode::Char('q') => PollResult::Exit,
                                KeyCode::Char('\n' | ' ') => { self.tree_state.toggle_selected(); PollResult::Redraw }
                                KeyCode::Left => { self.tree_state.key_left(); PollResult::Redraw }
                                KeyCode::Right => { self.tree_state.key_right(); PollResult::Redraw }
                                KeyCode::Down => { self.tree_state.key_down(&self.tree_items); PollResult::Redraw }
                                KeyCode::Up => { self.tree_state.key_up(&self.tree_items); PollResult::Redraw }
                                KeyCode::Home => { self.tree_state.select_first(); PollResult::Redraw }
                                KeyCode::End => { self.tree_state.select_last(&self.tree_items); PollResult::Redraw }
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
                                _ => PollResult::None
                            }
                        }
                        Event::Mouse(mouse_event) => {
                            // Mouse event
                            match mouse_event.kind {
                                MouseEventKind::ScrollDown => { self.tree_state.key_down(&self.tree_items); PollResult::Redraw }
                                MouseEventKind::ScrollUp => { self.tree_state.key_up(&self.tree_items); PollResult::Redraw }
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

    /// Main application loop
    fn run(&mut self) -> Result<(), io::Error> {
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
}

#[derive(Clone)]
struct CGroup {
    path: PathBuf,
    error: Option<String>,
    memory: usize,
    children: Vec<CGroup>,
}

impl CGroup {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            error: None,
            memory: 0,
            children: Vec::new(),
        }
    }

    fn new_error(path: PathBuf, msg: String) -> Self {
        Self {
            path,
            error: Some(msg),
            memory: 0,
            children: Vec::new(),
        }
    }
}

impl<'a> From<CGroup> for Text<'a> {
    fn from(cgroup: CGroup) -> Self {
        let pathstr: String = cgroup.path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into();

        let path = Span::styled(pathstr, Style::default().add_modifier(Modifier::BOLD));

        Text::from(Spans::from(match cgroup.error {
            Some(msg) => {
                vec![
                    path,
                    Span::raw(": "),
                    Span::styled(msg, Style::default().fg(Color::Red)),
                ]
            }
            None => {
                let mut spans = format_size(cgroup.memory);
                spans.push(Span::raw(": "));
                spans.push(path);
                spans
            }
        }))
    }
}

#[derive(Clone, Copy)]
enum SortOrder {
    NameAsc,
    NameDsc,
    SizeAsc,
    SizeDsc,
}

fn load_cgroups(sort: SortOrder) -> Vec<CGroup> {
    let mut path_buf = PathBuf::new();
    path_buf.push("/sys/fs/cgroup");

    let root = PathBuf::new();

    match load_cgroup_rec(path_buf, &root, sort) {
        Ok(cgroup) => cgroup.children,
        Err(e) => vec![CGroup::new_error(root, e.to_string()) ]
    }
}

fn load_cgroup_rec(abs_path: PathBuf, rel_path: &PathBuf, sort: SortOrder) -> io::Result<CGroup> {
    let mut cgroup = CGroup::new(rel_path.clone());

    let dir = abs_path.read_dir()?;

    dir.for_each(|file| {
        if let Ok(file) = file {
            let fname = file.file_name();

            if fname == *"memory.current" {
                match get_file_line(&file.path()) {
                    Ok(line) => {
                        match line.parse::<usize>() {
                            Ok(mem) => cgroup.memory = mem,
                            Err(e) => cgroup.error = Some(e.to_string()),
                        }
                    }
                    Err(e) => cgroup.error = Some(e.to_string()),
                }
            }

            if let Ok(ftype) = file.file_type() {
                if ftype.is_dir() {
                    let mut sub_rel_path = rel_path.clone();
                    sub_rel_path.push(fname);

                    match load_cgroup_rec(file.path(), &sub_rel_path, sort) {
                        Ok(sub_cgroup) => cgroup.children.push(sub_cgroup),
                        Err(e) => cgroup.children.push(CGroup::new_error(sub_rel_path, e.to_string()))
                    }
                }
            }
        }
    });

    match sort {
        SortOrder::NameAsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path)),
        SortOrder::NameDsc => cgroup.children.sort_by(|a, b| a.path.cmp(&b.path).reverse()),
        SortOrder::SizeAsc => cgroup.children.sort_by(|a, b| a.memory.cmp(&b.memory)),
        SortOrder::SizeDsc => cgroup.children.sort_by(|a, b| a.memory.cmp(&b.memory).reverse()),
    }

    Ok(cgroup)
}

fn get_file_line(path: &PathBuf) -> io::Result<String> {
    let file = File::open(path)?;

    match io::BufReader::new(file)
        .lines()
        .next()
    {
        None => Err(io::ErrorKind::InvalidData)?,
        Some(Err(e)) => Err(e)?,
        Some(Ok(line)) => Ok(line),
    }
}

const POWERS: [&str; 7] = ["b", "k", "M", "G", "T", "P", "E"];
const STYLES: [Color; 7] = [Color::LightGreen, Color::LightBlue, Color::LightYellow,
    Color::LightRed, Color::LightRed, Color::LightRed, Color::LightRed];

fn format_size(size: usize) -> Vec<Span<'static>> {
    let mut fsize = size as f64;
    let mut power = 0;

    while power < 6 && fsize >= 1024_f64 {
        power += 1;
        fsize /= 1024_f64;
    }

    let style = Style::default().fg(STYLES[power]);

    let dp = if power <= 1 {
        0
    } else {
        4 - (fsize.log10().ceil() as usize)
    };

    vec![Span::styled(format!("{:>5.*} {}", dp, fsize, POWERS[power]), style)]
}