use std::fmt::Display;

use crate::commands::{Command, SetTaskActiveCommand};
use crate::editor::EditorState;

/// Set a task to active
#[derive(Debug, Clone)]
pub struct ClearTaskActiveCommand {
    previous_active_task: Option<u32>,
}
impl ClearTaskActiveCommand {
    pub fn new() -> Self {
        Self {
            previous_active_task: None,
        }
    }
}
impl Display for ClearTaskActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clear active task")
    }
}
impl Command for ClearTaskActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if editor.active_task.is_none() {
            return Err(String::from("There is no active task to clear"));
        }
        self.previous_active_task = editor.active_task;
        editor.active_task = None;

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        match self.previous_active_task {
            Some(id) => {
                let mut cmd = SetTaskActiveCommand::new(id);
                cmd.execute(editor)
            }
            None => Ok(()),
        }
    }
}
