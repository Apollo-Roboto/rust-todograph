use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Set a task to active
#[derive(Default, Debug, Clone)]
pub struct ClearActiveCommand {
    previous_active_task: Option<u32>,
}
impl ClearActiveCommand {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for ClearActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clear active item")
    }
}
impl Command for ClearActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if editor.active_task.is_none() {
            return Err(String::from("There is no active task to clear"));
        }
        self.previous_active_task = editor.active_task;
        editor.active_task = None;

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        editor.active_task = self.previous_active_task;

        Ok(())
    }
}
