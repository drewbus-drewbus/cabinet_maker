use thiserror::Error;

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("invalid hardware specification: {0}")]
    InvalidSpec(String),

    #[error("incompatible hardware: {0}")]
    Incompatible(String),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_error_display() {
        let err = HardwareError::InvalidSpec("negative cup diameter".into());
        assert!(err.to_string().contains("negative cup diameter"));
    }

    #[test]
    fn test_hardware_error_incompatible() {
        let err = HardwareError::Incompatible("undermount slide requires 1/2\" clearance".into());
        assert!(err.to_string().contains("undermount"));
    }
}
