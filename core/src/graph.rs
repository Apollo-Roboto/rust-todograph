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

impl PartialEq for TaskGraph {
    fn eq(&self, other: &Self) -> bool {
        self.tasks == other.tasks
    }
}
impl TaskGraph {
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

        previous_parent
    }

    /// Removes link to parent
    /// Returns the previous parent id
    pub fn unlink_parent(&mut self, task_id: u32) -> Option<u32> {
        let mut parent_id = None;

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            parent_id = task.parent;
            task.parent = None;
        };

        parent_id
    }

    /// Check if an id already exists
    pub fn does_id_exists(&self, id: u32) -> bool {
        self.tasks.iter().any(|t| t.id == id)
    }

    /// Create a task
    pub fn create_task(&mut self, task: MindTask) {
        self.tasks.push(task);
    }

    /// Removes a task
    pub fn delete_task(&mut self, task_id: u32) {
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

    pub fn iter_children_of_task(&self, parent_id: u32) -> impl Iterator<Item = &MindTask> {
        self.tasks
            .iter()
            .filter(move |t| t.parent.is_some_and(|p| p == parent_id))
    }

    pub fn calc_progress(&self, task_id: u32) -> Option<f32> {
        let task = self.tasks.iter().find(|t| t.id == task_id)?;

        let children: Vec<&MindTask> = self.iter_children_of_task(task_id).collect();

        if children.is_empty() {
            let value = match task.state {
                MindTaskState::Todo => 0.0,
                MindTaskState::Doing => 0.0,
                MindTaskState::Done => 1.0,
            };
            return Some(value);
        }
        let child_count = children.len() as f32;

        let mut current_percent = 0.0;

        for child in children {
            if child.state == MindTaskState::Done {
                current_percent += 1.0 / child_count;
                continue;
            }

            let child_completion = self.calc_progress(child.id).unwrap_or(0.0);
            current_percent += child_completion / child_count;
        }

        Some(current_percent)
    }

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
        todo!();
        count
    }
}

impl From<Vec<MindTask>> for TaskGraph {
    fn from(tasks: Vec<MindTask>) -> Self {
        let mut last_id = 0;
        tasks.iter().for_each(|t| {
            if t.id > last_id {
                last_id = t.id
            }
        });

        Self {
            tasks,
            id_counter: last_id,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_calc_progress_simple() {
        let graph = TaskGraph::from(vec![
            MindTask {
                id: 0,
                state: MindTaskState::Doing,
                parent: None,
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
        ]);

        assert_eq!(graph.calc_progress(0), Some(0.666_666_7));
        assert_eq!(graph.calc_progress(1), Some(0.0));
        assert_eq!(graph.calc_progress(2), Some(1.0));
    }

    #[test]
    fn test_calc_progress_complex() {
        let graph = TaskGraph::from(vec![
            MindTask {
                id: 1,
                state: MindTaskState::Doing,
                ..Default::default()
            },
            MindTask {
                id: 2,
                state: MindTaskState::Done,
                parent: Some(1),
                ..Default::default()
            },
            MindTask {
                id: 3,
                state: MindTaskState::Doing,
                parent: Some(1),
                ..Default::default()
            },
            MindTask {
                id: 4,
                state: MindTaskState::Todo,
                parent: Some(2),
                ..Default::default()
            },
            MindTask {
                id: 5,
                state: MindTaskState::Done,
                parent: Some(2),
                ..Default::default()
            },
        ]);

        assert_eq!(graph.calc_progress(1), Some(0.5));
        assert_eq!(graph.calc_progress(2), Some(0.5));
        assert_eq!(graph.calc_progress(3), Some(0.0));
        assert_eq!(graph.calc_progress(4), Some(0.0));
        assert_eq!(graph.calc_progress(5), Some(1.0));
    }
}
