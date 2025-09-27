#![allow(unused)]
use std::collections::HashSet;

use crate::{
    Point, TaskGraph,
    commands::{Command, EditorCommandHistory},
};

pub enum EditorEvent {
    /// temporary I guess
    None,
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

pub struct Editor {
    pub history: EditorCommandHistory,
    pub state: EditorState,
    event_callback: Box<dyn Fn(&Editor, EditorEvent) -> () + 'static>,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            history: Default::default(),
            state: Default::default(),
            event_callback: Box::new(|_, _| ()),
        }
    }
}
impl Editor {
    pub fn on_event(&mut self, func: impl Fn(&Editor, EditorEvent) -> () + 'static) {
        self.event_callback = Box::new(func);
    }

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

    fn invoke_on_event(&self, event: EditorEvent) {
        (self.event_callback)(&self, event);
    }
}
