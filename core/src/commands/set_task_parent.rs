use std::fmt::Display;

use crate::TaskManager;
use crate::commands::Command;

/// Create a task
#[derive(Debug, Clone)]
pub struct SetTaskParentCommand {
    task_id: u32,
    parent_id: Option<u32>,
    previous_parent_id: Option<Option<u32>>,
}
impl SetTaskParentCommand {
    pub fn new(task_id: u32, parent_id: Option<u32>) -> Self {
        Self {
            task_id,
            parent_id,
            previous_parent_id: None,
        }
    }
}
impl Display for SetTaskParentCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Set task {} parent to {}",
            self.task_id,
            self.parent_id
                .map_or(String::from("None"), |id| id.to_string())
        )
    }
}
impl Command for SetTaskParentCommand {
    fn execute(&mut self, manager: &mut TaskManager) -> Result<(), String> {
        match self.parent_id {
            Some(parent_id) => {
                self.previous_parent_id = Some(manager.set_parent(self.task_id, parent_id));
            }
            None => {
                self.previous_parent_id = Some(manager.unlink_parent(self.task_id));
            }
        }
        Ok(())
    }

    fn undo(&mut self, manager: &mut TaskManager) -> Result<(), String> {
        if let Some(parent_id) = self.previous_parent_id {
            let mut cmd = SetTaskParentCommand::new(self.task_id, parent_id);
            cmd.execute(manager)
        } else {
            Ok(())
        }
    }
}
