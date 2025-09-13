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
        // TODO: If the previous state is the same, I do not want to keep it on the history
        self.previous_state = Some(editor.graph.set_task_state(self.task_id, self.state_to_set));
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_state) = self.previous_state {
            let mut cmd = SetTaskStateCommand::new(self.task_id, previous_state);
            cmd.execute(editor)
        } else {
            Ok(())
        }
    }
}
