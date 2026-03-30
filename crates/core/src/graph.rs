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
    pub fn set_task_parent(&mut self, task_id: u32, parent_id: u32) -> Option<u32> {
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
    pub fn remove_task_parent(&mut self, task_id: u32) -> Option<u32> {
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
        let Some(task_index) = self.tasks.iter().position(|t| t.id == task_id) else {
            return;
        };

        // remove this id from tasks that depended on it
        self.tasks
            .iter_mut()
            .filter(|t| t.depends_on.contains(&task_id))
            .for_each(|t| {
                t.depends_on.remove(&task_id);
            });

        self.tasks.remove(task_index);
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

    pub fn add_task_dependency(&mut self, task_id: u32, depends_on: u32) -> Result<(), ()> {
        // TODO: properly check that there is no loop, beyond parent/child check

        if task_id == depends_on {
            return Err(());
        }

        let Some(dependency_task) = self.tasks.iter().find(|t| t.id == depends_on) else {
            return Err(());
        };

        // check if dependency is a child, cannot have child task as dependency
        if dependency_task.parent.is_some_and(|id| id == task_id) {
            return Err(());
        }

        let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) else {
            return Err(());
        };

        // check if dependency is a parent, cannot have parent task as dependency
        if task.parent.is_some_and(|id| id == depends_on) {
            return Err(());
        }

        task.depends_on.insert(depends_on);

        Ok(())
    }

    pub fn remove_task_dependency(&mut self, task_id: u32, depends_on: u32) {
        let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) else {
            return;
        };
        task.depends_on.remove(&depends_on);
    }

    pub fn remove_all_task_dependencies(&mut self, task_id: u32) {
        let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) else {
            return;
        };
        task.depends_on = HashSet::new();
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

    pub fn get_all_parent_edges(&self) -> Vec<(Point, Point)> {
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

    pub fn get_all_dependency_edges(&self) -> Vec<(Point, Point)> {
        let mut edges = Vec::new();
        for task in &self.tasks {
            for dependency_id in task.depends_on.iter() {
                let Some(dependency_task) = self.tasks.iter().find(|t| t.id == *dependency_id)
                else {
                    continue;
                };

                edges.push((
                    Point::new(task.pos.x, task.pos.y),
                    Point::new(dependency_task.pos.x, dependency_task.pos.y),
                ));
            }
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

    /// a task is blocked if any of its dependency is not done
    pub fn is_task_blocked(&self, id: u32) -> Option<bool> {
        let Some(task) = self.tasks.iter().find(|t| t.id == id) else {
            return None;
        };
        for dependency_id in task.depends_on.iter() {
            if let Some(dependency) = self.tasks.iter().find(|t| t.id == *dependency_id) {
                if dependency.state != MindTaskState::Done {
                    return Some(true);
                }
            }
        }

        Some(false)
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
    fn test_is_task_blocked() {
        let graph = TaskGraph::from(vec![
            MindTask {
                id: 0,
                state: MindTaskState::Todo,
                ..Default::default()
            },
            MindTask {
                id: 1,
                state: MindTaskState::Done,
                ..Default::default()
            },
            MindTask {
                id: 2,
                state: MindTaskState::Todo,
                depends_on: HashSet::from_iter(vec![0, 1]),
                ..Default::default()
            },
            MindTask {
                id: 3,
                state: MindTaskState::Todo,
                depends_on: HashSet::from_iter(vec![1]),
                ..Default::default()
            },
        ]);

        assert_eq!(graph.is_task_blocked(0), Some(false));
        assert_eq!(graph.is_task_blocked(1), Some(false));
        assert_eq!(graph.is_task_blocked(2), Some(true));
        assert_eq!(graph.is_task_blocked(3), Some(false));
    }

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
