use super::help::HelpScene;

pub fn build_procs_help_scene<'a>() -> HelpScene<'a> {
    let mut help = HelpScene::new();

    // Create the help text
    help.add_line("Key bindings for process display:");
    help.add_line("");

    help.add_key("Up Arrow", "Move selection up.");
    help.add_key("Down Arrow", "Move selection down.");
    help.add_key("Page Up", "Move selection up a page.");
    help.add_key("Page Down", "Move selection down a page.");
    help.add_key("Home", "Move selection to the top.");
    help.add_key("End", "Move selection to the end.");
    help.add_key("a", "Toggle between processes and threads.");
    help.add_key("c", "Toggle child cgroup processes/threads.");
    help.add_key("n", "Sort by command. Pressing again toggles ascending / descending sort order.");
    help.add_key("s", "Sort by memory usage / PID. Pressing again toggles ascending / descending sort order.");
    help.add_key("[", "Move to previous statistic.");
    help.add_key("]", "Move to next statistic.");
    help.add_key("r", "Refresh the list.");
    help.add_key("h", "Shows this help screen.");
    help.add_key("Esc / q", "Exit the window.");

    help.add_line("");
    help.add_line("Press q, h or Esc to exit help");

    help
}
