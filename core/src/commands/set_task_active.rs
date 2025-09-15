use std::fmt::Display;

use crate::commands::Command;
use crate::commands::clear_task_active::ClearTaskActiveCommand;
use crate::editor::EditorState;

/// Set a task to active
#[derive(Debug, Clone)]
pub struct SetTaskActiveCommand {
    task_id: u32,
    previous_active_task: Option<u32>,
}
impl SetTaskActiveCommand {
    pub fn new(task_id: u32) -> Self {
        Self {
            task_id,
            previous_active_task: None,
        }
    }
}
impl Display for SetTaskActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set task {} active", self.task_id)
    }
}
impl Command for SetTaskActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let currently_active = editor.active_task;

        if currently_active.is_some() && currently_active == Some(self.task_id) {
            return Err(String::from("Already active"));
        }
        self.previous_active_task = editor.active_task;

        editor.active_task = Some(self.task_id);

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        match self.previous_active_task {
            Some(id) => {
                let mut cmd = SetTaskActiveCommand::new(id);
                cmd.execute(editor)
            }
            None => {
                let mut cmd = ClearTaskActiveCommand::new();
                cmd.execute(editor)
            }
        }
    }
}
