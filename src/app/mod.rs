mod scenes;

use std::io;

use super::TermType;

use scenes::cgroup_tree::CGroupTreeScene;
use scenes::stat_choose::StatChooseScene;
use scenes::help::HelpScene;
use scenes::Scene;

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
}

#[derive(PartialEq, Eq)]
pub enum AppScene {
    CGroupTree,
    StatChoose,
    Help,
}

pub struct App<'a> {
    scene: AppScene,
    terminal: &'a mut TermType,
    cgroup_tree_scene: Box<CGroupTreeScene<'a>>,
    stat_choose_scene: Box<StatChooseScene<'a>>,
    help_scene: Box<HelpScene>,
}

impl<'a> App<'a> {
    /// Creates the app
    pub fn new(terminal: &'a mut TermType, debug: bool) -> Self {
        Self {
            scene: AppScene::CGroupTree,
            terminal,
            cgroup_tree_scene: Box::new(CGroupTreeScene::new(debug)),
            stat_choose_scene: Box::new(StatChooseScene::new(debug)),
            help_scene: Box::new(HelpScene::new(debug)),
        }
    }

    /// Main application loop
    pub fn run(&'a mut self) -> Result<(), io::Error> {
        let mut reload = true;

        loop {
            let scene: &mut dyn Scene = match self.scene {
                AppScene::CGroupTree => &mut *self.cgroup_tree_scene,
                AppScene::StatChoose => &mut *self.stat_choose_scene,
                AppScene::Help => &mut *self.help_scene,
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
                            SceneChangeParm::Stat(item) => self.cgroup_tree_scene.set_stat(item),
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
