use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Set a task to active
#[derive(Debug, Clone)]
pub struct ClearActiveCommand {
    keep_selected: bool,
    previous_active_task: Option<u32>,
}
impl ClearActiveCommand {
    pub fn new(keep_selected: bool) -> Self {
        Self {
            keep_selected,
            previous_active_task: None,
        }
    }
}
impl Display for ClearActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clear active item")
    }
}
impl Command for ClearActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(task_id) = editor.active_task.take() else {
            return Err(String::from("There is no active task to clear"));
        };

        if !self.keep_selected {
            if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == task_id) {
                task.selected = false;
            }
        }

        self.previous_active_task = Some(task_id);
        editor.active_task = None;

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(task_id) = self.previous_active_task {
            if !self.keep_selected {
                if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.selected = true;
                }
            }
            editor.active_task = Some(task_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undo_redo_do_not_keep_selected() {
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
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            ..Default::default()
        });
        let id4 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id4,
            ..Default::default()
        });

        state.active_task = Some(id1);

        let mut cmd = ClearActiveCommand::new(false);

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

    #[test]
    fn test_undo_redo_do_keep_selected() {
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
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            ..Default::default()
        });
        let id4 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id4,
            ..Default::default()
        });

        state.active_task = Some(id1);

        let mut cmd = ClearActiveCommand::new(true);

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
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id1)
                .is_some_and(|t| t.selected)
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
