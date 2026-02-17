use std::fmt::Write;

use cm_cam::toolpath::{Motion, Toolpath};
use cm_core::units::Unit;

use crate::machine::MachineProfile;

/// Emits G-code from toolpaths using a machine profile.
pub struct GCodeEmitter<'a> {
    profile: &'a MachineProfile,
    units: Unit,
}

impl<'a> GCodeEmitter<'a> {
    pub fn new(profile: &'a MachineProfile, units: Unit) -> Self {
        Self { profile, units }
    }

    /// Generate a complete G-code program from a list of toolpaths.
    ///
    /// Each toolpath may use a different tool. Tool changes are inserted
    /// when the tool number changes.
    pub fn emit(&self, toolpaths: &[Toolpath]) -> String {
        let mut out = String::with_capacity(4096);
        let dp = self.profile.post.decimal_places as usize;

        // --- Program header ---
        if let Some(ref header) = self.profile.post.program_header {
            writeln!(out, "{header}").unwrap();
        }
        writeln!(out, "(Machine: {})", self.profile.machine.name).unwrap();
        writeln!(out, "{}", self.units.gcode()).unwrap(); // G20 or G21
        writeln!(out, "G90").unwrap(); // Absolute positioning
        writeln!(out, "G17").unwrap(); // XY plane
        writeln!(out, "G40").unwrap(); // Cancel cutter compensation
        writeln!(out, "G49").unwrap(); // Cancel tool length offset
        writeln!(out, "G54").unwrap(); // Work coordinate system 1

        let mut current_tool: Option<u32> = None;
        let mut current_feed: Option<f64> = None;
        let mut last_motion: Option<Motion> = None;

        for tp in toolpaths {
            // Tool change if needed
            if current_tool != Some(tp.tool_number) {
                writeln!(out).unwrap();
                writeln!(out, "(Tool change: T{})", tp.tool_number).unwrap();
                writeln!(out, "M05").unwrap(); // Stop spindle
                writeln!(
                    out,
                    "G00 Z{:.*}",
                    dp, self.profile.post.safe_z
                )
                .unwrap(); // Retract
                writeln!(out, "T{} M06", tp.tool_number).unwrap(); // Tool change
                writeln!(out, "G43 H{}", tp.tool_number).unwrap(); // Tool length offset
                writeln!(out, "S{} M03", tp.rpm as u32).unwrap(); // Spindle on CW
                current_tool = Some(tp.tool_number);
                current_feed = None;
                last_motion = None;
            }

            // Emit each segment
            for seg in &tp.segments {
                match seg.motion {
                    Motion::Rapid => {
                        // G00 - modal, only emit if changed
                        if last_motion != Some(Motion::Rapid) {
                            write!(out, "G00 ").unwrap();
                        }
                        writeln!(
                            out,
                            "X{:.*} Y{:.*} Z{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                        )
                        .unwrap();
                        last_motion = Some(Motion::Rapid);
                    }
                    Motion::Linear => {
                        // Determine if this is a plunge (pure Z move into material)
                        let feed = if seg.z < 0.0 {
                            tp.plunge_rate
                        } else {
                            tp.feed_rate
                        };

                        if last_motion.is_none() || !matches!(last_motion, Some(Motion::Linear)) {
                            write!(out, "G01 ").unwrap();
                        }
                        write!(
                            out,
                            "X{:.*} Y{:.*} Z{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                        )
                        .unwrap();

                        if current_feed != Some(feed) {
                            write!(out, " F{:.*}", dp.min(1), feed).unwrap();
                            current_feed = Some(feed);
                        }
                        writeln!(out).unwrap();
                        last_motion = Some(Motion::Linear);
                    }
                    Motion::ArcCW { i, j } => {
                        write!(
                            out,
                            "G02 X{:.*} Y{:.*} Z{:.*} I{:.*} J{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                            dp, i,
                            dp, j,
                        )
                        .unwrap();
                        if current_feed != Some(tp.feed_rate) {
                            write!(out, " F{:.*}", dp.min(1), tp.feed_rate).unwrap();
                            current_feed = Some(tp.feed_rate);
                        }
                        writeln!(out).unwrap();
                        last_motion = Some(seg.motion);
                    }
                    Motion::ArcCCW { i, j } => {
                        write!(
                            out,
                            "G03 X{:.*} Y{:.*} Z{:.*} I{:.*} J{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                            dp, i,
                            dp, j,
                        )
                        .unwrap();
                        if current_feed != Some(tp.feed_rate) {
                            write!(out, " F{:.*}", dp.min(1), tp.feed_rate).unwrap();
                            current_feed = Some(tp.feed_rate);
                        }
                        writeln!(out).unwrap();
                        last_motion = Some(seg.motion);
                    }
                }
            }
        }

        // --- Program footer ---
        writeln!(out).unwrap();
        writeln!(out, "M05").unwrap(); // Spindle stop
        writeln!(out, "G00 Z{:.*}", dp, self.profile.post.safe_z).unwrap(); // Retract
        writeln!(out, "G00 X0.0000 Y0.0000").unwrap(); // Return to origin
        writeln!(out, "{}", self.profile.post.program_end).unwrap(); // M30
        writeln!(out, "%").unwrap(); // End of program marker

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cam::ops::{generate_profile_cut, CamConfig};
    use cm_core::geometry::Rect;
    use cm_core::tool::Tool;

    #[test]
    fn test_basic_gcode_output() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        let gcode = emitter.emit(&[tp]);

        // Verify header
        assert!(gcode.contains("G20"), "should set inch mode");
        assert!(gcode.contains("G90"), "should set absolute mode");
        assert!(gcode.contains("G17"), "should set XY plane");

        // Verify tool setup
        assert!(gcode.contains("T1 M06"), "should have tool change");
        assert!(gcode.contains("S5000 M03"), "should start spindle");

        // Verify footer
        assert!(gcode.contains("M05"), "should stop spindle");
        assert!(gcode.contains("M30"), "should end program");

        // Verify it contains cutting moves
        assert!(gcode.contains("G01"), "should have linear feed moves");
        assert!(gcode.contains("G00"), "should have rapid moves");
    }

    #[test]
    fn test_gcode_has_no_negative_z_in_rapids() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        let gcode = emitter.emit(&[tp]);

        // Parse G00 lines and ensure Z is never negative
        for line in gcode.lines() {
            if line.starts_with("G00") || (line.starts_with('X') && !gcode[..gcode.find(line).unwrap()].lines().last().map_or(false, |l| l.contains("G01"))) {
                // This is a simplified check - in production we'd parse properly
                if line.contains("G00") && line.contains('Z') {
                    let z_val = extract_z_value(line);
                    if let Some(z) = z_val {
                        assert!(z >= 0.0, "Rapid move has negative Z: {line}");
                    }
                }
            }
        }
    }

    fn extract_z_value(line: &str) -> Option<f64> {
        line.split_whitespace()
            .find(|word| word.starts_with('Z'))
            .and_then(|z_word| z_word[1..].parse::<f64>().ok())
    }
}
