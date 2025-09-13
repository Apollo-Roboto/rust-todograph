use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::commands::CreateTaskCommand;
use crate::editor::EditorState;

/// Delete a task
#[derive(Debug, Clone)]
pub struct DeleteTaskCommand {
    task_to_delete: MindTask,
}
impl DeleteTaskCommand {
    pub fn new(task: MindTask) -> Self {
        Self {
            task_to_delete: task,
        }
    }
}
impl Display for DeleteTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete task {}", self.task_to_delete.id)
    }
}
impl Command for DeleteTaskCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        editor.graph.delete_task(self.task_to_delete.id);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let mut cmd = CreateTaskCommand::new(self.task_to_delete.clone());
        cmd.execute(editor)
    }
}
