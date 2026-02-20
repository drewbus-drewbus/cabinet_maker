use std::fmt::Write;

use cm_cam::toolpath::{Motion, Toolpath};
use cm_core::units::Unit;

use crate::machine::{Controller, MachineProfile};

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
    /// when the tool number changes. The G-code dialect is determined by
    /// the controller type in the machine profile.
    pub fn emit(&self, toolpaths: &[Toolpath]) -> String {
        let mut out = String::with_capacity(4096);
        let dp = self.profile.post.decimal_places as usize;
        let controller = self.profile.machine.controller;
        let mut line_num: u32 = 10;

        // --- Program header ---
        if let Some(ref header) = self.profile.post.program_header {
            self.write_line(&mut out, &mut line_num, header);
        }
        self.write_line(&mut out, &mut line_num, &format!("(Machine: {})", self.profile.machine.name));
        self.write_line(&mut out, &mut line_num, self.units.gcode()); // G20 or G21
        self.write_line(&mut out, &mut line_num, "G90"); // Absolute positioning
        self.write_line(&mut out, &mut line_num, "G17"); // XY plane

        // Controller-specific header codes
        match controller {
            Controller::LinuxCnc => {
                self.write_line(&mut out, &mut line_num, "G40"); // Cancel cutter comp
                self.write_line(&mut out, &mut line_num, "G49"); // Cancel tool length offset
                self.write_line(&mut out, &mut line_num, "G54"); // WCS 1
            }
            Controller::Grbl => {
                // GRBL: no G40, G49, G54 (not supported or not needed)
            }
            Controller::Mach => {
                self.write_line(&mut out, &mut line_num, "G40");
                self.write_line(&mut out, &mut line_num, "G49");
                self.write_line(&mut out, &mut line_num, "G54");
            }
        }

        let mut current_tool: Option<u32> = None;
        let mut current_feed: Option<f64> = None;
        let mut last_motion: Option<Motion> = None;
        let mut in_canned_cycle = false;

        for tp in toolpaths {
            // Tool change if needed
            if current_tool != Some(tp.tool_number) {
                // Cancel any active canned cycle before tool change
                if in_canned_cycle {
                    self.write_line(&mut out, &mut line_num, "G80");
                    in_canned_cycle = false;
                }

                self.write_line(&mut out, &mut line_num, "");
                self.write_line(&mut out, &mut line_num,
                    &format!("(Tool change: T{})", tp.tool_number));
                self.write_line(&mut out, &mut line_num, "M05"); // Stop spindle
                self.write_line(&mut out, &mut line_num,
                    &format!("G00 Z{:.*}", dp, self.profile.post.safe_z)); // Retract

                match controller {
                    Controller::LinuxCnc | Controller::Mach => {
                        self.write_line(&mut out, &mut line_num,
                            &format!("T{} M06", tp.tool_number));
                        self.write_line(&mut out, &mut line_num,
                            &format!("G43 H{}", tp.tool_number));
                    }
                    Controller::Grbl => {
                        // GRBL: no ATC — pause for manual tool change
                        self.write_line(&mut out, &mut line_num,
                            &format!("(Change to tool T{})", tp.tool_number));
                        self.write_line(&mut out, &mut line_num, "M00"); // Program pause
                    }
                }

                self.write_line(&mut out, &mut line_num,
                    &format!("S{} M03", tp.rpm as u32)); // Spindle on CW

                current_tool = Some(tp.tool_number);
                current_feed = None;
                last_motion = None;
            }

            // Emit each segment
            let mut line = String::with_capacity(80);
            for seg in &tp.segments {
                match seg.motion {
                    Motion::Rapid => {
                        // Cancel canned cycle before rapids
                        if in_canned_cycle {
                            self.write_line(&mut out, &mut line_num, "G80");
                            in_canned_cycle = false;
                        }

                        // G00 - modal, only emit if changed
                        line.clear();
                        if last_motion != Some(Motion::Rapid) {
                            write!(line, "G00 ").unwrap();
                        }
                        write!(
                            line,
                            "X{:.*} Y{:.*} Z{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                        )
                        .unwrap();
                        self.write_line(&mut out, &mut line_num, &line);
                        last_motion = Some(Motion::Rapid);
                    }
                    Motion::Linear => {
                        if in_canned_cycle {
                            self.write_line(&mut out, &mut line_num, "G80");
                            in_canned_cycle = false;
                        }

                        // Determine if this is a plunge
                        let feed = if seg.z < 0.0 {
                            tp.plunge_rate
                        } else {
                            tp.feed_rate
                        };

                        line.clear();
                        if last_motion.is_none() || !matches!(last_motion, Some(Motion::Linear)) {
                            write!(line, "G01 ").unwrap();
                        }
                        write!(
                            line,
                            "X{:.*} Y{:.*} Z{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                        )
                        .unwrap();

                        if current_feed != Some(feed) {
                            write!(line, " F{:.*}", dp.min(1), feed).unwrap();
                            current_feed = Some(feed);
                        }
                        self.write_line(&mut out, &mut line_num, &line);
                        last_motion = Some(Motion::Linear);
                    }
                    Motion::ArcCW { i, j } => {
                        if in_canned_cycle {
                            self.write_line(&mut out, &mut line_num, "G80");
                            in_canned_cycle = false;
                        }

                        line.clear();
                        write!(
                            line,
                            "G02 X{:.*} Y{:.*} Z{:.*} I{:.*} J{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                            dp, i,
                            dp, j,
                        )
                        .unwrap();
                        if current_feed != Some(tp.feed_rate) {
                            write!(line, " F{:.*}", dp.min(1), tp.feed_rate).unwrap();
                            current_feed = Some(tp.feed_rate);
                        }
                        self.write_line(&mut out, &mut line_num, &line);
                        last_motion = Some(seg.motion);
                    }
                    Motion::ArcCCW { i, j } => {
                        if in_canned_cycle {
                            self.write_line(&mut out, &mut line_num, "G80");
                            in_canned_cycle = false;
                        }

                        line.clear();
                        write!(
                            line,
                            "G03 X{:.*} Y{:.*} Z{:.*} I{:.*} J{:.*}",
                            dp, seg.endpoint.x,
                            dp, seg.endpoint.y,
                            dp, seg.z,
                            dp, i,
                            dp, j,
                        )
                        .unwrap();
                        if current_feed != Some(tp.feed_rate) {
                            write!(line, " F{:.*}", dp.min(1), tp.feed_rate).unwrap();
                            current_feed = Some(tp.feed_rate);
                        }
                        self.write_line(&mut out, &mut line_num, &line);
                        last_motion = Some(seg.motion);
                    }
                    Motion::DrillCycle { retract_z, final_z, peck_depth } => {
                        // GRBL doesn't support canned cycles — emit manual moves
                        if controller == Controller::Grbl {
                            // Manual peck drill
                            let mut line = String::new();
                            write!(line, "G00 X{:.*} Y{:.*} Z{:.*}",
                                dp, seg.endpoint.x, dp, seg.endpoint.y, dp, retract_z).unwrap();
                            self.write_line(&mut out, &mut line_num, &line);

                            if peck_depth > 0.0 {
                                let mut z = -peck_depth;
                                while z > final_z + 1e-10 {
                                    let peck_line = format!("G01 Z{:.*} F{:.*}",
                                        dp, z, dp.min(1), tp.plunge_rate);
                                    self.write_line(&mut out, &mut line_num, &peck_line);
                                    let retract_line = format!("G00 Z{:.*}", dp, retract_z);
                                    self.write_line(&mut out, &mut line_num, &retract_line);
                                    z -= peck_depth;
                                }
                                // Final peck to full depth
                                let final_line = format!("G01 Z{:.*} F{:.*}",
                                    dp, final_z, dp.min(1), tp.plunge_rate);
                                self.write_line(&mut out, &mut line_num, &final_line);
                            } else {
                                let drill_line = format!("G01 Z{:.*} F{:.*}",
                                    dp, final_z, dp.min(1), tp.plunge_rate);
                                self.write_line(&mut out, &mut line_num, &drill_line);
                            }
                            let retract_line = format!("G00 Z{:.*}", dp, retract_z);
                            self.write_line(&mut out, &mut line_num, &retract_line);
                            last_motion = None;
                        } else {
                            // LinuxCNC and Mach support G81/G83
                            let mut line = String::new();
                            if peck_depth > 0.0 {
                                write!(line, "G83 X{:.*} Y{:.*} Z{:.*} R{:.*} Q{:.*} F{:.*}",
                                    dp, seg.endpoint.x,
                                    dp, seg.endpoint.y,
                                    dp, final_z,
                                    dp, retract_z,
                                    dp, peck_depth,
                                    dp.min(1), tp.plunge_rate,
                                ).unwrap();
                            } else {
                                write!(line, "G81 X{:.*} Y{:.*} Z{:.*} R{:.*} F{:.*}",
                                    dp, seg.endpoint.x,
                                    dp, seg.endpoint.y,
                                    dp, final_z,
                                    dp, retract_z,
                                    dp.min(1), tp.plunge_rate,
                                ).unwrap();
                            }
                            self.write_line(&mut out, &mut line_num, &line);
                            in_canned_cycle = true;
                            current_feed = Some(tp.plunge_rate);
                            last_motion = None;
                        }
                    }
                }
            }
        }

        // Cancel any active canned cycle
        if in_canned_cycle {
            self.write_line(&mut out, &mut line_num, "G80");
        }

        // --- Program footer ---
        self.write_line(&mut out, &mut line_num, "");
        self.write_line(&mut out, &mut line_num, "M05"); // Spindle stop
        self.write_line(&mut out, &mut line_num,
            &format!("G00 Z{:.*}", dp, self.profile.post.safe_z)); // Retract
        self.write_line(&mut out, &mut line_num, "G00 X0.0000 Y0.0000"); // Return to origin
        self.write_line(&mut out, &mut line_num, &self.profile.post.program_end); // M30

        // End-of-program marker (not used by GRBL)
        if controller != Controller::Grbl {
            writeln!(out, "%").unwrap();
        }

        out
    }

    /// Write a line with optional line numbers.
    fn write_line(&self, out: &mut String, line_num: &mut u32, content: &str) {
        if self.profile.post.line_numbers && !content.is_empty() {
            writeln!(out, "N{} {}", line_num, content).unwrap();
            *line_num += 10;
        } else {
            writeln!(out, "{}", content).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cam::ops::{generate_profile_cut, generate_drill, CamConfig, DrillHole};
    use cm_cam::ops::generate_drill_pattern;
    use cm_cam::toolpath::ToolpathSegment;
    use cm_core::geometry::{Point2D, Rect};
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

        for line in gcode.lines() {
            if line.contains("G00") && line.contains('Z') {
                let z_val = extract_z_value(line);
                if let Some(z) = z_val {
                    assert!(z >= 0.0, "Rapid move has negative Z: {line}");
                }
            }
        }
    }

    // --- Multi-controller tests ---

    #[test]
    fn test_linuxcnc_has_full_header() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);
        let gcode = emitter.emit(&[]);

        assert!(gcode.contains("G40"), "LinuxCNC should have G40");
        assert!(gcode.contains("G49"), "LinuxCNC should have G49");
        assert!(gcode.contains("G54"), "LinuxCNC should have G54");
        assert!(gcode.contains("%"), "LinuxCNC should have end marker");
    }

    #[test]
    fn test_grbl_simplified_header() {
        let profile = MachineProfile::shapeoko_xxl();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);
        let gcode = emitter.emit(&[tp]);

        // GRBL should NOT have G40, G49, G54
        assert!(!gcode.contains("G40"), "GRBL should not have G40");
        assert!(!gcode.contains("G49"), "GRBL should not have G49");
        assert!(!gcode.contains("G54"), "GRBL should not have G54");

        // GRBL should use M00 for tool change instead of M06
        assert!(gcode.contains("M00"), "GRBL should use M00 for tool pause");
        assert!(!gcode.contains("M06"), "GRBL should not have M06");

        // GRBL should not have % end marker
        assert!(!gcode.contains("%"), "GRBL should not have end marker");
    }

    #[test]
    fn test_mach_with_line_numbers() {
        let profile = MachineProfile::avid_cnc_48x96();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);
        let gcode = emitter.emit(&[tp]);

        // Mach should have line numbers
        assert!(gcode.contains("N10 "), "Mach should have N-word line numbers");
        assert!(gcode.contains("N20 "), "should have sequential line numbers");

        // Mach should have full header like LinuxCNC
        assert!(gcode.contains("G40"), "Mach should have G40");
        assert!(gcode.contains("G49"), "Mach should have G49");
        assert!(gcode.contains("%"), "Mach should have end marker");
    }

    #[test]
    fn test_mach_tool_change() {
        let profile = MachineProfile::avid_cnc_48x96();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);
        let gcode = emitter.emit(&[tp]);

        assert!(gcode.contains("T1 M06"), "Mach should use T M06 for tool change");
        assert!(gcode.contains("G43 H1"), "Mach should have tool length offset");
    }

    #[test]
    fn test_canned_cycle_gcode_linuxcnc() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig { use_canned_cycles: true, ..Default::default() };
        let tp = generate_drill(
            cm_core::geometry::Point2D::new(5.0, 5.0),
            0.5, &tool, 5000.0, &config,
        );
        let gcode = emitter.emit(&[tp]);

        assert!(gcode.contains("G81"), "LinuxCNC should emit G81 for simple drill");
        assert!(gcode.contains("G80"), "should have G80 cancel");
    }

    #[test]
    fn test_canned_cycle_peck_gcode() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig { use_canned_cycles: true, ..Default::default() };
        let holes = vec![DrillHole { x: 1.0, y: 1.0, depth: 2.0 }];
        let tp = generate_drill_pattern(&holes, &tool, 5000.0, &config);
        let gcode = emitter.emit(&[tp]);

        assert!(gcode.contains("G83"), "should emit G83 for peck drilling");
        assert!(gcode.contains("Q"), "G83 should have Q (peck depth)");
    }

    #[test]
    fn test_grbl_no_canned_cycles() {
        let profile = MachineProfile::shapeoko_xxl();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig { use_canned_cycles: true, ..Default::default() };
        let tp = generate_drill(
            cm_core::geometry::Point2D::new(5.0, 5.0),
            0.5, &tool, 5000.0, &config,
        );
        let gcode = emitter.emit(&[tp]);

        // GRBL should NOT have G81/G83 — manual moves instead
        assert!(!gcode.contains("G81"), "GRBL should not emit G81");
        assert!(!gcode.contains("G83"), "GRBL should not emit G83");
        assert!(gcode.contains("G01"), "GRBL should use manual plunge");
    }

    #[test]
    fn test_builtin_profiles() {
        let tormach = MachineProfile::tormach_pcnc1100();
        assert_eq!(tormach.machine.controller, Controller::LinuxCnc);

        let shapeoko = MachineProfile::shapeoko_xxl();
        assert_eq!(shapeoko.machine.controller, Controller::Grbl);
        assert_eq!(shapeoko.post.decimal_places, 3);

        let avid = MachineProfile::avid_cnc_48x96();
        assert_eq!(avid.machine.controller, Controller::Mach);
        assert!(avid.post.line_numbers);
    }

    // --- Additional multi-controller + edge-case tests ---

    #[test]
    fn test_grbl_manual_peck_drill() {
        // GRBL should produce manual peck moves for deep holes
        let profile = MachineProfile::shapeoko_xxl();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig { use_canned_cycles: true, ..Default::default() };
        let holes = vec![DrillHole { x: 1.0, y: 1.0, depth: 2.0 }];
        let tp = generate_drill_pattern(&holes, &tool, 5000.0, &config);
        let gcode = emitter.emit(&[tp]);

        // Should NOT have canned cycles
        assert!(!gcode.contains("G81"), "GRBL should not emit G81");
        assert!(!gcode.contains("G83"), "GRBL should not emit G83");
        // Should have multiple Z plunges (manual peck)
        let z_moves: Vec<_> = gcode.lines()
            .filter(|l| l.contains("G01") && l.contains("Z-"))
            .collect();
        assert!(z_moves.len() >= 2, "GRBL should manually peck multiple Z moves, got {}", z_moves.len());
    }

    #[test]
    fn test_multiple_tool_changes() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tp1 = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.0, 1.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(2.0, 1.0), z: -0.5 },
            ],
        };
        let tp2 = Toolpath {
            tool_number: 2, rpm: 3000.0, feed_rate: 80.0, plunge_rate: 40.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(6.0, 5.0), z: -0.3 },
            ],
        };
        let tp3 = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(3.0, 3.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(4.0, 3.0), z: -0.5 },
            ],
        };

        let gcode = emitter.emit(&[tp1, tp2, tp3]);

        // Should have 3 tool change sections
        let t1_count = gcode.matches("T1 M06").count();
        let t2_count = gcode.matches("T2 M06").count();
        assert_eq!(t1_count, 2, "T1 should be loaded twice");
        assert_eq!(t2_count, 1, "T2 should be loaded once");

        // Both RPMs should appear
        assert!(gcode.contains("S5000"), "should have 5000 RPM");
        assert!(gcode.contains("S3000"), "should have 3000 RPM");
    }

    #[test]
    fn test_arc_gcode_output() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(0.0, 0.0), z: 1.0 },
                ToolpathSegment { motion: Motion::ArcCW { i: 2.5, j: 0.0 }, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::ArcCCW { i: -2.5, j: 0.0 }, endpoint: Point2D::new(0.0, 0.0), z: -0.5 },
            ],
        };

        let gcode = emitter.emit(&[tp]);
        assert!(gcode.contains("G02"), "should have G02 for CW arc");
        assert!(gcode.contains("G03"), "should have G03 for CCW arc");
        assert!(gcode.contains("I"), "arcs should have I values");
        assert!(gcode.contains("J"), "arcs should have J values");
    }

    #[test]
    fn test_empty_toolpath_list() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);
        let gcode = emitter.emit(&[]);

        // Should still have valid header and footer
        assert!(gcode.contains("G20"));
        assert!(gcode.contains("M30"));
        assert!(gcode.contains("M05"));
        // Should not crash on empty input
    }

    #[test]
    fn test_metric_mode() {
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Millimeters);
        let gcode = emitter.emit(&[]);

        assert!(gcode.contains("G21"), "should set millimeter mode");
        assert!(!gcode.contains("G20"), "should not have inch mode");
    }

    #[test]
    fn test_decimal_places_respected() {
        // Shapeoko uses 3 decimal places
        let profile = MachineProfile::shapeoko_xxl();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.23456, 2.34567), z: 0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(3.45678, 4.56789), z: -0.25 },
            ],
        };
        let gcode = emitter.emit(&[tp]);

        // With 3 decimal places, values should be truncated to 3 decimals
        assert!(gcode.contains("X1.235") || gcode.contains("X1.234"), "should have 3 decimal places");
        assert!(!gcode.contains("X1.23456"), "should not have 5 decimal places");
    }

    #[test]
    fn test_canned_cycle_cancel_before_rapids() {
        // If a drill cycle is followed by a rapid, G80 should appear first
        let profile = MachineProfile::tormach_pcnc1100();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment {
                    motion: Motion::DrillCycle { retract_z: 0.25, final_z: -0.5, peck_depth: 0.0 },
                    endpoint: Point2D::new(1.0, 1.0),
                    z: -0.5,
                },
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
            ],
        };
        let gcode = emitter.emit(&[tp]);

        // G80 should appear between G81 and G00
        let g81_pos = gcode.find("G81").unwrap();
        let g80_pos = gcode.find("G80").unwrap();
        let g00_pos = gcode[g81_pos..].find("G00").map(|p| p + g81_pos).unwrap();

        assert!(g80_pos > g81_pos, "G80 should come after G81");
        assert!(g80_pos < g00_pos, "G80 should come before next G00");
    }

    #[test]
    fn test_line_numbers_increment_by_10() {
        let profile = MachineProfile::avid_cnc_48x96();
        let emitter = GCodeEmitter::new(&profile, Unit::Inches);

        let tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.0, 1.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(2.0, 1.0), z: -0.5 },
            ],
        };
        let gcode = emitter.emit(&[tp]);

        // Check that line numbers increment by 10
        let n_lines: Vec<u32> = gcode.lines()
            .filter_map(|l| {
                l.strip_prefix('N')
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|n| n.parse::<u32>().ok())
            })
            .collect();

        assert!(n_lines.len() >= 2, "should have at least 2 numbered lines");
        for window in n_lines.windows(2) {
            assert_eq!(window[1] - window[0], 10, "line numbers should increment by 10");
        }
    }

    fn extract_z_value(line: &str) -> Option<f64> {
        line.split_whitespace()
            .find(|word| word.starts_with('Z'))
            .and_then(|z_word| z_word[1..].parse::<f64>().ok())
    }
}
