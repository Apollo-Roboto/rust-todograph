use std::collections::HashSet;

use crate::{
    Point, TaskGraph,
    commands::{Command, EditorCommandHistory},
};

// TODO: Update the ui from changes within the editor
// with callbacks or channels
// maybe make it reactive? https://github.com/rxRust/rxRust

pub enum EditorEvent {
    CommandSuccess,
    CommandError(String),
    Loading,
    LoadingDone,
    LoadingFailed(String),
    Saving,
    SavingDone,
    SavingFailed(String),
}

#[derive(Default)]
pub struct EditorState {
    pub graph: TaskGraph,
    pub active_task: Option<u32>,
    pub selected_tasks: HashSet<u32>,
    pub pan_zoom: (Point, f32),
}

#[derive(Default)]
pub struct Editor {
    pub history: EditorCommandHistory,
    pub state: EditorState,
}
impl Editor {
    pub fn execute(&mut self, cmd: Box<dyn Command>) -> Result<(), String> {
        self.history.execute(cmd, &mut self.state)
    }

    pub fn undo(&mut self) -> Result<(), String> {
        self.history.undo(&mut self.state)
    }

    pub fn redo(&mut self) -> Result<(), String> {
        self.history.redo(&mut self.state)
    }
}
