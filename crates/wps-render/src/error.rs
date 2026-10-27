use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RenderError {
    #[error("Font shaping error: {0}")]
    FontShaping(String),

    #[error("DisplayList error: {0}")]
    DisplayList(String),

    #[error("Canvas render error: {0}")]
    Canvas(String),
}
