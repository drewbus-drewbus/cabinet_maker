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

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_invalid_dimension() {
        let err = Error::InvalidDimension("width must be positive".into());
        assert_eq!(err.to_string(), "invalid dimension: width must be positive");
    }

    #[test]
    fn test_error_display_out_of_range() {
        let err = Error::OutOfRange {
            name: "rpm".into(),
            value: 30000.0,
            min: 0.0,
            max: 24000.0,
        };
        assert!(err.to_string().contains("30000"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("file not found"));
    }
}
