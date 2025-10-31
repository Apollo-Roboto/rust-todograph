use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::editor::EditorState;

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
        write!(f, "Create task #{}", self.task_to_create.id)
    }
}
impl Command for CreateTaskCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        editor.graph.create_task(self.task_to_create.clone());
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        editor.graph.delete_task(self.task_to_create.id);
        Ok(())
    }
}
