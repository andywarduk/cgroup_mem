mod scenes;

use std::io;
use std::path::PathBuf;

use crate::cgroup::SortOrder;

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
    NewSort(SortOrder),
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
    pub fn new(terminal: &'a mut TermType, debug: bool) -> Self {
        Self {
            scene: AppScene::CGroupTree,
            terminal,
            cgroup_tree_scene: Box::new(CGroupTreeScene::new(debug)),
            cgroup_tree_help_scene: Box::new(CGroupTreeHelpScene::new(debug)),
            stat_choose_scene: Box::new(StatChooseScene::new(debug)),
            procs_scene: Box::new(ProcsScene::new(debug)),
            procs_help_scene: Box::new(ProcsHelpScene::new(debug)),
        }
    }

    /// Main application loop
    pub fn run(&'a mut self) -> Result<(), io::Error> {
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
                scene.reload();
                reload = false;
            }

            // Draw the scene
            scene.draw(self.terminal)?;

            // Poll events
            match scene.poll()? {
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
                            SceneChangeParm::Stat(item) => {
                                self.cgroup_tree_scene.set_stat(item);
                                self.procs_scene.set_stat(item);
                            }
                            SceneChangeParm::ProcCGroup(cgroup) => {
                                self.procs_scene.set_cgroup(cgroup);
                            }
                            SceneChangeParm::ProcThreads(threads) => {
                                self.procs_scene.set_threads(threads);
                            }
                            SceneChangeParm::NewSort(sort) => {
                                self.cgroup_tree_scene.set_sort(sort);
                                self.procs_scene.set_sort(sort);
                            }
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
}
