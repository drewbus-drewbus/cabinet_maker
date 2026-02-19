use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostError {
    #[error("machine profile error: {0}")]
    MachineProfile(String),

    #[error("G-code emission error: {0}")]
    Emission(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_error_display() {
        let err = PostError::MachineProfile("unknown controller type".into());
        assert!(err.to_string().contains("unknown controller"));
    }

    #[test]
    fn test_post_error_from_toml() {
        let bad_toml = "{{bad";
        let toml_err = toml::from_str::<toml::Value>(bad_toml).unwrap_err();
        let err = PostError::from(toml_err);
        assert!(matches!(err, PostError::TomlParse(_)));
    }
}
