use thiserror::Error;

#[derive(Debug, Error)]
pub enum CamError {
    #[error("invalid toolpath: {0}")]
    InvalidToolpath(String),

    #[error("invalid cut parameters: {0}")]
    InvalidCutParams(String),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cam_error_display() {
        let err = CamError::InvalidToolpath("empty segment list".into());
        assert_eq!(err.to_string(), "invalid toolpath: empty segment list");
    }

    #[test]
    fn test_cam_error_from_core() {
        let core_err = cm_core::Error::OutOfRange {
            name: "depth".into(),
            value: 2.0,
            min: 0.0,
            max: 1.0,
        };
        let err = CamError::from(core_err);
        assert!(matches!(err, CamError::Core(_)));
    }
}
