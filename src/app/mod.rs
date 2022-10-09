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

#[derive(PartialEq, Eq)]
pub enum PollResult {
    None,
    Redraw,
    Reload,
    Exit,
    Scene(AppScene),
    SceneParms(AppScene, Vec<SceneChangeParm>),
}

#[derive(PartialEq, Eq)]
pub enum SceneChangeParm {
    Stat(usize),
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
        let mut reload = true;

        loop {
            let scene: &mut dyn Scene = match self.scene {
                AppScene::CGroupTree => &mut *self.cgroup_tree_scene,
                AppScene::CgroupTreeHelp => &mut *self.cgroup_tree_help_scene,
                AppScene::StatChoose => &mut *self.stat_choose_scene,
                AppScene::Procs => &mut *self.procs_scene,
                AppScene::ProcsHelp => &mut *self.procs_help_scene,
            };

            if reload {
                // Reload the scene
                scene.reload();
                reload = false;
            }

            // Draw the scene
            scene.draw(self.terminal)?;

            // Poll events
            match Self::poll(scene)? {
                PollResult::Exit => break,
                PollResult::Redraw => (),
                PollResult::Reload => reload = true,
                PollResult::Scene(scene) => {
                    self.scene = scene;
                    reload = true
                }
                PollResult::SceneParms(scene, parms) => {
                    for parm in parms {
                        match parm {
                            SceneChangeParm::Stat(item) => self.set_stat(item),
                            SceneChangeParm::ProcCGroup(cgroup) => self.set_cgroup(cgroup),
                            SceneChangeParm::ProcThreads(threads) => self.set_threads(threads),
                            SceneChangeParm::Sort(sort) => self.set_sort(sort),
                        }
                    }
                    self.scene = scene;
                    reload = true
                }
                PollResult::None => unreachable!(),
            }
        }

        Ok(())
    }

    fn poll(scene: &mut dyn Scene) -> Result<PollResult, io::Error> {
        let mut result = PollResult::None;

        while result == PollResult::None {
            result = if let Some(poll_duration) = scene.time_to_refresh() {
                if event::poll(poll_duration)? {
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
