use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Default for Point {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {
    pub const ZERO: Self = Self { x: 0., y: 0. };
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum MindTaskState {
    #[default]
    Todo = 0,
    Doing = 1,
    Done = 2,
}

impl MindTaskState {
    pub fn next(&self) -> Self {
        match self {
            MindTaskState::Todo => MindTaskState::Doing,
            MindTaskState::Doing => MindTaskState::Done,
            MindTaskState::Done => MindTaskState::Done,
        }
    }
    pub fn previous(&self) -> Self {
        match self {
            MindTaskState::Todo => MindTaskState::Todo,
            MindTaskState::Doing => MindTaskState::Todo,
            MindTaskState::Done => MindTaskState::Doing,
        }
    }
}

impl Display for MindTaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MindTaskState::Todo => "Todo",
                MindTaskState::Doing => "Doing",
                MindTaskState::Done => "Done",
            }
        )
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MindTask {
    /// identifier of this task for relationships
    pub id: u32,
    /// position on the map
    pub pos: Point,
    /// main name of the task, visible from the quick views
    pub title: String,
    /// current state
    pub state: MindTaskState,
    /// when was this item created
    pub creation_date: chrono::DateTime<chrono::Utc>,
    /// when was this item completed
    pub completion_date: Option<chrono::DateTime<chrono::Utc>>,
    /// linked parent task if any, parent should contain this task's id as children
    pub parent: Option<u32>,
    /// linked children tasks if any, children should contain this tasks's id as parent
    pub childrens: HashSet<u32>,
}

impl MindTask {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    pub fn is_leaf(&self) -> bool {
        self.childrens.is_empty()
    }
}
