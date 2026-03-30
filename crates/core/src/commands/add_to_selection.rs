use std::collections::HashSet;
use std::fmt::Display;

use crate::commands::Command;
use crate::editor::EditorState;

/// Add tasks to selection
#[derive(Debug, Clone)]
pub struct AddToSelectionCommand {
    task_ids: HashSet<u32>,
    previous_selection: Option<HashSet<u32>>,
}
impl AddToSelectionCommand {
    pub fn new(tasks: HashSet<u32>) -> Self {
        Self {
            task_ids: tasks,
            previous_selection: None,
        }
    }
}
impl Display for AddToSelectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add {} items to selection", self.task_ids.len())
    }
}
impl Command for AddToSelectionCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if self.task_ids.is_empty() {
            return Err(String::from("Nothing to add to the selection"));
        }

        let previous_selection: HashSet<u32> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .map(|t| t.id)
            .collect();

        if self.task_ids.is_subset(&previous_selection) {
            return Err(String::from("Selection won't change"));
        }

        self.previous_selection = Some(previous_selection);

        editor.graph.tasks.iter_mut().for_each(|t| {
            if !t.selected && self.task_ids.contains(&t.id) {
                t.selected = true;
            }
        });

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(previous_selection) = self.previous_selection.clone() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = previous_selection.contains(&t.id));
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::default();

        let id1 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id1,
            selected: true,
            ..Default::default()
        });
        let id2 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            ..Default::default()
        });
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            ..Default::default()
        });
        let id4 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id4,
            ..Default::default()
        });

        state.active_task = Some(id1);

        let selection = HashSet::from_iter(vec![id2, id3]);
        let mut cmd = AddToSelectionCommand::new(selection.clone());

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        assert_eq!(
            state.active_task,
            Some(id1),
            "active task was expected to stay active"
        );
        assert_eq!(state.graph.tasks.iter().filter(|t| t.selected).count(), 3);
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id1)
                .is_some_and(|t| t.selected)
        );
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id4)
                .is_some_and(|t| !t.selected)
        );

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
