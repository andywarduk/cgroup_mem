use std::io::Stdout;

use tui::{
    text::Text,
    backend::CrosstermBackend,
    Frame, widgets::Block,
    style::{Modifier, Style}
};
use tui_tree_widget::{TreeItem, TreeState, Tree};

use crate::cgroup::{CGroup, SortOrder, load_cgroups};

#[derive(Default)]
pub struct CGroupTree<'a> {
    cgroups: Vec<CGroup>,
    items: Vec<TreeItem<'a>>,
    state: TreeState,
}

impl<'a> CGroupTree<'a> {
    /// Build tree
    pub fn build_tree(&mut self, stat: &str, sort: SortOrder) {
        // Load cgroup information
        self.cgroups = load_cgroups(stat, sort);

        // Build tree items
        self.items = Self::build_tree_level(&self.cgroups);
    }

    fn build_tree_level(cgroups: &Vec<CGroup>) -> Vec<TreeItem<'a>> {
        let mut tree_items = Vec::with_capacity(cgroups.len());

        for cg in cgroups {
            let text: Text = cg.into();
            let sub_nodes = Self::build_tree_level(cg.children());
            tree_items.push(TreeItem::new(text, sub_nodes));
        }

        tree_items
    }

    pub fn render(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, block: Block) {
        // Get the size of the frame
        let size = frame.size();

        // Create the tree
        let tree = Tree::new(self.items.clone())
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        // Draw the tree
        frame.render_stateful_widget(tree, size, &mut self.state);
    }

    pub fn left(&mut self) {
        self.state.key_left();
    }

    pub fn right(&mut self) {
        self.state.key_right();
    }

    pub fn down(&mut self) {
        self.state.key_down(&self.items);
    }

    pub fn up(&mut self) {
        self.state.key_up(&self.items);
    }

    pub fn first(&mut self) {
        self.state.select_first();
    }

    pub fn last(&mut self) {
        self.state.select_last(&self.items);
    }

    pub fn close_all(&mut self) {
        self.state.close_all();
    }

    pub fn selected(&self) -> Vec<usize> {
        self.state.selected()
    }

    pub fn cgroup(&mut self) -> Option<&CGroup> {
        let selected = self.selected();
        let (cgroup, _) = selected
            .iter()
            .fold((None, &self.cgroups), |(_, level), e| {
                (Some(&level[*e]), level[*e].children())
            });

        cgroup
    }
}
