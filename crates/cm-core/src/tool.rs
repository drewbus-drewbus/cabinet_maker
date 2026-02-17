use serde::{Deserialize, Serialize};

/// A cutting tool in the tool library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool number in the machine's tool table (T1, T2, etc.)
    pub number: u32,

    /// Tool type.
    pub tool_type: ToolType,

    /// Cutting diameter in project units.
    pub diameter: f64,

    /// Number of flutes/cutting edges.
    pub flutes: u32,

    /// Cutting length (max depth of cut) in project units.
    pub cutting_length: f64,

    /// Human-readable description.
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// Flat endmill (upcut, downcut, or compression)
    Endmill,
    /// Ball nose endmill
    BallNose,
    /// V-bit (for V-carving)
    VBit,
    /// Drill bit
    Drill,
    /// Straight/slot cutter (for dados)
    Straight,
}

impl Tool {
    /// A standard 1/4" upcut endmill — the workhorse of CNC woodworking.
    pub fn quarter_inch_endmill() -> Self {
        Self {
            number: 1,
            tool_type: ToolType::Endmill,
            diameter: 0.25,
            flutes: 2,
            cutting_length: 1.0,
            description: "1/4\" 2-flute upcut endmill".into(),
        }
    }

    /// Tool radius (half diameter).
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }

    /// Calculate a basic feed rate for wood.
    ///
    /// Feed = RPM * flutes * chip_load
    ///
    /// Typical chip loads for wood:
    /// - 1/4" endmill: 0.005-0.007" per tooth
    /// - 1/2" endmill: 0.010-0.015" per tooth
    /// - 3/4" endmill: 0.015-0.020" per tooth
    pub fn recommended_feed_rate(&self, rpm: f64) -> f64 {
        let chip_load = match self.tool_type {
            ToolType::Endmill | ToolType::Straight => {
                if self.diameter <= 0.25 {
                    0.005
                } else if self.diameter <= 0.5 {
                    0.012
                } else {
                    0.018
                }
            }
            ToolType::BallNose => 0.005,
            ToolType::VBit => 0.004,
            ToolType::Drill => 0.003,
        };
        rpm * self.flutes as f64 * chip_load
    }

    /// Recommended depth of cut per pass (typically 1x-2x diameter for wood).
    pub fn recommended_depth_per_pass(&self) -> f64 {
        self.diameter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarter_inch_feed_rate() {
        let tool = Tool::quarter_inch_endmill();
        // At 5000 RPM: 5000 * 2 * 0.005 = 50 ipm
        let feed = tool.recommended_feed_rate(5000.0);
        assert!((feed - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_tool_radius() {
        let tool = Tool::quarter_inch_endmill();
        assert!((tool.radius() - 0.125).abs() < 1e-10);
    }
}
