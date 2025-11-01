use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Remove tasks from selection
#[derive(Default, Debug, Clone)]
pub struct RemoveFromSelectionCommand {
    task_ids: HashSet<u32>,
    previous_selection: Option<HashSet<u32>>,
    previous_active_task: Option<Option<u32>>,
}
impl RemoveFromSelectionCommand {
    pub fn new(tasks: HashSet<u32>) -> Self {
        Self {
            task_ids: tasks,
            previous_selection: None,
            previous_active_task: None,
        }
    }
}
impl Display for RemoveFromSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Remove {} items from selection", self.task_ids.len())
    }
}
impl Command for RemoveFromSelectionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if self.task_ids.is_empty() {
            return Err(String::from("Nothing to remove from the selection"));
        }

        let previous_selection: HashSet<u32> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .map(|t| t.id)
            .collect();

        if self.task_ids.is_disjoint(&previous_selection) {
            return Err(String::from("Selection won't change"));
        }

        self.previous_selection = Some(previous_selection);

        if let Some(active_task) = editor.active_task {
            if self.task_ids.contains(&active_task) {
                // so the task is inside the selection to remove, it needs to be made not active
                self.previous_active_task = Some(editor.active_task);
                editor.active_task = None;
            }
        }

        editor.graph.tasks.iter_mut().for_each(|t| {
            if t.selected && self.task_ids.contains(&t.id) {
                t.selected = false;
            }
        });

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
            selected: true,
            ..Default::default()
        });
        let id2 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            selected: true,
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            selected: true,
            ..Default::default()
        });
        let id4 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id4,
            ..Default::default()
        });

        state.active_task = Some(id1);

        let selection = HashSet::from_iter(vec![id1, id3]);
        let mut cmd = RemoveFromSelectionCommand::new(selection.clone());

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert_eq!(
            state.active_task, None,
            "active task was expected to be cleared"
        );
        assert_eq!(state.graph.tasks.iter().filter(|t| t.selected).count(), 1);
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id1)
                .is_some_and(|t| !t.selected)
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
