use super::help::HelpScene;

pub fn build_cgroup_tree_help_scene<'a>() -> HelpScene<'a> {
    let mut help = HelpScene::new();

    help.add_line("Key bindings for cgroup memory display:");
    help.add_line("");

    help.add_key("Up Arrow", "Move selection up.");
    help.add_key("Down Arrow", "Move selection down.");
    help.add_key(
        "Left Arrow",
        "Collapse tree node if on a parent node or move to parent otherwise.",
    );
    help.add_key("Right Arrow", "Expand tree node if on a parent node.");
    help.add_key("Home", "Move selection to the top.");
    help.add_key("End", "Move selection to the end.");
    help.add_key(
        "n",
        "Sort by cgroup name. Pressing again toggles ascending / descending sort order.",
    );
    help.add_key(
        "s",
        "Sort by statistic. Pressing again toggles ascending / descending sort order.",
    );
    help.add_key("c", "Collapse all expanded nodes.");
    help.add_key("z", "Select statistic to show.");
    help.add_key("[", "Move to previous statistic.");
    help.add_key("]", "Move to next statistic.");
    help.add_key("p", "Show processes for the selected cgroup.");
    help.add_key(
        "P",
        "Show processes for the selected cgroup and all descendents.",
    );
    help.add_key("t", "Show threads for the selected cgroup.");
    help.add_key(
        "T",
        "Show threads for the selected cgroup and all descendents.",
    );
    help.add_key("r", "Refresh the list.");
    help.add_key("h", "Shows this help screen.");
    help.add_key("Esc / q", "Exit the program.");

    help.add_line("");
    help.add_line("Press q, h or Esc to exit help");

    help
}
