use thiserror::Error;

#[derive(Debug, Error)]
pub enum JoineryError {
    #[error("no matching rule for joint: {0}")]
    NoMatchingRule(String),

    #[error("invalid joint specification: {0}")]
    InvalidJoint(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joinery_error_display() {
        let err = JoineryError::NoMatchingRule("DrawerCorner + Bamboo".into());
        assert!(err.to_string().contains("DrawerCorner"));
    }

    #[test]
    fn test_joinery_error_from_core() {
        let core_err = cm_core::Error::ParameterNotFound("joint_type".into());
        let err = JoineryError::from(core_err);
        assert!(matches!(err, JoineryError::Core(_)));
    }
}
