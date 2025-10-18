use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Clear selection
#[derive(Default, Debug, Clone)]
pub struct ClearSelectionCommand {
    previous_selection: Option<HashSet<u32>>,
    previous_active_task: Option<Option<u32>>,
}
impl ClearSelectionCommand {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for ClearSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cleared selection")
    }
}
impl Command for ClearSelectionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let previous_selection: HashSet<u32> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .map(|t| t.id)
            .collect();

        if previous_selection.is_empty() {
            return Err(String::from("No selection to clear"));
        }

        self.previous_selection = Some(previous_selection);
        self.previous_active_task = Some(editor.active_task);

        editor
            .graph
            .tasks
            .iter_mut()
            .for_each(|t| t.selected = false);

        editor.active_task = None;

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_selection) = self.previous_selection.as_ref() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = previous_selection.contains(&t.id));
        }

        if let Some(task) = self.previous_active_task {
            editor.active_task = task;
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
            selected: true,
            ..Default::default()
        });

        let mut cmd = ClearSelectionCommand::new();

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
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
