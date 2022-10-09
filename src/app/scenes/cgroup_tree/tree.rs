use std::{io::Stdout, path::PathBuf};

use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Block,
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app::PollResult,
    cgroup::{
        load_cgroups,
        stats::{StatType, STATS},
        CGroup,
        SortOrder,
    },
    formatters::{format_mem_qty, format_qty},
};

#[derive(Default)]
pub struct CGroupTree<'a> {
    cgroups: Vec<CGroup>,
    items: Vec<TreeItem<'a>>,
    state: TreeState,
}

impl<'a> CGroupTree<'a> {
    /// Build tree
    pub fn build_tree(&mut self, stat: usize, sort: SortOrder) {
        // Save currently selected node path
        let selected = self.cgroup().map(|cg| cg.path().clone());

        // Save currently opened node paths
        let opened: Vec<PathBuf> = self
            .state
            .get_all_opened()
            .into_iter()
            .filter_map(|sel| self.cgroup_from_selected(sel))
            .map(|cg| cg.path().clone())
            .collect();

        // Close all opened
        self.state.close_all();
        self.state.select(vec![]);

        // Load cgroup information
        let cgroups = load_cgroups(stat, sort);

        // Build tree items
        let items = self.build_tree_level(&cgroups, stat, &selected, &opened, vec![]);

        // Save the vectors
        self.cgroups = cgroups;
        self.items = items;
    }

    fn build_tree_level(
        &mut self,
        cgroups: &Vec<CGroup>,
        stat: usize,
        selected: &Option<PathBuf>,
        opened: &Vec<PathBuf>,
        cur: Vec<usize>,
    ) -> Vec<TreeItem<'a>> {
        let mut tree_items = Vec::with_capacity(cgroups.len());

        for (i, cg) in cgroups.iter().enumerate() {
            // Build text for this node
            let text: Text = Self::cgroup_text(cg, stat);

            // Add node to the index vector
            let mut next = cur.clone();
            next.push(i);

            // Was this path previously selected?
            let path = cg.path();

            if let Some(selected) = selected {
                if selected == path {
                    // Yes - select it
                    self.state.select(next.clone())
                }
            }

            // Was this path previously expanded?
            if opened
                .iter()
                .any(|old_path| path == &PathBuf::from("") || old_path == path)
            {
                // Yes - expand it
                self.state.open(next.clone());
            }

            // Process sub nodes
            let sub_nodes = self.build_tree_level(cg.children(), stat, selected, opened, next);

            // Push this item
            tree_items.push(TreeItem::new(text, sub_nodes));
        }

        tree_items
    }

    #[must_use]
    fn cgroup_text(cgroup: &CGroup, stat: usize) -> Text<'a> {
        let filename = cgroup.path().file_name();

        // Get path as a string
        let pathstr = match filename {
            Some(f) => f.to_string_lossy().clone().into(),
            None => "/".to_string(),
        };

        let path = Span::from(pathstr);

        Text::from(Spans::from(match cgroup.error() {
            Some(msg) => {
                vec![
                    path,
                    Span::raw(": "),
                    Span::styled(msg.clone(), Style::default().fg(Color::Red)),
                ]
            }
            None => {
                let mut spans = match STATS[stat].stat_type() {
                    StatType::MemQtyCumul => format_mem_qty(cgroup.stat()),
                    StatType::Qty => format_qty(cgroup.stat()),
                };
                spans.push(Span::raw(": "));
                spans.push(path);
                spans
            }
        }))
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

    #[must_use]
    pub fn left(&mut self) -> PollResult {
        self.state.key_left();
        Some(vec![])
    }

    #[must_use]
    pub fn right(&mut self) -> PollResult {
        self.state.key_right();
        Some(vec![])
    }

    #[must_use]
    pub fn down(&mut self) -> PollResult {
        self.state.key_down(&self.items);
        Some(vec![])
    }

    #[must_use]
    pub fn up(&mut self) -> PollResult {
        self.state.key_up(&self.items);
        Some(vec![])
    }

    #[must_use]
    pub fn first(&mut self) -> PollResult {
        self.state.select_first();
        Some(vec![])
    }

    #[must_use]
    pub fn last(&mut self) -> PollResult {
        self.state.select_last(&self.items);
        Some(vec![])
    }

    #[must_use]
    pub fn close_all(&mut self) -> PollResult {
        self.state.close_all();
        Some(vec![])
    }

    #[must_use]
    pub fn selected(&self) -> Vec<usize> {
        self.state.selected()
    }

    #[must_use]
    pub fn cgroup(&self) -> Option<&CGroup> {
        self.cgroup_from_selected(self.selected())
    }

    #[must_use]
    fn cgroup_from_selected(&self, selected: Vec<usize>) -> Option<&CGroup> {
        let (cgroup, _) = selected
            .iter()
            .fold((None, &self.cgroups), |(_, level), e| {
                (Some(&level[*e]), level[*e].children())
            });

        cgroup
    }
}
