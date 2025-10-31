use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}
impl std::ops::SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}

impl Point {
    pub const ZERO: Self = Self { x: 0., y: 0. };
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn is_within(&self, p1: Point, p2: Point) -> bool {
        let (min_x, max_x) = if p1.x < p2.x {
            (p1.x, p2.x)
        } else {
            (p2.x, p1.x)
        };
        let (min_y, max_y) = if p1.y < p2.y {
            (p1.y, p2.y)
        } else {
            (p2.y, p1.y)
        };

        self.x >= min_x && self.x <= max_x && self.y >= min_y && self.y <= max_y
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MindTask {
    /// Identifier of this task for relationships
    pub id: u32,
    /// Position on the map
    #[serde(default)]
    pub pos: Point,
    /// Is selected.
    ///
    /// Note: this is not serialized
    #[serde(skip)]
    pub selected: bool,
    /// Main name of the task, visible from the quick views
    pub title: String,
    /// Detailed notes of the task
    #[serde(default)]
    pub notes: String,
    /// Current state
    #[serde(default)]
    pub state: MindTaskState,
    /// When was this item created
    pub creation_date: chrono::DateTime<chrono::Utc>,
    /// When was this item completed
    #[serde(default)]
    pub completion_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Linked parent task if any, parent should contain this task's id as children
    #[serde(default)]
    pub parent: Option<u32>,
    /// Linked children tasks if any, children should contain this tasks's id as parent
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub metadata: SaveDataMetadata,
    pub tasks: Vec<MindTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDataMetadata {
    pub version: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_point_is_within() {
        assert_eq!(
            Point::new(0., 0.).is_within(Point::new(-1., -1.), Point::new(1., 1.)),
            true
        );
        assert_eq!(
            Point::new(0., 0.).is_within(Point::new(-1., 1.), Point::new(1., -1.)),
            true
        );
        assert_eq!(
            Point::new(10., 16.).is_within(Point::new(0., 0.), Point::new(100., 100.)),
            true
        );
        assert_eq!(
            Point::new(0., 0.).is_within(Point::new(-1., -1.), Point::new(-10., -10.)),
            false
        );
        assert_eq!(
            Point::new(-15., 5.).is_within(Point::new(17., 25.), Point::new(18., 30.)),
            false
        );
    }
}
