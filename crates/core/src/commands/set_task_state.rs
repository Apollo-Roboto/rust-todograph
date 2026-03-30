use std::fmt::Display;

use crate::MindTaskState;
use crate::commands::Command;
use crate::editor::EditorState;

/// Set the state of a task
#[derive(Debug, Clone)]
pub struct SetTaskStateCommand {
    task_id: u32,
    state_to_set: MindTaskState,
    previous_state: Option<MindTaskState>,
}
impl SetTaskStateCommand {
    pub fn new(task_id: u32, state: MindTaskState) -> Self {
        Self {
            task_id,
            state_to_set: state,
            previous_state: None,
        }
    }
}
impl Display for SetTaskStateCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Set state of task {} to {}",
            self.task_id, self.state_to_set
        )
    }
}
impl Command for SetTaskStateCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(task) = editor.graph.tasks.iter().find(|t| t.id == self.task_id) else {
            return Err(String::from("Could not find task"));
        };
        let current_state = task.state;

        if current_state == self.state_to_set {
            return Err(String::from("State won't change"));
        }
        editor.graph.set_task_state(self.task_id, self.state_to_set);
        self.previous_state = Some(current_state);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_state) = self.previous_state {
            let mut cmd = SetTaskStateCommand::new(self.task_id, previous_state);
            cmd.execute(editor)?;
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
        let mut cmd = SetTaskStateCommand::new(1, MindTaskState::Doing);

        let id1 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id1,
            ..Default::default()
        });
        let id2 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            parent: Some(id1),
            ..Default::default()
        });

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
