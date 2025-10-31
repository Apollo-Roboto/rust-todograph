use std::collections::HashSet;
use std::fmt::Display;

use chrono::Utc;

use crate::MindTask;
use crate::commands::Command;
use crate::editor::EditorState;

/// Create a task
#[derive(Debug, Clone)]
pub struct DuplicateSelectedCommand {
    items_to_duplicate: Option<Vec<MindTask>>,
    previously_active: Option<Option<u32>>,
    original_selection: Option<HashSet<u32>>,
}
impl DuplicateSelectedCommand {
    pub fn new() -> Self {
        Self {
            items_to_duplicate: None,
            previously_active: None,
            original_selection: None,
        }
    }
}
impl Display for DuplicateSelectedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate selection")
    }
}
impl Command for DuplicateSelectedCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if self.items_to_duplicate.is_none() {
            let mut items_to_duplicate: Vec<&mut MindTask> = editor
                .graph
                .tasks
                .iter_mut()
                .filter(|t| t.selected == true)
                .collect();

            self.original_selection = Some(items_to_duplicate.iter().map(|t| t.id).collect());

            for item in &mut items_to_duplicate {
                item.selected = false;
                if Some(item.id) == editor.active_task {
                    self.previously_active = Some(editor.active_task);
                    editor.active_task = None;
                }
            }

            let mut cloned_items: Vec<MindTask> = items_to_duplicate
                .iter()
                .by_ref()
                .clone()
                .map(|t| (*t).clone())
                .collect();

            let now = Utc::now();

            for item in &mut cloned_items {
                item.id = editor.graph.generate_id();
                item.creation_date = now;
                item.selected = true;
            }

            self.items_to_duplicate = Some(cloned_items);
        }

        if let Some(items_to_duplicate) = self.items_to_duplicate.as_ref() {
            for item in items_to_duplicate {
                editor.graph.create_task(item.clone());
            }
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(items) = self.items_to_duplicate.as_ref() {
            for item in items {
                editor.graph.delete_task(item.id);
            }
        }
        if let Some(previously_active) = self.previously_active {
            editor.active_task = previously_active;
        }

        if let Some(original_selection) = self.original_selection.as_ref() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = original_selection.contains(&t.id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

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
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            selected: true,
            ..Default::default()
        });

        state.active_task = Some(id2);

        let mut cmd = DuplicateSelectedCommand::new();

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert_eq!(state.active_task, None);
        assert_eq!(state.graph.tasks.len(), 5);

        let selected: HashSet<u32> = state
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected)
            .cloned()
            .map(|t| t.id)
            .collect();

        // Check that the selection is now on the new items
        assert_eq!(selected.len(), 2);
        assert!(!selected.contains(&id1));
        assert!(!selected.contains(&id2));
        assert!(!selected.contains(&id3));

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
