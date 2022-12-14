mod scenes;

use std::io;
use std::path::{Path, PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

use self::scenes::cgroup_tree::CGroupTreeScene;
use self::scenes::cgroup_tree_help::build_cgroup_tree_help_scene;
use self::scenes::help::HelpScene;
use self::scenes::procs::ProcsScene;
use self::scenes::procs_help::build_procs_help_scene;
use self::scenes::stat_choose::StatChooseScene;
use self::scenes::Scene;
use super::TermType;
use crate::cgroup::CGroupSortOrder;
use crate::proc::ProcSortOrder;

type PollResult = Option<Vec<Action>>;

#[derive(PartialEq, Eq)]
pub enum Action {
    Reload,
    Exit,
    Stat(usize),
    Scene(AppScene),
    ProcCGroup(PathBuf),
    ProcMode(bool, bool),
    CGroupSort(CGroupSortOrder),
    ProcSort(ProcSortOrder),
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
    cgroup_tree_help_scene: Box<HelpScene<'a>>,
    stat_choose_scene: Box<StatChooseScene<'a>>,
    procs_scene: Box<ProcsScene<'a>>,
    procs_help_scene: Box<HelpScene<'a>>,
}

impl<'a> App<'a> {
    /// Creates the app
    pub fn new(terminal: &'a mut TermType, cgroup2fs: &'a Path, stat: usize, debug: bool) -> Self {
        let mut res = Self {
            scene: AppScene::CGroupTree,
            terminal,
            reload: true,
            running: true,
            cgroup_tree_scene: Box::new(CGroupTreeScene::new(cgroup2fs, debug)),
            cgroup_tree_help_scene: Box::new(build_cgroup_tree_help_scene()),
            stat_choose_scene: Box::new(StatChooseScene::new()),
            procs_scene: Box::new(ProcsScene::new(cgroup2fs, debug)),
            procs_help_scene: Box::new(build_procs_help_scene()),
        };

        // Set initial statistic
        res.set_stat(stat);

        // Set initial sort order
        res.set_cgroup_sort(CGroupSortOrder::StatDsc);

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
                                MouseEventKind::ScrollDown => scene
                                    .key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
                                MouseEventKind::ScrollUp => {
                                    scene.key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                                }
                                _ => None,
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
                Action::ProcMode(threads, include_children) => {
                    self.set_procs_mode(threads, include_children)
                }
                Action::CGroupSort(sort) => self.set_cgroup_sort(sort),
                Action::ProcSort(sort) => self.set_proc_sort(sort),
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

    fn set_cgroup_sort(&mut self, sort: CGroupSortOrder) {
        self.cgroup_tree_scene.set_sort(sort);
        self.procs_scene.set_cgroup_sort(sort);
    }

    fn set_proc_sort(&mut self, sort: ProcSortOrder) {
        self.cgroup_tree_scene.set_proc_sort(sort);
        self.procs_scene.set_sort(sort);
    }

    fn set_cgroup(&mut self, cgroup: PathBuf) {
        self.procs_scene.set_cgroup(cgroup);
    }

    fn set_procs_mode(&mut self, threads: bool, include_children: bool) {
        self.procs_scene.set_mode(threads, include_children);
    }
}
