use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Block,
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
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
        // Load cgroup information
        self.cgroups = load_cgroups(stat, sort);

        // Build tree items
        self.items = Self::build_tree_level(&self.cgroups, stat);
    }

    fn build_tree_level(cgroups: &Vec<CGroup>, stat: usize) -> Vec<TreeItem<'a>> {
        let mut tree_items = Vec::with_capacity(cgroups.len());

        for cg in cgroups {
            let text: Text = Self::cgroup_text(cg, stat);
            let sub_nodes = Self::build_tree_level(cg.children(), stat);
            tree_items.push(TreeItem::new(text, sub_nodes));
        }

        tree_items
    }

    fn cgroup_text(cgroup: &CGroup, stat: usize) -> Text<'a> {
        let filename = cgroup.path().file_name();

        // Get path as a string
        let pathstr = match filename {
            Some(f) => f.to_string_lossy().clone().into(),
            None => "/".to_string(),
        };

        // Make it bold
        let path = Span::styled(pathstr, Style::default().add_modifier(Modifier::BOLD));

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
