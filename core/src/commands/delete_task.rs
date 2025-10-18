use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::commands::CreateTaskCommand;
use crate::editor::EditorState;

/// Delete a task
#[derive(Debug, Clone)]
pub struct DeleteTaskCommand {
    task_to_delete: MindTask,
    was_active: Option<bool>,
}
impl DeleteTaskCommand {
    pub fn new(task: MindTask) -> Self {
        Self {
            task_to_delete: task,
            was_active: None,
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
        if Some(self.task_to_delete.id) == editor.active_task {
            self.was_active = Some(true);
            editor.active_task = None;
        }
        editor.graph.delete_task(self.task_to_delete.id);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let mut cmd = CreateTaskCommand::new(self.task_to_delete.clone());
        cmd.execute(editor)?;

        if let Some(was_active) = self.was_active
            && was_active
        {
            editor.active_task = Some(self.task_to_delete.id);
        }
        Ok(())
    }
}
