use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CoreError {
    #[error("VFS I/O error: {0}")]
    VfsIo(String),

    #[error("File not found at path: {0}")]
    FileNotFound(String),

    #[error("Path already exists: {0}")]
    PathAlreadyExists(String),

    #[error("Invalid unit conversion: {from} cannot be converted to {to}")]
    InvalidUnitConversion {
        from: &'static str,
        to: &'static str,
    },

    #[error("Invalid color format: {0}")]
    InvalidColorFormat(String),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("Undo stack is empty")]
    UndoStackEmpty,

    #[error("Redo stack is empty")]
    RedoStackEmpty,

    #[error("CRDT synchronization error: {0}")]
    CrdtSync(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
