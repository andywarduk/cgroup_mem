mod scenes;

use std::io;

use super::TermType;

use scenes::cgroup_tree::CGroupTreeScene;
use scenes::help::HelpScene;
use scenes::Scene;

#[derive(PartialEq, Eq)]
pub enum PollResult {
    None,
    Redraw,
    Reload,
    Exit,
    Scene(AppScene),
}

#[derive(PartialEq, Eq)]
pub enum AppScene {
    CGroupTree,
    Help,
}

pub struct App<'a> {
    scene: AppScene,
    terminal: &'a mut TermType,
    cgroup_tree_scene: Box<CGroupTreeScene<'a>>,
    help_scene: Box<HelpScene>,
}

impl<'a> App<'a> {
    /// Creates the app
    pub fn new(terminal: &'a mut TermType, debug: bool) -> Self {
        Self {
            scene: AppScene::CGroupTree,
            terminal,
            cgroup_tree_scene: Box::new(CGroupTreeScene::new(debug)),
            help_scene: Box::new(HelpScene::new(debug)),
        }
    }

    /// Main application loop
    pub fn run(&'a mut self) -> Result<(), io::Error> {
        let mut reload = true;

        loop {
            let scene: &mut dyn Scene = match self.scene {
                AppScene::CGroupTree => &mut *self.cgroup_tree_scene,
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
                PollResult::Scene(s) => {
                    self.scene = s;
                    reload = true
                }
                PollResult::None => unreachable!(),
            }
        }

        Ok(())
    }
}
