use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Retitle a task
#[derive(Debug, Clone)]
pub struct SetTaskTitleCommand {
    task_id: u32,
    title: String,
    previous_title: Option<String>,
}
impl SetTaskTitleCommand {
    pub fn new(task_id: u32, title: String) -> Self {
        Self {
            task_id,
            title,
            previous_title: None,
        }
    }
}
impl Display for SetTaskTitleCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set task {} title to {}", self.task_id, self.title)
    }
}
impl Command for SetTaskTitleCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) {
            self.previous_title = Some(task.title.clone());
            task.title = self.title.clone();
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(title) = &self.previous_title {
            let mut cmd = SetTaskTitleCommand::new(self.task_id, title.clone());
            cmd.execute(editor)
        } else {
            Ok(())
        }
    }
}
