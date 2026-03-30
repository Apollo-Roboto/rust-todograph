use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::editor::EditorState;

/// Delete a task
#[derive(Default, Debug, Clone)]
pub struct DeleteSelectedCommand {
    deleted_items: Option<Vec<MindTask>>,
    previously_active_task: Option<Option<u32>>,
    // Some dependencies will be broken outside of the selection
    dependencies: Option<Vec<(u32, u32)>>,
}
impl DeleteSelectedCommand {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Display for DeleteSelectedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete selected")
    }
}
impl Command for DeleteSelectedCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let items_to_delete: Vec<MindTask> = editor
            .graph
            .tasks
            .iter()
            .filter(|t| t.selected == true)
            .cloned()
            .collect();

        // find all the dependencies
        let mut dependencies = Vec::new();
        for item in &items_to_delete {
            dependencies.append(
                &mut editor
                    .graph
                    .tasks
                    .iter()
                    .filter_map(|t| t.depends_on.contains(&item.id).then(|| (t.id, item.id)))
                    .collect(),
            );
        }
        self.dependencies = Some(dependencies);

        for item in &items_to_delete {
            if Some(item.id) == editor.active_task {
                self.previously_active_task = Some(Some(item.id));
                editor.active_task = None;
            }
            editor.graph.delete_task(item.id);
        }

        self.deleted_items = Some(items_to_delete);
        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(items) = &self.deleted_items {
            for item in items {
                editor.graph.create_task(item.clone());
            }
        }

        if let Some(task) = self.previously_active_task {
            editor.active_task = task;
        }

        if let Some(dependencies) = &self.dependencies {
            dependencies.iter().for_each(|(task_id, depends_on_id)| {
                editor
                    .graph
                    .add_task_dependency(*task_id, *depends_on_id)
                    .unwrap()
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::default();

        let id1 = state.graph.generate_id();
        let id2 = state.graph.generate_id();
        let id3 = state.graph.generate_id();
        state.graph.tasks.push(crate::MindTask {
            id: id1,
            depends_on: HashSet::from_iter(vec![id2]),
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            selected: true,
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            selected: true,
            ..Default::default()
        });

        state.active_task = Some(id2);

        let mut cmd = DeleteSelectedCommand::new();

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        pretty_assertions::assert_ne!(
            state_before_execute,
            state,
            "The state was supposed to change"
        );

        assert_eq!(state.active_task, None);
        assert_eq!(state.graph.tasks.len(), 1);
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id1)
                .is_some_and(|t| t.depends_on.is_empty())
        );

        cmd.undo(&mut state).unwrap();
        pretty_assertions::assert_eq!(
            state_before_execute,
            state,
            "The state was supposed to be identical to before"
        );
        cmd.execute(&mut state).unwrap();
        pretty_assertions::assert_ne!(
            state_before_execute,
            state,
            "The state was supposed to change"
        );
    }
}
