use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Cabinet validation failed: {0}")]
    CabinetValidation(String),

    #[error("Project validation failed: {0}")]
    ProjectValidation(String),

    #[error("Toolpath validation failed on sheet {sheet}: {message}")]
    ToolpathValidation { sheet: usize, message: String },

    #[error("Nesting error: {0}")]
    Nesting(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
