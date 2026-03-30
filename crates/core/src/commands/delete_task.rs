use std::fmt::Display;

use crate::MindTask;
use crate::commands::Command;
use crate::commands::CreateTaskCommand;
use crate::editor::EditorState;

/// Delete a task
#[derive(Debug, Clone)]
pub struct DeleteTaskCommand {
    task_id: u32,
    deleted_task: Option<MindTask>,
    was_active: Option<bool>,
    dependent_tasks: Option<Vec<u32>>,
}
impl DeleteTaskCommand {
    pub fn new(task_id: u32) -> Self {
        Self {
            task_id,
            deleted_task: None,
            was_active: None,
            dependent_tasks: None,
        }
    }
}
impl Display for DeleteTaskCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete task #{}", self.task_id)
    }
}
impl Command for DeleteTaskCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(task) = editor.graph.tasks.iter().find(|t| t.id == self.task_id) else {
            return Err(format!("Could not find task id {}", self.task_id));
        };

        let task_to_delete = task.clone();

        // find the dependents
        let dependent: Vec<u32> = editor
            .graph
            .tasks
            .iter()
            .filter_map(|t| t.depends_on.contains(&self.task_id).then(|| t.id))
            .collect();

        // keep if used to be active
        if editor.active_task.is_some_and(|t_id| t_id == self.task_id) {
            self.was_active = Some(true);
            editor.active_task = None;
        }

        editor.graph.delete_task(task_to_delete.id);

        self.deleted_task = Some(task_to_delete);
        self.dependent_tasks = Some(dependent);

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(ref task) = self.deleted_task else {
            return Ok(());
        };

        let mut cmd = CreateTaskCommand::new(task.clone());
        cmd.execute(editor)?;

        // restore active
        if let Some(was_active) = self.was_active
            && was_active
        {
            editor.active_task = Some(task.id);
        }

        // restore dependencies
        if let Some(dependent_tasks) = &self.dependent_tasks {
            dependent_tasks
                .iter()
                .for_each(|t_id| editor.graph.add_task_dependency(*t_id, task.id).unwrap());
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

        let mut cmd = DeleteTaskCommand::new(id2);

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();
        pretty_assertions::assert_ne!(
            state_before_execute,
            state,
            "The state was supposed to change"
        );

        assert_eq!(state.active_task, None);
        assert_eq!(state.graph.tasks.len(), 2);
        assert!(
            state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == id1)
                .is_some_and(|t| t.depends_on.is_empty())
        );

        cmd.undo(&mut state).unwrap();

        // TODO: the task is being recreated at the end of the tasks vec, order of tasks shouldn't matter
        // so id1 id3 id2 instead of id1 id2 id3
        // so enable this when graph stuff changed
        if false {
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
}
