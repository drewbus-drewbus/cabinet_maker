use thiserror::Error;

#[derive(Debug, Error)]
pub enum NestingError {
    #[error("part too large: '{part_id}' ({width:.3}\" x {height:.3}\") exceeds sheet ({sheet_w:.1}\" x {sheet_l:.1}\")")]
    PartTooLarge {
        part_id: String,
        width: f64,
        height: f64,
        sheet_w: f64,
        sheet_l: f64,
    },

    #[error("invalid nesting configuration: {0}")]
    InvalidConfig(String),

    #[error(transparent)]
    Core(#[from] cm_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_error_display_part_too_large() {
        let err = NestingError::PartTooLarge {
            part_id: "left_side".into(),
            width: 50.0,
            height: 30.0,
            sheet_w: 48.0,
            sheet_l: 96.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("left_side"));
        assert!(msg.contains("50.000"));
        assert!(msg.contains("48.0"));
    }

    #[test]
    fn test_nesting_error_invalid_config() {
        let err = NestingError::InvalidConfig("kerf must be positive".into());
        assert!(err.to_string().contains("kerf"));
    }
}
