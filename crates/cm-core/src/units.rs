use serde::{Deserialize, Serialize};

/// Unit system for the project. All internal computation uses f64 values
/// interpreted in the project's unit system. Conversion happens at I/O
/// boundaries (reading TOML, writing G-code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Inches,
    Millimeters,
}

impl Unit {
    /// Convert a value from this unit to the other unit.
    pub fn convert_to(self, value: f64, target: Unit) -> f64 {
        match (self, target) {
            (Unit::Inches, Unit::Millimeters) => value * 25.4,
            (Unit::Millimeters, Unit::Inches) => value / 25.4,
            _ => value,
        }
    }

    /// G-code command to set this unit mode.
    pub fn gcode(&self) -> &'static str {
        match self {
            Unit::Inches => "G20",
            Unit::Millimeters => "G21",
        }
    }
}

pub fn inches_to_mm(v: f64) -> f64 {
    v * 25.4
}

pub fn mm_to_inches(v: f64) -> f64 {
    v / 25.4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        let one_inch_in_mm = Unit::Inches.convert_to(1.0, Unit::Millimeters);
        assert!((one_inch_in_mm - 25.4).abs() < 1e-10);

        let round_trip = Unit::Millimeters.convert_to(one_inch_in_mm, Unit::Inches);
        assert!((round_trip - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_same_unit_conversion() {
        assert!((Unit::Inches.convert_to(5.0, Unit::Inches) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_gcode() {
        assert_eq!(Unit::Inches.gcode(), "G20");
        assert_eq!(Unit::Millimeters.gcode(), "G21");
    }
}
