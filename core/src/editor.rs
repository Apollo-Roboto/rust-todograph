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
        // TODO this should probably not silently fail
        match self.history.execute(cmd, &mut self.state) {
            Ok(_) => {}
            Err(e) => {
                println!("Command execution error: {e}");
            }
        }

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), String> {
        // TODO this should probably not silently fail
        match self.history.undo(&mut self.state) {
            Ok(_) => {}
            Err(e) => {
                println!("Command undo error: {e}");
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), String> {
        // TODO this should probably not silently fail
        match self.history.redo(&mut self.state) {
            Ok(_) => {}
            Err(e) => {
                println!("Command redo error: {e}");
            }
        }

        Ok(())
    }
}
