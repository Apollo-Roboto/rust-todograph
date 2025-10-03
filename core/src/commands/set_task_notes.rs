use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Change the notes of a task
#[derive(Debug, Clone)]
pub struct SetTaskNotesCommand {
    task_id: u32,
    description: String,
    // TODO: is it possible to only keep the diff instead of the whole string that could be very long
    previous_notes: Option<String>,
}
impl SetTaskNotesCommand {
    pub fn new(task_id: u32, description: String) -> Self {
        Self {
            task_id,
            description,
            previous_notes: None,
        }
    }
}
impl Display for SetTaskNotesCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Changing task {} notes ({} characters)",
            self.task_id,
            self.description.len()
        )
    }
}
impl Command for SetTaskNotesCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) {
            self.previous_notes = Some(task.notes.clone());
            task.notes = self.description.clone();
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(description) = &self.previous_notes {
            let mut cmd = SetTaskNotesCommand::new(self.task_id, description.clone());
            cmd.execute(editor)
        } else {
            Ok(())
        }
    }
}
