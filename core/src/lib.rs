mod models;

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::path::Path;

pub use models::*;

const DEVELOPMENT: bool = true;

#[derive(Default, Clone, Debug)]
pub struct TaskManager {
    pub tasks: Vec<MindTask>,
    pub active: Option<u32>,
    pub selected: Vec<u32>,
}

impl TaskManager {
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

        Ok(Self {
            tasks,
            active: None,
            selected: Vec::new(),
        })
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
        // - if the current task is in doing, ensure the parent is also in doing

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.parent = Some(parent_id);
        };

        if let Some(parent) = self.tasks.iter_mut().find(|t| t.id == parent_id) {
            parent.childrens.insert(task_id);
        };
    }

    /// Removes link to parent
    pub fn unlink_parent(&mut self, task_id: u32, parent_id: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.parent = None;
        };

        if let Some(parent) = self.tasks.iter_mut().find(|t| t.id == parent_id) {
            parent.childrens.remove(&task_id);
        };
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
        let mut parent_id = None;
        let mut children_ids = HashSet::new();

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            parent_id = task.parent;
            children_ids = task.childrens.clone();
        };

        if let Some(parent_id) = parent_id {
            self.unlink_parent(task_id, parent_id);
        }

        for children_id in children_ids {
            self.unlink_children(task_id, children_id);
        }
    }

    /// Removes a task and break connections acordingly
    pub fn remove_task(&mut self, task_id: u32) {
        self.unlink_all(task_id);

        // remove from task list
        if let Some(i) = self.tasks.iter().position(|t| t.id == task_id) {
            self.tasks.remove(i);
        }
    }

    /// Set the task to doing and set parents to doing acordingly
    pub fn set_task_state(&mut self, task_id: u32, state: MindTaskState) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.state = state;

            // if a children is started, the parent is also started
            if let Some(parent_id) = task.parent
                && (state == MindTaskState::Doing)
            {
                self.set_task_state(parent_id, MindTaskState::Doing);
            }

            // TODO: if a children is completed, the parent is started ONLY if not completed
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

    pub fn select_all(&mut self) {
        self.selected = self.tasks.iter().map(|t| t.id).collect();
        self.active = None;
    }

    pub fn deselect_all(&mut self) {
        self.selected = Vec::new();
        self.active = None;
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

    use super::*;

    #[test]
    fn test_calc_progress() {
        let manager = TaskManager {
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

        let manager = TaskManager {
            tasks: vec![MindTask {
                id: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert_eq!(manager.calc_progress(0), Some(0.0));
    }
}
