use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

#[derive(Debug, Clone)]
pub struct RemoveTaskDependencyCommand {
    task_id: u32,
    depends_on_id: u32,
}
impl RemoveTaskDependencyCommand {
    pub fn new(task_id: u32, depends_on_id: u32) -> Self {
        Self {
            task_id,
            depends_on_id,
        }
    }
}
impl Display for RemoveTaskDependencyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Removing dependency {} to task {}",
            self.depends_on_id, self.task_id,
        )
    }
}
impl Command for RemoveTaskDependencyCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(task) = editor.graph.tasks.iter().find(|t| t.id == self.task_id) else {
            return Err(format!("task not found"));
        };
        if !task.depends_on.contains(&self.depends_on_id) {
            return Err(format!(
                "task already excludes {} as depdency",
                self.depends_on_id
            ));
        }

        editor
            .graph
            .remove_task_dependency(self.task_id, self.depends_on_id);

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        editor
            .graph
            .add_task_dependency(self.task_id, self.depends_on_id)
            .unwrap();
        Ok(())
    }
}
