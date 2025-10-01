use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Change the description of a task
#[derive(Debug, Clone)]
pub struct SetTaskDescriptionCommand {
    task_id: u32,
    description: String,
    // TODO: is it possible to only keep the diff instead of the whole string that could be very long
    previous_description: Option<String>,
}
impl SetTaskDescriptionCommand {
    pub fn new(task_id: u32, description: String) -> Self {
        Self {
            task_id,
            description,
            previous_description: None,
        }
    }
}
impl Display for SetTaskDescriptionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Changing task {} description ({} characters)",
            self.task_id,
            self.description.len()
        )
    }
}
impl Command for SetTaskDescriptionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) {
            self.previous_description = Some(task.description.clone());
            task.description = self.description.clone();
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(description) = &self.previous_description {
            let mut cmd = SetTaskDescriptionCommand::new(self.task_id, description.clone());
            cmd.execute(editor)
        } else {
            Ok(())
        }
    }
}
