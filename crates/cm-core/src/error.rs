use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid dimension: {0}")]
    InvalidDimension(String),

    #[error("parameter not found: {0}")]
    ParameterNotFound(String),

    #[error("value out of range: {name} = {value} (expected {min}..{max})")]
    OutOfRange {
        name: String,
        value: f64,
        min: f64,
        max: f64,
    },
}
