//! Integration tests for the full cabinet-maker pipeline.
//!
//! Tests the complete flow: TOML project → part generation → toolpath generation
//! → G-code emission, verifying correctness at each stage.

use cm_cabinet::project::Project;
use cm_cabinet::part::{PartOperation, DadoOrientation};
use cm_cam::ops::{
    generate_profile_cut, generate_dado_toolpath, generate_rabbet_toolpath,
    generate_drill, CamConfig, RabbetEdge,
};
use cm_cam::toolpath::Motion;
use cm_core::geometry::Point2D;
use cm_core::tool::Tool;
use cm_core::units::Unit;
use cm_post::gcode::GCodeEmitter;
use cm_post::machine::MachineProfile;

const BOOKSHELF_TOML: &str = r#"
[project]
name = "Test Bookshelf"
units = "inches"

[material]
name = "3/4\" Birch Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"

[cabinet]
name = "bookshelf"
cabinet_type = "basic_box"
width = 36.0
height = 30.0
depth = 12.0
material_thickness = 0.75
back_thickness = 0.25
shelf_count = 2
shelf_joinery = "dado"
dado_depth_fraction = 0.5
has_back = true
back_joinery = "rabbet"

[[tools]]
number = 1
tool_type = "endmill"
diameter = 0.25
flutes = 2
cutting_length = 1.0
description = "1/4\" 2-flute upcut endmill"
"#;

/// Parse the bookshelf TOML and verify project structure.
#[test]
fn test_project_loads_from_toml() {
    let project = Project::from_toml(BOOKSHELF_TOML).expect("failed to parse TOML");
    assert_eq!(project.project.name, "Test Bookshelf");
    assert_eq!(project.project.units, Unit::Inches);
    assert_eq!(project.cabinet.width, 36.0);
    assert_eq!(project.cabinet.height, 30.0);
    assert_eq!(project.cabinet.depth, 12.0);
    assert_eq!(project.cabinet.shelf_count, 2);
    assert_eq!(project.tools.len(), 1);
}

/// Verify part generation produces correct number and types of parts.
#[test]
fn test_part_generation() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();

    // Expected: left_side, right_side, bottom, top, shelf (qty 2), back = 6 entries
    assert_eq!(parts.len(), 6);

    let labels: Vec<&str> = parts.iter().map(|p| p.label.as_str()).collect();
    assert!(labels.contains(&"left_side"));
    assert!(labels.contains(&"right_side"));
    assert!(labels.contains(&"top"));
    assert!(labels.contains(&"bottom"));
    assert!(labels.contains(&"shelf"));
    assert!(labels.contains(&"back"));

    // Shelf should have quantity 2
    let shelf = parts.iter().find(|p| p.label == "shelf").unwrap();
    assert_eq!(shelf.quantity, 2);
}

/// Verify side panels have the correct operations (dados + rabbet).
#[test]
fn test_side_panel_operations() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();
    let left = parts.iter().find(|p| p.label == "left_side").unwrap();

    // Should have: bottom dado + top dado + 2 shelf dados + 1 back rabbet = 5 ops
    assert_eq!(left.operations.len(), 5);

    let dado_count = left.operations.iter()
        .filter(|op| matches!(op, PartOperation::Dado(_)))
        .count();
    assert_eq!(dado_count, 4, "should have 4 dados (top, bottom, 2 shelves)");

    let rabbet_count = left.operations.iter()
        .filter(|op| matches!(op, PartOperation::Rabbet(_)))
        .count();
    assert_eq!(rabbet_count, 1, "should have 1 rabbet for back panel");
}

/// Full pipeline: TOML → parts → toolpaths → G-code.
/// Verifies the entire pipeline produces valid G-code.
#[test]
fn test_full_pipeline_bookshelf() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();
    let machine = MachineProfile::tormach_pcnc1100();
    let tool = project.tools.first().cloned().unwrap();
    let rpm = machine.machine.max_rpm * 0.9;

    let cam_config = CamConfig {
        safe_z: machine.post.safe_z,
        rapid_z: machine.post.rapid_z,
        ..Default::default()
    };

    let mut all_toolpaths = Vec::new();

    for part in &parts {
        // Generate operation toolpaths
        for op in &part.operations {
            match op {
                PartOperation::Dado(dado) => {
                    let horizontal = dado.orientation == DadoOrientation::Horizontal;
                    let tp = generate_dado_toolpath(
                        &part.rect, dado.position, dado.width, dado.depth,
                        horizontal, &tool, rpm, &cam_config,
                    );
                    all_toolpaths.push(tp);
                }
                PartOperation::Rabbet(rabbet) => {
                    let edge = match rabbet.edge {
                        cm_cabinet::part::Edge::Top => RabbetEdge::Top,
                        cm_cabinet::part::Edge::Bottom => RabbetEdge::Bottom,
                        cm_cabinet::part::Edge::Left => RabbetEdge::Left,
                        cm_cabinet::part::Edge::Right => RabbetEdge::Right,
                    };
                    let tp = generate_rabbet_toolpath(
                        &part.rect, edge, rabbet.width, rabbet.depth,
                        &tool, rpm, &cam_config,
                    );
                    all_toolpaths.push(tp);
                }
                PartOperation::Drill(drill) => {
                    let tp = generate_drill(
                        Point2D::new(part.rect.min_x() + drill.x, part.rect.min_y() + drill.y),
                        drill.depth, &tool, rpm, &cam_config,
                    );
                    all_toolpaths.push(tp);
                }
            }
        }

        // Profile cut
        let tp = generate_profile_cut(&part.rect, part.thickness, &tool, rpm, &cam_config);
        all_toolpaths.push(tp);
    }

    // Should have toolpaths for all operations + profile cuts
    // Sides: 4 dados + 1 rabbet + 1 profile = 6 each, x2 = 12
    // Top/bottom: 1 rabbet + 1 profile = 2 each, x2 = 4
    // Shelf: 1 profile = 1
    // Back: 1 profile = 1
    // Total: 12 + 4 + 1 + 1 = 18
    assert_eq!(all_toolpaths.len(), 18, "expected 18 toolpaths");

    // Emit G-code
    let emitter = GCodeEmitter::new(&machine, project.project.units);
    let gcode = emitter.emit(&all_toolpaths);

    // Verify G-code structure
    assert!(gcode.contains("G20"), "should set inch mode");
    assert!(gcode.contains("G90"), "should set absolute mode");
    assert!(gcode.contains("T1 M06"), "should have tool change");
    assert!(gcode.contains("M03"), "should start spindle");
    assert!(gcode.contains("M05"), "should stop spindle");
    assert!(gcode.contains("M30"), "should end program");
    assert!(gcode.contains("G00"), "should have rapid moves");
    assert!(gcode.contains("G01"), "should have linear feed moves");

    // Verify no empty G-code
    assert!(gcode.lines().count() > 100, "G-code should have substantial content");
}

/// Verify G-code never has rapid moves at negative Z (safety check).
#[test]
fn test_gcode_no_rapid_plunge() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();
    let machine = MachineProfile::tormach_pcnc1100();
    let tool = project.tools.first().cloned().unwrap();
    let rpm = 5000.0;
    let cam_config = CamConfig::default();

    let mut all_toolpaths = Vec::new();
    for part in &parts {
        for op in &part.operations {
            match op {
                PartOperation::Dado(dado) => {
                    let horizontal = dado.orientation == DadoOrientation::Horizontal;
                    all_toolpaths.push(generate_dado_toolpath(
                        &part.rect, dado.position, dado.width, dado.depth,
                        horizontal, &tool, rpm, &cam_config,
                    ));
                }
                PartOperation::Rabbet(rabbet) => {
                    let edge = match rabbet.edge {
                        cm_cabinet::part::Edge::Top => RabbetEdge::Top,
                        cm_cabinet::part::Edge::Bottom => RabbetEdge::Bottom,
                        cm_cabinet::part::Edge::Left => RabbetEdge::Left,
                        cm_cabinet::part::Edge::Right => RabbetEdge::Right,
                    };
                    all_toolpaths.push(generate_rabbet_toolpath(
                        &part.rect, edge, rabbet.width, rabbet.depth,
                        &tool, rpm, &cam_config,
                    ));
                }
                PartOperation::Drill(drill) => {
                    all_toolpaths.push(generate_drill(
                        Point2D::new(part.rect.min_x() + drill.x, part.rect.min_y() + drill.y),
                        drill.depth, &tool, rpm, &cam_config,
                    ));
                }
            }
        }
        all_toolpaths.push(generate_profile_cut(&part.rect, part.thickness, &tool, rpm, &cam_config));
    }

    // Check all toolpaths: rapid moves must never go below Z=0
    for (tp_idx, tp) in all_toolpaths.iter().enumerate() {
        for (seg_idx, seg) in tp.segments.iter().enumerate() {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(
                    seg.z >= 0.0,
                    "Rapid move at negative Z in toolpath {tp_idx}, segment {seg_idx}: z={:.4}",
                    seg.z,
                );
            }
        }
    }
}

/// Verify that tabs exist on profile cuts and are shallower than full depth.
#[test]
fn test_profile_cuts_have_tabs() {
    let tool = Tool::quarter_inch_endmill();
    let config = CamConfig {
        tabs_per_side: 2,
        tab_width: 0.5,
        tab_height: 0.125,
        depth_per_pass: 0.25,
        ..Default::default()
    };

    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();

    for part in &parts {
        let tp = generate_profile_cut(&part.rect, part.thickness, &tool, 5000.0, &config);

        let full_depth = -part.thickness;
        let tab_z = full_depth + config.tab_height;

        // On the final pass, there should be segments at tab_z
        let tab_segments: Vec<_> = tp.segments.iter()
            .filter(|s| (s.z - tab_z).abs() < 1e-10)
            .collect();

        assert!(
            !tab_segments.is_empty(),
            "Part '{}' profile cut should have tab segments at z={:.4}",
            part.label, tab_z,
        );
    }
}

/// Verify dado depths match the parametric specification.
#[test]
fn test_dado_depths_match_specification() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let parts = project.cabinet.generate_parts();
    let tool = Tool::quarter_inch_endmill();
    let config = CamConfig::default();

    let expected_dado_depth = project.cabinet.material_thickness * project.cabinet.dado_depth_fraction;

    let left = parts.iter().find(|p| p.label == "left_side").unwrap();
    for op in &left.operations {
        if let PartOperation::Dado(dado) = op {
            assert!(
                (dado.depth - expected_dado_depth).abs() < 1e-10,
                "dado depth {:.4} doesn't match expected {:.4}",
                dado.depth, expected_dado_depth,
            );

            // Generate toolpath and verify cut depth
            let tp = generate_dado_toolpath(
                &left.rect, dado.position, dado.width, dado.depth,
                true, &tool, 5000.0, &config,
            );

            let deepest = tp.segments.iter()
                .map(|s| s.z)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            assert!(
                (deepest - (-expected_dado_depth)).abs() < 1e-10,
                "toolpath deepest cut {:.4} doesn't match dado depth {:.4}",
                deepest, -expected_dado_depth,
            );
        }
    }
}

/// Verify the project can round-trip through TOML serialization.
#[test]
fn test_project_toml_round_trip() {
    let project = Project::from_toml(BOOKSHELF_TOML).unwrap();
    let toml_string = project.to_toml().expect("failed to serialize to TOML");
    let project2 = Project::from_toml(&toml_string).expect("failed to re-parse TOML");

    assert_eq!(project.project.name, project2.project.name);
    assert_eq!(project.cabinet.width, project2.cabinet.width);
    assert_eq!(project.cabinet.height, project2.cabinet.height);
    assert_eq!(project.cabinet.depth, project2.cabinet.depth);
    assert_eq!(project.cabinet.shelf_count, project2.cabinet.shelf_count);
}

/// Test with a minimal cabinet (no shelves, no back).
#[test]
fn test_minimal_cabinet_pipeline() {
    let toml = r#"
[project]
name = "Minimal Box"
units = "inches"

[material]
name = "3/4\" Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"

[cabinet]
name = "box"
cabinet_type = "basic_box"
width = 24.0
height = 24.0
depth = 12.0
material_thickness = 0.75
shelf_count = 0
has_back = false
"#;

    let project = Project::from_toml(toml).unwrap();
    let parts = project.cabinet.generate_parts();

    // No shelves, no back: left_side, right_side, top, bottom = 4
    assert_eq!(parts.len(), 4);

    let tool = Tool::quarter_inch_endmill();
    let machine = MachineProfile::tormach_pcnc1100();
    let config = CamConfig::default();

    let mut toolpaths = Vec::new();
    for part in &parts {
        for op in &part.operations {
            if let PartOperation::Dado(dado) = op {
                let horizontal = dado.orientation == DadoOrientation::Horizontal;
                toolpaths.push(generate_dado_toolpath(
                    &part.rect, dado.position, dado.width, dado.depth,
                    horizontal, &tool, 5000.0, &config,
                ));
            }
        }
        toolpaths.push(generate_profile_cut(&part.rect, part.thickness, &tool, 5000.0, &config));
    }

    let emitter = GCodeEmitter::new(&machine, Unit::Inches);
    let gcode = emitter.emit(&toolpaths);

    assert!(gcode.contains("G20"));
    assert!(gcode.contains("M30"));
    assert!(gcode.lines().count() > 50);
}

/// Test that millimeter mode produces correct G-code header.
#[test]
fn test_millimeter_mode() {
    let toml = r#"
[project]
name = "Metric Box"
units = "millimeters"

[material]
name = "18mm Plywood"
thickness = 18.0
sheet_width = 1220.0
sheet_length = 2440.0
material_type = "plywood"

[cabinet]
name = "metric_shelf"
cabinet_type = "basic_box"
width = 600.0
height = 800.0
depth = 300.0
material_thickness = 18.0
shelf_count = 1
has_back = false
"#;

    let project = Project::from_toml(toml).unwrap();
    let parts = project.cabinet.generate_parts();
    let tool = Tool::quarter_inch_endmill();
    let config = CamConfig {
        safe_z: 25.0,
        rapid_z: 5.0,
        depth_per_pass: 6.0,
        ..Default::default()
    };
    let machine = MachineProfile::tormach_pcnc1100();

    let mut toolpaths = Vec::new();
    for part in &parts {
        toolpaths.push(generate_profile_cut(&part.rect, part.thickness, &tool, 5000.0, &config));
    }

    let emitter = GCodeEmitter::new(&machine, Unit::Millimeters);
    let gcode = emitter.emit(&toolpaths);

    assert!(gcode.contains("G21"), "metric mode should use G21");
    assert!(!gcode.contains("G20"), "metric mode should not have G20");
}
