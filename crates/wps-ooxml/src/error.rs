use thiserror::Error;

#[derive(Error, Debug)]
pub enum OoxmlError {
    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("XML parsing/serialization error: {0}")]
    Xml(String),

    #[error("Missing OPC part: {0}")]
    MissingPart(String),

    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupt package: {0}")]
    CorruptPackage(String),
}
