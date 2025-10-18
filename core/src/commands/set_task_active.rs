use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Set a task to active
#[derive(Debug, Clone)]
pub struct SetTaskActiveCommand {
    task_id: u32,
    keep_selection: bool,
    previous_active_task: Option<u32>,
    previous_selection: Option<HashSet<u32>>,
}
impl SetTaskActiveCommand {
    pub fn new(task_id: u32, keep_selection: bool) -> Self {
        Self {
            task_id,
            keep_selection,
            previous_active_task: None,
            previous_selection: None,
        }
    }
}
impl Display for SetTaskActiveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.keep_selection {
            write!(f, "Set task {} active (keep selection)", self.task_id)
        } else {
            write!(f, "Set task {} active", self.task_id)
        }
    }
}
impl Command for SetTaskActiveCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let currently_active = editor.active_task;

        if currently_active.is_some() && currently_active == Some(self.task_id) {
            return Err(String::from("Already active"));
        }
        self.previous_active_task = editor.active_task;

        self.previous_selection = Some(
            editor
                .graph
                .tasks
                .iter()
                .filter(|t| t.selected == true)
                .map(|t| t.id)
                .collect(),
        );

        editor.active_task = Some(self.task_id);

        match self.keep_selection {
            true => {
                if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) {
                    task.selected = true;
                }
            }
            false => {
                editor
                    .graph
                    .tasks
                    .iter_mut()
                    .for_each(|t| t.selected = t.id == self.task_id);
            }
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_selection) = self.previous_selection.as_ref() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = previous_selection.contains(&t.id));
        };

        editor.active_task = self.previous_active_task;

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
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            ..Default::default()
        });

        let mut cmd = SetTaskActiveCommand::new(id2, true);

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();

        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        // check is active
        assert_eq!(state.active_task, Some(id2));

        // check is selected
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id2)
                .is_some_and(|t| t.selected == true)
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
