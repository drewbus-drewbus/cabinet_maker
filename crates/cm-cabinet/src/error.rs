use thiserror::Error;

#[derive(Debug, Error)]
pub enum CabinetError {
    #[error("invalid cabinet dimensions: {0}")]
    InvalidDimensions(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("material not found: {0}")]
    MaterialNotFound(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabinet_error_display() {
        let err = CabinetError::InvalidDimensions("width must be positive".into());
        assert_eq!(err.to_string(), "invalid cabinet dimensions: width must be positive");
    }

    #[test]
    fn test_cabinet_error_from_core() {
        let core_err = cm_core::Error::InvalidDimension("negative depth".into());
        let err = CabinetError::from(core_err);
        assert!(matches!(err, CabinetError::Core(_)));
    }

    #[test]
    fn test_cabinet_error_from_toml() {
        let bad_toml = "invalid [[[";
        let toml_err = toml::from_str::<toml::Value>(bad_toml).unwrap_err();
        let err = CabinetError::from(toml_err);
        assert!(matches!(err, CabinetError::TomlParse(_)));
    }
}
