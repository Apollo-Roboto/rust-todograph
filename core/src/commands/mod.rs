use std::fmt::Display;

use crate::TaskManager;

mod create_task;
mod delete_task;
mod set_task_parent;
mod set_task_position;
mod set_task_state;

pub use create_task::CreateTaskCommand;
pub use delete_task::DeleteTaskCommand;
pub use set_task_parent::SetTaskParentCommand;
pub use set_task_position::SetTaskPositionCommand;
pub use set_task_state::SetTaskStateCommand;

pub trait Command: Display {
    /// Execute the command on the manager, returns an error message if failed
    fn execute(&mut self, manager: &mut TaskManager) -> Result<(), String>;
    /// Undo the command on the manager, allowing users to undo an action
    /// Returns an error message if failed
    fn undo(&mut self, manager: &mut TaskManager) -> Result<(), String>;
}

/// Executed command that lead to an error is not added to the history
pub struct TaskCommandHistory {
    past: Vec<Box<dyn Command>>,
    future: Vec<Box<dyn Command>>,
    limit: usize,
}
impl Default for TaskCommandHistory {
    fn default() -> Self {
        let limit = 20;
        Self {
            past: Vec::with_capacity(limit),
            future: Vec::with_capacity(limit),
            limit,
        }
    }
}
impl TaskCommandHistory {
    // Create a new command history with a specific limit
    pub fn new_with_limit(limit: usize) -> Self {
        Self {
            past: Vec::with_capacity(limit),
            future: Vec::with_capacity(limit),
            limit,
        }
    }

    // Empties the history, making undo impossible until an new command is called
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }

    /// Execute a new command
    pub fn execute(
        &mut self,
        mut cmd: Box<dyn Command>,
        manager: &mut TaskManager,
    ) -> Result<(), String> {
        println!("Executing command {cmd}");

        cmd.execute(manager)?;

        if self.past.len() >= self.limit {
            self.past.remove(0);
        }

        self.past.push(cmd);

        // The future has been forever changed
        self.future.clear();

        Ok(())
    }

    /// Undo the last command
    pub fn undo(&mut self, manager: &mut TaskManager) -> Result<(), String> {
        let Some(mut cmd) = self.past.pop() else {
            return Ok(());
        };

        println!("Undoing command {cmd}");

        cmd.undo(manager)?;

        // added to the future so it can be redone
        self.future.push(cmd);

        Ok(())
    }

    /// Redo the last undo
    pub fn redo(&mut self, manager: &mut TaskManager) -> Result<(), String> {
        let Some(mut cmd) = self.future.pop() else {
            return Ok(());
        };

        println!("Redoing command {cmd}");

        cmd.execute(manager)?;

        if self.past.len() >= self.limit {
            self.past.remove(0);
        }
        self.past.push(cmd);

        Ok(())
    }

    /// get the limit of the history
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// get the past commands
    pub fn past(&self) -> impl Iterator<Item = &dyn Command> {
        self.past.iter().map(|c| c.as_ref())
    }

    /// get the future commands
    pub fn future(&self) -> impl Iterator<Item = &dyn Command> {
        self.future.iter().map(|c| c.as_ref())
    }

    /// get the last command executed
    pub fn last(&self) -> Option<&dyn Command> {
        self.past.last().map(|v| &**v)
    }

    /// get the next command to be executed
    pub fn next(&self) -> Option<&dyn Command> {
        self.future.first().map(|v| &**v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default)]
    struct MockCommand;
    impl Display for MockCommand {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock Command")
        }
    }
    impl Command for MockCommand {
        fn execute(&mut self, _manager: &mut TaskManager) -> Result<(), String> {
            Ok(())
        }

        fn undo(&mut self, _manager: &mut TaskManager) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_history_limit() {
        let mut manager = TaskManager::default();
        let mut history = TaskCommandHistory::new_with_limit(3);

        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        // calls beyond here should discard old commands
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();
        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);
    }

    #[test]
    fn test_history_simple() {
        let mut manager = TaskManager::default();
        let mut history = TaskCommandHistory::default();

        assert_eq!(history.past.len(), 0);
        assert_eq!(history.future.len(), 0);

        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();

        assert_eq!(history.past.len(), 1);
        assert_eq!(history.future.len(), 0);

        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 0);

        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);

        history.undo(&mut manager).unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 1);

        history.undo(&mut manager).unwrap();

        assert_eq!(history.past.len(), 1);
        assert_eq!(history.future.len(), 2);

        history.redo(&mut manager).unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 1);

        history
            .execute(Box::new(MockCommand), &mut manager)
            .unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);
    }
}
