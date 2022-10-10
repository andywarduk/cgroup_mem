use std::{
    io::Stdout,
    path::{Path, PathBuf},
};

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
        CGroupSortOrder,
    },
    formatters::{format_mem_qty, format_qty},
};

#[derive(Default)]
pub struct CGroupTree<'a> {
    cgroups: Vec<CGroup>,
    items: Vec<TreeItem<'a>>,
    state: TreeState,
    single_root: bool,
}

impl<'a> CGroupTree<'a> {
    /// Build tree
    pub fn build_tree(&mut self, cgroup2fs: &Path, stat: usize, sort: CGroupSortOrder) {
        // Save currently selected node path
        let old_selected = self.cgroup().map(|cg| cg.path().clone());

        // Save currently opened node paths
        let old_opened: Vec<PathBuf> = self
            .state
            .get_all_opened()
            .into_iter()
            .filter_map(|sel| self.cgroup_from_selected(sel))
            .map(|cg| cg.path().clone())
            .collect();

        // Close all opened
        self.state.close_all();

        // Load cgroup information
        let cgroups = load_cgroups(cgroup2fs, stat, sort);

        // Build tree items
        let (select, items) = self.build_tree_level(
            &cgroups,
            stat,
            &old_selected,
            &old_opened,
            vec![],
        );

        // Save the vectors
        self.cgroups = cgroups;
        self.items = items;

        // Select the new node if any
        if let Some(select) = select {
            self.state.select(select);
        } else {
            self.state.select(vec!());
        }

        // Expand the root node is we're switching to a view with a single root node
        if self.items.len() == 1 {
            if !self.single_root {
                self.state.open(vec![0]);
                self.single_root = true;
            }
        } else {
            self.single_root = false;
        }
    }

    fn build_tree_level(
        &mut self,
        cgroups: &[CGroup],
        stat: usize,
        old_selected: &Option<PathBuf>,
        old_opened: &Vec<PathBuf>,
        cur_item: Vec<usize>,
    ) -> (Option<Vec<usize>>, Vec<TreeItem<'a>>) {
        let mut select = None;
        let mut tree_items = Vec::new();

        for (i, cg) in cgroups.iter().enumerate() {
            // Build text for this node
            let text: Text = Self::cgroup_text(cg, stat);

            // Add node to the index vector
            let mut next = cur_item.clone();
            next.push(i);

            // Was this path previously selected?
            let path = cg.path();

            if let Some(selected) = old_selected {
                if selected == path {
                    // Yes - select it
                    select = Some(next.clone());
                }
            }

            // Was this path previously expanded?
            if old_opened
                .iter()
                .any(|old_path| old_path == path)
            {
                // Yes - expand it
                self.state.open(next.clone());
            }

            // Process sub nodes
            let (sub_select, sub_nodes) = self.build_tree_level(
                cg.children(),
                stat,
                old_selected,
                old_opened,
                next,
            );

            if sub_select.is_some() {
                select = sub_select;
            }

            // Push this item
            tree_items.push(TreeItem::new(text, sub_nodes));
        }

        (select, tree_items)
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
                let span = match STATS[stat].stat_type() {
                    StatType::MemQtyCumul => format_mem_qty(cgroup.stat()),
                    StatType::Qty => format_qty(cgroup.stat()),
                };
                vec![span, Span::raw(": "), path]
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
