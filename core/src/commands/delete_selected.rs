use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::editor::EditorState;

/// Delete a task
#[derive(Default, Debug, Clone)]
pub struct DeleteSelectedCommand {
    deleted_items: Option<Vec<MindTask>>,
    previously_active_task: Option<Option<u32>>,
}
impl DeleteSelectedCommand {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for DeleteSelectedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete selected")
    }
}
impl Command for DeleteSelectedCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let items_to_delete: Vec<MindTask> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .cloned()
            .collect();

        for item in &items_to_delete {
            if Some(item.id) == editor.active_task {
                self.previously_active_task = Some(Some(item.id));
                editor.active_task = None;
            }
            editor.graph.delete_task(item.id);
        }

        self.deleted_items = Some(items_to_delete);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(items) = &self.deleted_items {
            for item in items {
                editor.graph.create_task(item.clone());
            }
        }

        if let Some(task) = self.previously_active_task {
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
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            selected: true,
            ..Default::default()
        });

        state.active_task = Some(id2);

        let mut cmd = DeleteSelectedCommand::new();

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert_eq!(state.active_task, None);
        assert_eq!(state.graph.tasks.len(), 1);

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
