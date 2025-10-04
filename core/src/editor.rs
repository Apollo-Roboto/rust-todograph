#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    path::Path,
};

use serde::Serialize;

use crate::{
    MindTask, Point, SaveData, SaveDataMetadata, TaskGraph,
    commands::{Command, EditorCommandHistory},
};

pub enum EditorEvent {
    /// temporary I guess
    None,
    CommandSuccess,
    CommandFailed(String),
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

    /// Load from a file
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        let data: SaveData = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        self.state.graph = TaskGraph::from(data.tasks);

        Ok(())
    }

    /// Save to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let data = SaveData {
            metadata: SaveDataMetadata {
                version: crate::APPLICATION_VERSION.to_string(),
            },
            tasks: self.state.graph.tasks.clone(),
        };

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(path)
            .map_err(|e| e.to_string())?;

        match crate::APPLICATION_IS_RELEASE {
            false => serde_json::to_writer_pretty(file, &data),
            true => serde_json::to_writer(file, &data),
        }
        .map_err(|e| e.to_string())
    }

    pub fn execute(&mut self, cmd: Box<dyn Command>) -> Result<(), String> {
        // TODO this should probably not silently fail
        match self.history.execute(cmd, &mut self.state) {
            Ok(_) => {
                self.invoke_on_event(EditorEvent::CommandSuccess);
            }
            Err(e) => {
                println!("Command execution error: {e}");
                self.invoke_on_event(EditorEvent::CommandFailed(e));
            }
        }

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), String> {
        // TODO this should probably not silently fail
        match self.history.undo(&mut self.state) {
            Ok(_) => {
                self.invoke_on_event(EditorEvent::CommandSuccess);
            }
            Err(e) => {
                println!("Command undo error: {e}");
                self.invoke_on_event(EditorEvent::CommandFailed(e));
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), String> {
        // TODO this should probably not silently fail
        match self.history.redo(&mut self.state) {
            Ok(_) => {
                self.invoke_on_event(EditorEvent::CommandSuccess);
            }
            Err(e) => {
                println!("Command redo error: {e}");
                self.invoke_on_event(EditorEvent::CommandFailed(e));
            }
        }

        Ok(())
    }

    fn invoke_on_event(&self, event: EditorEvent) {
        (self.event_callback)(&self, event);
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, path::PathBuf};

    use super::*;

    #[test]
    fn test_load() {
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../resources/sample.fwork");

        let mut editor = Editor::default();

        let res = editor.load(file_path);

        res.expect("Failed to load file");

        assert!(!editor.state.graph.tasks.is_empty());
    }

    #[test]
    fn test_save() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut editor = Editor::default();

        let res = editor.save(path);

        let data: SaveData =
            serde_json::from_reader(temp_file).expect("Failed to deserialized saved file");

        res.expect("Failed to save file");
    }
}
