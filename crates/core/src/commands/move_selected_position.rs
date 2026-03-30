use std::collections::HashMap;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;
use crate::{MindTask, Point};

/// Set the position of the selection
#[derive(Debug, Clone)]
pub struct MoveSelectedPositionCommand {
    move_by: Point,
    previous_positions: Option<HashMap<u32, Point>>,
}
impl MoveSelectedPositionCommand {
    pub fn new(by: Point) -> Self {
        Self {
            move_by: by,
            previous_positions: None,
        }
    }
}
impl Display for MoveSelectedPositionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Move selected by {}", self.move_by)
    }
}
impl Command for MoveSelectedPositionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let mut items_to_move: Vec<&mut MindTask> = editor
            .graph
            .tasks
            .iter_mut()
            .filter(|t| t.selected == true)
            .collect();

        let mut previous_positions = HashMap::new();

        for item in &mut items_to_move {
            previous_positions.insert(item.id, item.pos);
            item.pos += self.move_by;
        }

        self.previous_positions = Some(previous_positions);

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(previous_positions) = &self.previous_positions else {
            return Ok(());
        };

        for (id, pos) in previous_positions {
            let Some(task) = editor.graph.tasks.iter_mut().find(|t| t.id == *id) else {
                continue;
            };
            task.pos = *pos;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_undo_redo() {
        let mut state = EditorState::default();

        let id1 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id1,
            pos: Point::new(0., 0.),
            ..Default::default()
        });
        let id2 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            pos: Point::new(0., 0.),
            selected: true,
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            pos: Point::new(-10., -20.),
            selected: true,
            ..Default::default()
        });

        let mut cmd = MoveSelectedPositionCommand::new(Point::new(5., 5.));

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert!(state.graph.tasks.iter().find(|t|t.id == id1).is_some_and(|t|t.pos == Point::new(0., 0.)));
        assert!(state.graph.tasks.iter().find(|t|t.id == id2).is_some_and(|t|t.pos == Point::new(5., 5.)));
        assert!(state.graph.tasks.iter().find(|t|t.id == id3).is_some_and(|t|t.pos == Point::new(-5., -15.)));

        cmd.undo(&mut state).unwrap();
        assert_eq!(
            state_before_execute, state,
            "The state was supposed to be identical to before"
        );
        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );
    }
}
