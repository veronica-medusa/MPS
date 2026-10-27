use crate::error::CoreError;

pub trait Command<T>: Send + Sync {
    /// Executes the command on target and returns the exact inverse command
    fn execute(&self, target: &mut T) -> Result<Box<dyn Command<T>>, CoreError>;
    fn name(&self) -> &'static str;
}

pub struct BatchCommand<T> {
    commands: Vec<Box<dyn Command<T>>>,
    name: &'static str,
}

impl<T: 'static> BatchCommand<T> {
    pub fn new(name: &'static str, commands: Vec<Box<dyn Command<T>>>) -> Self {
        Self { commands, name }
    }
}

impl<T: 'static> Command<T> for BatchCommand<T> {
    fn execute(&self, target: &mut T) -> Result<Box<dyn Command<T>>, CoreError> {
        let mut inverse_commands = Vec::with_capacity(self.commands.len());
        for cmd in &self.commands {
            let inv = cmd.execute(target)?;
            inverse_commands.push(inv);
        }
        // Inverses must be executed in reverse order for rollback
        inverse_commands.reverse();
        Ok(Box::new(BatchCommand::new(self.name, inverse_commands)))
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

pub struct CommandHistory<T> {
    undo_stack: Vec<Box<dyn Command<T>>>,
    redo_stack: Vec<Box<dyn Command<T>>>,
    max_history: usize,
    clean_save_index: Option<usize>,
}

impl<T: 'static> CommandHistory<T> {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
            clean_save_index: Some(0),
        }
    }

    pub fn execute(&mut self, cmd: Box<dyn Command<T>>, target: &mut T) -> Result<(), CoreError> {
        let inverse = cmd.execute(target)?;
        self.undo_stack.push(inverse);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
            if let Some(idx) = self.clean_save_index {
                self.clean_save_index = idx.checked_sub(1);
            }
        }
        self.redo_stack.clear();
        Ok(())
    }

    pub fn undo(&mut self, target: &mut T) -> Result<&'static str, CoreError> {
        let inverse_cmd = self.undo_stack.pop().ok_or(CoreError::UndoStackEmpty)?;
        let name = inverse_cmd.name();
        let redo_cmd = inverse_cmd.execute(target)?;
        self.redo_stack.push(redo_cmd);
        Ok(name)
    }

    pub fn redo(&mut self, target: &mut T) -> Result<&'static str, CoreError> {
        let redo_cmd = self.redo_stack.pop().ok_or(CoreError::RedoStackEmpty)?;
        let name = redo_cmd.name();
        let undo_cmd = redo_cmd.execute(target)?;
        self.undo_stack.push(undo_cmd);
        Ok(name)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn mark_clean(&mut self) {
        self.clean_save_index = Some(self.undo_stack.len());
    }

    pub fn is_dirty(&self) -> bool {
        self.clean_save_index != Some(self.undo_stack.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDocument {
        pub text: String,
    }

    struct InsertTextCommand {
        pub pos: usize,
        pub text: String,
    }

    impl Command<MockDocument> for InsertTextCommand {
        fn execute(&self, doc: &mut MockDocument) -> Result<Box<dyn Command<MockDocument>>, CoreError> {
            doc.text.insert_str(self.pos, &self.text);
            Ok(Box::new(DeleteTextCommand {
                pos: self.pos,
                len: self.text.len(),
            }))
        }

        fn name(&self) -> &'static str {
            "InsertText"
        }
    }

    struct DeleteTextCommand {
        pub pos: usize,
        pub len: usize,
    }

    impl Command<MockDocument> for DeleteTextCommand {
        fn execute(&self, doc: &mut MockDocument) -> Result<Box<dyn Command<MockDocument>>, CoreError> {
            let removed: String = doc.text.drain(self.pos..self.pos + self.len).collect();
            Ok(Box::new(InsertTextCommand {
                pos: self.pos,
                text: removed,
            }))
        }

        fn name(&self) -> &'static str {
            "DeleteText"
        }
    }

    #[test]
    fn test_command_undo_redo() {
        let mut doc = MockDocument { text: "Hello".to_string() };
        let mut history = CommandHistory::new(50);

        history.execute(Box::new(InsertTextCommand { pos: 5, text: " World".to_string() }), &mut doc).unwrap();
        assert_eq!(doc.text, "Hello World");
        assert!(history.is_dirty());

        history.undo(&mut doc).unwrap();
        assert_eq!(doc.text, "Hello");

        history.redo(&mut doc).unwrap();
        assert_eq!(doc.text, "Hello World");
    }
}
