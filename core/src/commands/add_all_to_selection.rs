use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::{Command, SetSelectionCommand};
use crate::editor::EditorState;

/// Clear selection
#[derive(Default, Debug, Clone)]
pub struct AddAllToSelectionCommand {
    cmd: Option<SetSelectionCommand>,
}
impl AddAllToSelectionCommand {
    pub fn new() -> Self {
        Self { cmd: None }
    }
}
impl Display for AddAllToSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Selected all")
    }
}
impl Command for AddAllToSelectionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let tasks: HashSet<u32> = editor.graph.tasks.iter().map(|t| t.id).collect();
        let mut cmd = SetSelectionCommand::new(tasks);
        cmd.execute(editor)?;
        self.cmd = Some(cmd);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(cmd) = &mut self.cmd {
            cmd.undo(editor)?;
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
        let mut cmd = AddAllToSelectionCommand::new();

        let id = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id,
            ..Default::default()
        });
        let id = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id,
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
