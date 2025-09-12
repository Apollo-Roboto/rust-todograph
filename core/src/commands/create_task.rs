use std::fmt::Display;

use crate::commands::Command;
use crate::commands::DeleteTaskCommand;
use crate::{MindTask, TaskGraph};

/// Create a task
#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    task_to_create: MindTask,
}
impl CreateTaskCommand {
    pub fn new(task: MindTask) -> Self {
        Self {
            task_to_create: task,
        }
    }
}
impl Display for CreateTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Create task {}", self.task_to_create.id)
    }
}
impl Command for CreateTaskCommand {
    fn execute(&mut self, manager: &mut TaskGraph) -> Result<(), String> {
        manager.create_task(self.task_to_create.clone());
        Ok(())
    }

    fn undo(&mut self, manager: &mut TaskGraph) -> Result<(), String> {
        let mut cmd = DeleteTaskCommand::new(self.task_to_create.clone());
        cmd.execute(manager)
    }
}
