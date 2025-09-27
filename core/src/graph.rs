#![allow(unused)]
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::path::Path;

use chrono::Utc;

pub use crate::models::*;

// TODO: Experiment with Petgraph

#[derive(Default, Clone, Debug)]
pub struct TaskGraph {
    pub tasks: Vec<MindTask>,
    id_counter: u32,
}

impl TaskGraph {
    /// Load all tasks from a file
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
        // - if the current task is in doing, ensure the parent is also in doing

        // find the last id for the counter
        let mut last_id = 0;
        tasks.iter().for_each(|t| {
            if t.id > last_id {
                last_id = t.id
            }
        });

        Ok(Self {
            tasks,
            id_counter: last_id,
        })
    }

    /// Save all tasks to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(path)
            .map_err(|e| e.to_string())?;

        match crate::DEVELOPMENT {
            true => serde_json::to_writer_pretty(file, &self.tasks),
            false => serde_json::to_writer(file, &self.tasks),
        }
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Generate a unique id
    pub fn generate_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }

    /// Set the parent of a task
    /// Returns the previous parent
    pub fn set_parent(&mut self, task_id: u32, parent_id: u32) -> Option<u32> {
        // todo check relationships
        // - no loops
        // - if parent doesn't exists, set parent to None
        // - if children doesn't exists, remove it
        // - if the current task is in doing, ensure the parent is also in doing

        let mut previous_parent = None;

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            previous_parent = task.parent;
            task.parent = Some(parent_id);
        };

        // remove children from previous parent
        if let Some(previous_parent) = previous_parent
            && previous_parent != parent_id
            && let Some(parent) = self.tasks.iter_mut().find(|t| t.id == previous_parent)
        {
            parent.childrens.remove(&task_id);
        }

        if let Some(parent) = self.tasks.iter_mut().find(|t| t.id == parent_id) {
            parent.childrens.insert(task_id);
        };

        previous_parent
    }

    /// Removes link to parent
    /// Returns the previous parent
    pub fn unlink_parent(&mut self, task_id: u32) -> Option<u32> {
        let mut parent_id = None;

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            parent_id = task.parent;
            task.parent = None;
        };

        if let Some(parent) = self.tasks.iter_mut().find(|t| Some(t.id) == parent_id) {
            parent.childrens.remove(&task_id);
        };

        parent_id
    }

    /// Removes link to childrens
    pub fn unlink_children(&mut self, task_id: u32, children_id: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.childrens.remove(&children_id);
        };

        if let Some(children) = self.tasks.iter_mut().find(|t| t.id == children_id) {
            children.parent = None;
        };
    }

    /// Removes links to parent and childrens
    /// Useful before deleting a task
    pub fn unlink_all(&mut self, task_id: u32) {
        let mut children_ids = HashSet::new();

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            children_ids = task.childrens.clone();
        };

        self.unlink_parent(task_id);

        for children_id in children_ids {
            self.unlink_children(task_id, children_id);
        }
    }

    /// Check if an id already exists
    fn does_id_exists(&self, id: u32) -> bool {
        self.tasks.iter().any(|t| t.id == id)
    }

    /// Create a task and establishes it's relations
    pub fn create_task(&mut self, task: MindTask) {
        if let Some(parent_id) = task.parent {
            self.set_parent(task.id, parent_id);
        }
        for child in task.childrens.iter() {
            self.set_parent(*child, task.id);
        }
        self.tasks.push(task);
    }

    /// Removes a task and break connections acordingly
    pub fn delete_task(&mut self, task_id: u32) {
        self.unlink_all(task_id);

        // remove from task list
        if let Some(task) = self.tasks.iter().position(|t| t.id == task_id) {
            self.tasks.remove(task);
        }
    }

    /// Set the task to doing
    /// Returns the previous state
    pub fn set_task_state(&mut self, task_id: u32, state: MindTaskState) -> MindTaskState {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            let previous_state = task.state;
            task.state = state;
            if let MindTaskState::Done = state {
                task.completion_date = Some(Utc::now());
            } else {
                task.completion_date = None;
            }
            previous_state
        } else {
            state
        }
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

    /// Calculate the progress of all task, 0.0 to 1.0, None if not found
    /// If todo or doing, calculates from children completion
    /// If done, returns 1.0
    pub fn calc_progress_all(&self) -> Option<f32> {
        if self.tasks.is_empty() {
            return None;
        }
        let mut sum_percent = 0.;
        let mut num_root = 0;

        self.tasks.iter().for_each(|t| {
            if t.is_root() {
                num_root += 1;
                sum_percent += self.calc_progress(t.id).unwrap_or(0.);
            }
        });

        if num_root == 0 {
            return None;
        }

        Some(sum_percent / num_root as f32)
    }

    pub fn get_all_edges(&self) -> Vec<(Point, Point)> {
        let mut edges = Vec::new();
        for task in &self.tasks {
            let Some(parent_id) = task.parent else {
                continue;
            };
            let Some(parent) = self.tasks.iter().find(|t| t.id == parent_id) else {
                continue;
            };
            edges.push((
                Point::new(task.pos.x, task.pos.y),
                Point::new(parent.pos.x, parent.pos.y),
            ));
        }

        edges
    }

    pub fn count_root(&self) -> usize {
        let mut count = 0;
        self.tasks.iter().for_each(|t| {
            if t.is_root() {
                count += 1;
            }
        });
        count
    }

    pub fn count_leaf(&self) -> usize {
        let mut count = 0;
        self.tasks.iter().for_each(|t| {
            if t.is_leaf() {
                count += 1;
            }
        });
        count
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_calc_progress() {
        let manager = TaskGraph {
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
            ..Default::default()
        };

        assert_eq!(manager.calc_progress(0), Some(0.666_666_7));

        let manager = TaskGraph {
            tasks: vec![MindTask {
                id: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert_eq!(manager.calc_progress(0), Some(0.0));
    }

    #[test]
    fn test_load_does_not_fails() {
        let file_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../resources/sample_tasks.json");
        let res = TaskGraph::load(file_path);

        res.expect("Failed to load file");
    }
}
