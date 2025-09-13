use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Select a task
#[derive(Debug, Clone)]
pub struct SetTaskActiveCommand {
    task_id: Option<u32>,
    previous_active_task: Option<u32>,
}
impl SetTaskActiveCommand {
    pub fn new(task_id: Option<u32>) -> Self {
        Self {
            task_id,
            previous_active_task: None,
        }
    }
}
impl Display for SetTaskActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Set task {} active",
            self.task_id
                .map_or(String::from("None"), |id| id.to_string())
        )
    }
}
impl Command for SetTaskActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let currently_active = editor.active_task;
        if currently_active == self.task_id {
            return Err(String::from("Already active"));
        }
        self.previous_active_task = editor.active_task;

        editor.active_task = self.task_id;

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let mut cmd = SetTaskActiveCommand::new(self.previous_active_task);
        cmd.execute(editor)
    }
}
