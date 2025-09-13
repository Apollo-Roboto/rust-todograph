use std::fmt::Display;

use crate::editor::EditorState;

/// Utility macro to import and then reexport each commands, this makes less lines I guess
macro_rules! import_commands {
    ($( $mod:ident :: $command_struct:ident ),* $(,)? ) => {
        $(
            mod $mod;
            pub use $mod::$command_struct;
        )*
    };
}

import_commands! {
    create_task::CreateTaskCommand,
    delete_task::DeleteTaskCommand,
    set_task_parent::SetTaskParentCommand,
    set_task_position::SetTaskPositionCommand,
    set_task_state::SetTaskStateCommand,
    set_task_active::SetTaskActiveCommand,
}

pub trait Command: Display {
    /// Execute the command on the editor, returns an error message if failed
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String>;
    /// Undo the command on the editor, allowing users to undo an action
    /// Returns an error message if failed
    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String>;
}

/// Executed command that lead to an error is not added to the history
pub struct EditorCommandHistory {
    past: Vec<Box<dyn Command>>,
    future: Vec<Box<dyn Command>>,
    limit: usize,
}
impl Default for EditorCommandHistory {
    fn default() -> Self {
        let limit = 20;
        Self {
            past: Vec::with_capacity(limit),
            future: Vec::with_capacity(limit),
            limit,
        }
    }
}
impl EditorCommandHistory {
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
        editor: &mut EditorState,
    ) -> Result<(), String> {
        println!("Executing command {cmd}");

        cmd.execute(editor)?;

        if self.past.len() >= self.limit {
            self.past.remove(0);
        }

        self.past.push(cmd);

        // The future has been forever changed
        self.future.clear();

        Ok(())
    }

    /// Undo the last command
    pub fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(mut cmd) = self.past.pop() else {
            return Ok(());
        };

        println!("Undoing command {cmd}");

        cmd.undo(editor)?;

        // added to the future so it can be redone
        self.future.push(cmd);

        Ok(())
    }

    /// Redo the last undo
    pub fn redo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        let Some(mut cmd) = self.future.pop() else {
            return Ok(());
        };

        println!("Redoing command {cmd}");

        cmd.execute(editor)?;

        if self.past.len() >= self.limit {
            self.past.remove(0);
        }
        self.past.push(cmd);

        Ok(())
    }

    /// Get the limit of the history
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Get the past commands
    pub fn past(&self) -> impl Iterator<Item = &dyn Command> {
        self.past.iter().map(|c| c.as_ref())
    }

    /// Get the future commands
    pub fn future(&self) -> impl Iterator<Item = &dyn Command> {
        self.future.iter().map(|c| c.as_ref())
    }

    /// Get the last command executed (called at undo)
    pub fn last(&self) -> Option<&dyn Command> {
        self.past.last().map(|v| &**v)
    }

    /// Get the next command to be executed (called at redo)
    pub fn next(&self) -> Option<&dyn Command> {
        self.future.first().map(|v| &**v)
    }
}

#[cfg(test)]
mod test {
    use crate::Editor;

    use super::*;

    #[derive(Debug, Default)]
    struct MockCommand;
    impl Display for MockCommand {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock Command")
        }
    }
    impl Command for MockCommand {
        fn execute(&mut self, _editor: &mut EditorState) -> Result<(), String> {
            Ok(())
        }

        fn undo(&mut self, _editor: &mut EditorState) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_history_limit() {
        let mut editor = Editor::default().state;
        let mut history = EditorCommandHistory::new_with_limit(3);

        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        // calls beyond here should discard old commands
        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        history.execute(Box::new(MockCommand), &mut editor).unwrap();
        history.execute(Box::new(MockCommand), &mut editor).unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);
    }

    #[test]
    fn test_history_simple() {
        let mut editor = Editor::default().state;
        let mut history = EditorCommandHistory::default();

        assert_eq!(history.past.len(), 0);
        assert_eq!(history.future.len(), 0);

        history.execute(Box::new(MockCommand), &mut editor).unwrap();

        assert_eq!(history.past.len(), 1);
        assert_eq!(history.future.len(), 0);

        history.execute(Box::new(MockCommand), &mut editor).unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 0);

        history.execute(Box::new(MockCommand), &mut editor).unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);

        history.undo(&mut editor).unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 1);

        history.undo(&mut editor).unwrap();

        assert_eq!(history.past.len(), 1);
        assert_eq!(history.future.len(), 2);

        history.redo(&mut editor).unwrap();

        assert_eq!(history.past.len(), 2);
        assert_eq!(history.future.len(), 1);

        history.execute(Box::new(MockCommand), &mut editor).unwrap();

        assert_eq!(history.past.len(), 3);
        assert_eq!(history.future.len(), 0);
    }
}
