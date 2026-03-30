use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

#[derive(Debug, Clone)]
pub struct RemoveAllTaskDependencyCommand {
    task_id: u32,
    previous_dependencies: Option<HashSet<u32>>,
}
impl RemoveAllTaskDependencyCommand {
    pub fn new(task_id: u32) -> Self {
        Self {
            task_id,
            previous_dependencies: None,
        }
    }
}
impl Display for RemoveAllTaskDependencyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Removing all dependencies from task {}", self.task_id,)
    }
}
impl Command for RemoveAllTaskDependencyCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) else {
            return Err(format!("task not found"));
        };

        if task.depends_on.is_empty() {
            return Err(String::from("dependencies already empty"));
        }

        let mut dependencies = HashSet::new();

        std::mem::swap(&mut dependencies, &mut task.depends_on);

        self.previous_dependencies = Some(dependencies);

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(dependencies) = self.previous_dependencies.as_mut() else {
            return Ok(());
        };

        let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) else {
            return Ok(());
        };

        std::mem::swap(dependencies, &mut task.depends_on);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::default();

        let id1 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id1,
            ..Default::default()
        });
        let id2 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            depends_on: HashSet::from_iter(vec![id1, id2]),
            ..Default::default()
        });

        state.active_task = Some(id2);

        let mut cmd = RemoveAllTaskDependencyCommand::new(id3);

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id3)
                .is_some_and(|t| t.depends_on.is_empty())
        );

        cmd.undo(&mut state).unwrap();
        assert_eq!(
            state_before_execute, state,
            "The state was supposed to be identical to before"
        );
        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );
    }
}
