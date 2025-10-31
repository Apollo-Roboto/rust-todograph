use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Add tasks to selection
#[derive(Debug, Clone)]
pub struct AddToSelectionCommand {
    task_ids: HashSet<u32>,
    _previous_selection: Option<HashSet<u32>>,
}
impl AddToSelectionCommand {
    pub fn new(tasks: HashSet<u32>) -> Self {
        Self {
            task_ids: tasks,
            _previous_selection: None,
        }
    }
}
impl Display for AddToSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add {} items to selection", self.task_ids.len())
    }
}
impl Command for AddToSelectionCommand {
    fn execute(&mut self, _editor: &mut EditorState) -> Result<(), String> {
        todo!()
    }

    fn undo(&mut self, _editor: &mut EditorState) -> Result<(), String> {
        todo!()
    }
}
