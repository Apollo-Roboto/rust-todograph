use std::fmt::Display;

use crate::Point;
use crate::commands::Command;
use crate::editor::EditorState;

/// Create a task
#[derive(Debug, Clone)]
pub struct SetTaskPositionCommand {
    task_id: u32,
    pos: Point,
    previous_pos: Option<Point>,
}
impl SetTaskPositionCommand {
    pub fn new(task_id: u32, pos: Point) -> Self {
        Self {
            task_id,
            pos,
            previous_pos: None,
        }
    }
}
impl Display for SetTaskPositionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set task {} position to {}", self.task_id, self.pos)
    }
}
impl Command for SetTaskPositionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == self.task_id) {
            self.previous_pos = Some(task.pos);
            task.pos = self.pos;
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(pos) = self.previous_pos {
            let mut cmd = SetTaskPositionCommand::new(self.task_id, pos);
            cmd.execute(editor)
        } else {
            Ok(())
        }
    }
}
