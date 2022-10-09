mod scenes;

use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

use crate::{cgroup::SortOrder, Args};

use super::TermType;

use self::scenes::{
    cgroup_tree::CGroupTreeScene,
    cgroup_tree_help::CGroupTreeHelpScene,
    procs::ProcsScene,
    procs_help::ProcsHelpScene,
    stat_choose::StatChooseScene,
    Scene,
};

type PollResult = Option<Vec<Action>>;

#[derive(PartialEq, Eq)]
pub enum Action {
    Reload,
    Exit,
    Stat(usize),
    Scene(AppScene),
    ProcCGroup(PathBuf),
    ProcThreads(bool),
    Sort(SortOrder),
}

#[derive(PartialEq, Eq)]
pub enum AppScene {
    CGroupTree,
    CgroupTreeHelp,
    StatChoose,
    Procs,
    ProcsHelp,
}

pub struct App<'a> {
    scene: AppScene,
    terminal: &'a mut TermType,
    reload: bool,
    running: bool,
    cgroup_tree_scene: Box<CGroupTreeScene<'a>>,
    cgroup_tree_help_scene: Box<CGroupTreeHelpScene>,
    stat_choose_scene: Box<StatChooseScene<'a>>,
    procs_scene: Box<ProcsScene<'a>>,
    procs_help_scene: Box<ProcsHelpScene>,
}

impl<'a> App<'a> {
    /// Creates the app
    pub fn new(terminal: &'a mut TermType, args: &Args) -> Self {
        let mut res = Self {
            scene: AppScene::CGroupTree,
            terminal,
            reload: true,
            running: true,
            cgroup_tree_scene: Box::new(CGroupTreeScene::new(args.debug)),
            cgroup_tree_help_scene: Box::new(CGroupTreeHelpScene::new()),
            stat_choose_scene: Box::new(StatChooseScene::new()),
            procs_scene: Box::new(ProcsScene::new(args.debug)),
            procs_help_scene: Box::new(ProcsHelpScene::new()),
        };

        // Set initial statistic
        res.set_stat((args.stat - 1) as usize);

        // Set initial sort order
        res.set_sort(SortOrder::SizeDsc);

        res
    }

    /// Main application loop
    pub fn run(&mut self) -> Result<(), io::Error> {
        while self.running {
            let scene: &mut dyn Scene = match self.scene {
                AppScene::CGroupTree => &mut *self.cgroup_tree_scene,
                AppScene::CgroupTreeHelp => &mut *self.cgroup_tree_help_scene,
                AppScene::StatChoose => &mut *self.stat_choose_scene,
                AppScene::Procs => &mut *self.procs_scene,
                AppScene::ProcsHelp => &mut *self.procs_help_scene,
            };

            if self.reload {
                // Reload the scene
                scene.reload();
                self.reload = false;
            }

            // Draw the scene
            scene.draw(self.terminal)?;

            // Poll events
            let actions = Self::poll(scene)?;

            // Process actions
            self.process_actions(actions);
        }

        Ok(())
    }

    fn poll(scene: &mut dyn Scene) -> Result<Vec<Action>, io::Error> {
        let result = loop {
            let result = if let Some(duration) = scene.time_to_refresh() {
                // Wait for event for timeout period
                if event::poll(duration)? {
                    // Got an event
                    match event::read()? {
                        Event::Key(key_event) => {
                            // A key was pressed
                            scene.key_event(key_event)
                        }
                        Event::Mouse(mouse_event) => {
                            // Mouse event
                            match mouse_event.kind {
                                MouseEventKind::ScrollDown => scene.key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
                                MouseEventKind::ScrollUp => scene.key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
                                _ => PollResult::None,
                            }
                        }
                        Event::Resize(_, _) => {
                            // Break out to redraw
                            Some(vec![])
                        }
                        _ => {
                            // All other events are ignored
                            None
                        }
                    }
                } else {
                    // No event in the timeout period
                    Some(vec![Action::Reload])
                }
            } else {
                // No time left
                Some(vec![Action::Reload])
            };

            if result.is_some() {
                break result;
            }
        };

        Ok(result.unwrap())
    }

    fn process_actions(&mut self, actions: Vec<Action>) {
        for action in actions {
            match action {
                Action::Reload => self.reload = true,
                Action::Exit => self.running = false,
                Action::Scene(scene) => self.set_scene(scene),
                Action::Stat(item) => self.set_stat(item),
                Action::ProcCGroup(cgroup) => self.set_cgroup(cgroup),
                Action::ProcThreads(threads) => self.set_threads(threads),
                Action::Sort(sort) => self.set_sort(sort),
            }
        }
    }

    fn set_scene(&mut self, scene: AppScene) {
        self.scene = scene;
        self.reload = true;
    }

    fn set_stat(&mut self, stat: usize) {
        self.cgroup_tree_scene.set_stat(stat);
        self.stat_choose_scene.set_stat(stat);
        self.procs_scene.set_stat(stat);
    }

    fn set_sort(&mut self, sort: SortOrder) {
        self.cgroup_tree_scene.set_sort(sort);
        self.procs_scene.set_sort(sort);
    }

    fn set_cgroup(&mut self, cgroup: PathBuf) {
        self.procs_scene.set_cgroup(cgroup);
    }

    fn set_threads(&mut self, threads: bool) {
        self.procs_scene.set_threads(threads);
    }
}
