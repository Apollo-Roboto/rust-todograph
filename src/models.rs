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
    pub const ZERO: Point = Point { x: 0., y: 0. };
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

#[derive(Debug, Clone)]
pub struct MindTask {
    /// position on the map
    pub position: Point,
    // pub z_index: i32, // maybe so I can remember what goes in front of what
    /// main name of the task, visible from the quick views
    pub title: String,
    /// current state
    pub state: MindTaskState,
    /// linked parent task if any
    pub parent: Option<Box<MindTask>>,
}

impl MindTask {
    pub fn new(name: String) -> Self {
        Self {
            position: Point::default(),
            title: name,
            state: MindTaskState::default(),
            parent: None,
        }
    }
}
