use std::fmt::Display;

use chrono::Utc;

use crate::commands::Command;
use crate::commands::CreateTaskCommand;
use crate::editor::EditorState;

/// Create a task
#[derive(Debug, Clone)]
pub struct DuplicateTaskCommand {
    original_task_id: u32,
    create_task_command: Option<CreateTaskCommand>,
}
impl DuplicateTaskCommand {
    pub fn new(task_id: u32) -> Self {
        Self {
            original_task_id: task_id,
            create_task_command: None,
        }
    }
}
impl Display for DuplicateTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicating task {}", self.original_task_id)
    }
}
impl Command for DuplicateTaskCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if self.create_task_command.is_none() {
            let Some(mut task) = editor
                .graph
                .tasks
                .iter()
                .find(|t| t.id == self.original_task_id)
                .cloned()
            else {
                return Err(format!("Could not find task id {}", self.original_task_id));
            };
            task.id = editor.graph.generate_id();
            task.creation_date = Utc::now();
            self.create_task_command = Some(CreateTaskCommand::new(task));
        }

        if let Some(ref mut cmd) = self.create_task_command {
            cmd.execute(editor)?;
        }
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(ref mut cmd) = self.create_task_command {
            cmd.undo(editor)?;
        }
        Ok(())
    }
}
