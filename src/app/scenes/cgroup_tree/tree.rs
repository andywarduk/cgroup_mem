use std::path::{Path, PathBuf};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Block;
use ratatui::Frame;
use tui_tree_widget::{flatten, Tree, TreeItem, TreeState};

use crate::app::PollResult;
use crate::cgroup::stats::{StatType, STATS};
use crate::cgroup::{load_cgroups, CGroup, CGroupSortOrder};
use crate::formatters::{format_mem_qty, format_qty};

#[derive(Default)]
pub struct CGroupTree<'a> {
    cgroups: Vec<CGroup>,
    items: Vec<TreeItem<'a, usize>>,
    state: TreeState<usize>,
    single_root: bool,
    page_size: u16,
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
        let (select, items) =
            self.build_tree_level(&cgroups, stat, &old_selected, &old_opened, vec![]);

        // Save the vectors
        self.cgroups = cgroups;
        self.items = items;

        // Select the new node if any
        if let Some(select) = select {
            self.state.select(select);
        } else {
            self.state.select(vec![]);
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
    ) -> (Option<Vec<usize>>, Vec<TreeItem<'a, usize>>) {
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
            if old_opened.iter().any(|old_path| old_path == path) {
                // Yes - expand it
                self.state.open(next.clone());
            }

            // Process sub nodes
            let (sub_select, sub_nodes) =
                self.build_tree_level(cg.children(), stat, old_selected, old_opened, next);

            if sub_select.is_some() {
                select = sub_select;
            }

            // Push this item
            tree_items.push(TreeItem::new(i, text, sub_nodes).unwrap());
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

        Text::from(Line::from(match cgroup.error() {
            Some(msg) => {
                vec![
                    Span::raw("         "),
                    path,
                    Span::raw(" - "),
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

    pub fn render(&mut self, frame: &mut Frame, block: Block) {
        // Get the size of the frame
        let size = frame.size();

        // Calculate number of rows in a page
        self.page_size = std::cmp::max(2, block.inner(size).height) - 1;

        // Create the tree
        let tree = Tree::new(self.items.clone())
            .unwrap()
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        // Draw the tree
        frame.render_stateful_widget(tree, size, &mut self.state);
    }

    fn move_by(&mut self, amount: isize, no_pos: isize) -> PollResult {
        let visible = flatten(&self.state.get_all_opened(), &self.items);

        if visible.is_empty() {
            return None;
        }

        let current_identifier = self.selected();

        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);

        let new_index = match current_index {
            Some(idx) => idx as isize + amount,
            None => no_pos + amount,
        }
        .max(0)
        .min(visible.len() as isize - 1) as usize;

        if Some(new_index) != current_index {
            let new_identifier = visible[new_index].identifier.clone();
            self.state.select(new_identifier);
            Some(vec![])
        } else {
            None
        }
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
        self.move_by(1, -1)
    }

    #[must_use]
    pub fn up(&mut self) -> PollResult {
        self.move_by(-1, self.page_size as isize + 1)
    }

    #[must_use]
    pub fn pg_down(&mut self) -> PollResult {
        self.move_by(self.page_size as isize, 0)
    }

    #[must_use]
    pub fn pg_up(&mut self) -> PollResult {
        self.move_by(-(self.page_size as isize), self.page_size as isize)
    }

    #[must_use]
    pub fn first(&mut self) -> PollResult {
        self.state.select_first(&self.items);
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
