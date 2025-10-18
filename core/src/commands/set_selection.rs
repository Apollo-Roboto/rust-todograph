use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Set the selection
#[derive(Debug, Clone)]
pub struct SetSelectionCommand {
    task_ids: HashSet<u32>,
    previous_selection: Option<HashSet<u32>>,
    previous_active_task: Option<Option<u32>>,
}
impl SetSelectionCommand {
    pub fn new(tasks: HashSet<u32>) -> Self {
        Self {
            task_ids: tasks,
            previous_selection: None,
            previous_active_task: None,
        }
    }
}
impl Display for SetSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Selected {:?} tasks", self.task_ids.len())
    }
}
impl Command for SetSelectionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        // if the active task is outside the selection, it needs to be made not active
        let previous_selection: HashSet<u32> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .map(|t| t.id)
            .collect();

        if previous_selection == self.task_ids {
            return Err(String::from("Selection is identical"));
        }

        self.previous_selection = Some(previous_selection);

        if let Some(active_task) = editor.active_task {
            if !self.task_ids.contains(&active_task) {
                // so the task is outside the new selection, it needs to be made not active
                self.previous_active_task = Some(editor.active_task);
                editor.active_task = None;
            }
        }

        editor
            .graph
            .tasks
            .iter_mut()
            .for_each(|t| t.selected = self.task_ids.contains(&t.id));

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_selection) = self.previous_selection.clone() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = previous_selection.contains(&t.id));
        };
        if let Some(previous_active_task) = self.previous_active_task.clone() {
            editor.active_task = previous_active_task;
        }
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

        state.active_task = Some(id1);

        let mut cmd = SetSelectionCommand::new(HashSet::from_iter(vec![id2]));

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );
        assert_eq!(state.active_task, None);
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
