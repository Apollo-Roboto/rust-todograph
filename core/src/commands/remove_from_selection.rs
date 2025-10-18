use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Remove tasks from selection
#[derive(Default, Debug, Clone)]
pub struct RemoveFromSelectionCommand {
    task_ids: HashSet<u32>,
    _previous_selection: Option<HashSet<u32>>,
}
impl RemoveFromSelectionCommand {
    pub fn new(tasks: HashSet<u32>) -> Self {
        Self {
            task_ids: tasks,
            _previous_selection: None,
        }
    }
}
impl Display for RemoveFromSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Remove {} tasks from selection", self.task_ids.len())
    }
}
impl Command for RemoveFromSelectionCommand {
    fn execute(&mut self, _editor: &mut EditorState) -> Result<(), String> {
        todo!()
    }

    fn undo(&mut self, _editor: &mut EditorState) -> Result<(), String> {
        todo!()
    }
}
