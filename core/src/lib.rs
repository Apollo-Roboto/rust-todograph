mod models;

use std::fs::OpenOptions;
use std::path::Path;

pub use models::*;

const DEVELOPMENT: bool = true;

#[derive(Default, Clone, Debug)]
pub struct TasksManager {
    pub tasks: Vec<MindTask>,
}

impl TasksManager {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        let tasks: Vec<MindTask> = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        // todo check relationships
        // - no loops
        // - if parent doesn't exists, set parent to None
        // - if children doesn't exists, remove it

        Ok(Self { tasks })
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(path)
            .map_err(|e| e.to_string())?;

        match DEVELOPMENT {
            true => serde_json::to_writer_pretty(file, &self.tasks),
            false => serde_json::to_writer(file, &self.tasks),
        }
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Generate a unique id from the current list of tasks
    pub fn generate_id(&self) -> u32 {
        // find the lowest number that's free
        for i in 0..u32::MAX {
            if self.tasks.iter().any(|t| t.id == i) {
                continue;
            }
            return i;
        }
        unreachable!();
    }

    pub fn set_parent(&mut self, task_id: u32, parent_id: u32) {
        // todo check relationships
        // - no loops
        // - if parent doesn't exists, set parent to None
        // - if children doesn't exists, remove it

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.parent = Some(parent_id);
        };

        if let Some(parent) = self.tasks.iter_mut().find(|t| t.id == parent_id) {
            parent.childrens.insert(task_id);
        };
    }

    pub fn remove_parent(&mut self, task_id: u32, parent_id: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.parent = None;
        };

        if let Some(parent) = self.tasks.iter_mut().find(|t| t.id == parent_id) {
            parent.childrens.remove(&task_id);
        };
    }

    /// Calculate the progress of a task, 0.0 to 1.0, None if not found
    /// If todo or doing, calculates from children completion
    /// If done, returns 1.0
    pub fn calc_progress(&self, task_id: u32) -> Option<f32> {
        // navigate all children of a task to find how complete it is

        let task = self.tasks.iter().find(|t| t.id == task_id)?;

        if task.state == MindTaskState::Done {
            return Some(1.0);
        }

        let mut current_percent = 0.;

        let child_count = task.childrens.len() as f32;

        for children_id in &task.childrens {
            let Some(children) = self.tasks.iter().find(|t| t.id == *children_id) else {
                unreachable!("Children should always exists");
            };

            let child_progress = self.calc_progress(children.id).unwrap_or(0.0);
            current_percent += child_progress / child_count;
        }

        Some(current_percent)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_calc_progress() {
        let manager = TasksManager {
            tasks: vec![
                MindTask {
                    id: 0,
                    state: MindTaskState::Doing,
                    parent: None,
                    childrens: HashSet::from([1, 2, 3]),
                    ..Default::default()
                },
                MindTask {
                    id: 1,
                    state: MindTaskState::Todo,
                    parent: Some(0),
                    ..Default::default()
                },
                MindTask {
                    id: 2,
                    state: MindTaskState::Done,
                    parent: Some(0),
                    ..Default::default()
                },
                MindTask {
                    id: 3,
                    state: MindTaskState::Done,
                    parent: Some(0),
                    ..Default::default()
                },
            ],
        };

        assert_eq!(manager.calc_progress(0), Some(0.666_666_7));

        let manager = TasksManager {
            tasks: vec![MindTask {
                id: 0,
                ..Default::default()
            }],
        };

        assert_eq!(manager.calc_progress(0), Some(0.0));
    }
}
