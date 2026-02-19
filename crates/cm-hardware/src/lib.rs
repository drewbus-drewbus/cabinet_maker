//! Hardware Library for Cabinet Maker.
//!
//! Provides a catalog of cabinet hardware (hinges, drawer slides, shelf pins, pulls,
//! confirmat screws, cam locks, edge banding) with automatic boring pattern generation.
//! After joinery rules apply operations, `HardwareApplicator` adds drill operations
//! and adjusts part dimensions (e.g., drawer width reduced by slide clearance).
//!
//! # Hardware Types
//!
//! | Hardware | Boring Pattern |
//! |----------|----------------|
//! | 35mm cup hinge (Blum, etc.) | 35mm bore at specified depth + mounting plate holes |
//! | Butt hinge | Mortise pocket (face-frame) |
//! | Side-mount slides | Screw holes at specified spacing |
//! | Undermount slides | Screw holes on cabinet bottom/stretcher |
//! | 5mm shelf pins | 32mm-spaced grid of holes |
//! | Drawer pulls | Through-holes at specified spacing |
//! | Confirmat screws | 5mm pilot + 10mm counterbore (face) + 7mm edge bore |
//! | Cam locks | 15mm cam bore (face) + 8mm bolt hole (edge) |
//! | Edge banding | Tracking only (BOM/costing, no boring pattern) |

pub mod catalog;
pub mod error;

pub use catalog::{
    Hardware, HardwareKind, HingeSpec, SlideSpec, ShelfPinSpec, PullSpec,
    ConfirmatSpec, CamLockSpec, EdgeBandSpec, EdgeBandMaterial,
    HardwareApplicator, HardwareOp,
};
pub use error::HardwareError;

use cm_cabinet::cabinet::{Cabinet, CabinetType};

/// A hardware assignment: what hardware goes where on a cabinet.
#[derive(Debug, Clone)]
pub struct HardwareAssignment {
    /// The hardware item.
    pub hardware: Hardware,
    /// How many of this hardware item are needed.
    pub quantity: u32,
    /// Description of where/why this hardware is assigned.
    pub description: String,
}

/// Automatically assign standard hardware to a cabinet based on its type and dimensions.
///
/// Rules:
/// - Hinges: 2 for doors < 40", 3 for 40-60", 4 for > 60" (35mm European standard)
/// - Slides: 1 pair per drawer (length = drawer depth - 1")
/// - Shelf pins: 4 per adjustable shelf (2 rows x 2 sides)
/// - Pulls: 1 per door/drawer
pub fn auto_assign_hardware(cabinet: &Cabinet) -> Vec<HardwareAssignment> {
    let mut assignments = Vec::new();

    match cabinet.cabinet_type {
        CabinetType::BasicBox => {
            // Bookshelf: just shelf pins for adjustable shelves
            if cabinet.shelf_count > 0 {
                assignments.push(HardwareAssignment {
                    hardware: Hardware::shelf_pin_5mm(),
                    quantity: cabinet.shelf_count * 4, // 4 pins per shelf
                    description: format!("{} adjustable shelves x 4 pins each", cabinet.shelf_count),
                });
            }
        }

        CabinetType::WallCabinet => {
            // Wall cabinet: 2 doors (pair), hinges per door, pulls, shelf pins
            let door_height = cabinet.height;
            let hinges_per_door = hinge_count_for_height(door_height);
            let door_count = 2u32; // standard pair

            assignments.push(HardwareAssignment {
                hardware: Hardware::blum_clip_top_110(),
                quantity: door_count * hinges_per_door,
                description: format!("{} doors x {} hinges each", door_count, hinges_per_door),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: door_count,
                description: format!("{} door pulls", door_count),
            });

            if cabinet.shelf_count > 0 {
                assignments.push(HardwareAssignment {
                    hardware: Hardware::shelf_pin_5mm(),
                    quantity: cabinet.shelf_count * 4,
                    description: format!("{} adjustable shelves x 4 pins each", cabinet.shelf_count),
                });
            }
        }

        CabinetType::BaseCabinet | CabinetType::VanityBase => {
            // Base/vanity: 1 door, hinges, pull, shelf pins
            let door_height = cabinet.height
                - cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);
            let hinges_per_door = hinge_count_for_height(door_height);

            assignments.push(HardwareAssignment {
                hardware: Hardware::blum_clip_top_110(),
                quantity: hinges_per_door,
                description: format!("1 door x {} hinges", hinges_per_door),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: 1,
                description: "1 door pull".to_string(),
            });

            if cabinet.shelf_count > 0 {
                assignments.push(HardwareAssignment {
                    hardware: Hardware::shelf_pin_5mm(),
                    quantity: cabinet.shelf_count * 4,
                    description: format!("{} adjustable shelves x 4 pins each", cabinet.shelf_count),
                });
            }
        }

        CabinetType::TallCabinet => {
            // Tall: 2 doors (upper + lower), more hinges, pulls, shelf pins
            let door_height = (cabinet.height
                - cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height)) / 2.0;
            let hinges_per_door = hinge_count_for_height(door_height);
            let door_count = 2u32;

            assignments.push(HardwareAssignment {
                hardware: Hardware::blum_clip_top_110(),
                quantity: door_count * hinges_per_door,
                description: format!("{} doors x {} hinges each", door_count, hinges_per_door),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: door_count,
                description: format!("{} door pulls", door_count),
            });

            if cabinet.shelf_count > 0 {
                assignments.push(HardwareAssignment {
                    hardware: Hardware::shelf_pin_5mm(),
                    quantity: cabinet.shelf_count * 4,
                    description: format!("{} adjustable shelves x 4 pins each", cabinet.shelf_count),
                });
            }
        }

        CabinetType::SinkBase => {
            // Sink base: 2 doors (no drawers, no shelves typically)
            let door_height = cabinet.height
                - cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);
            let hinges_per_door = hinge_count_for_height(door_height);
            let door_count = 2u32;

            assignments.push(HardwareAssignment {
                hardware: Hardware::blum_clip_top_110(),
                quantity: door_count * hinges_per_door,
                description: format!("{} doors x {} hinges each", door_count, hinges_per_door),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: door_count,
                description: format!("{} door pulls", door_count),
            });
        }

        CabinetType::DrawerBank => {
            // Drawer bank: slides + pulls per drawer
            let drawer_count = cabinet.drawers.as_ref().map_or(4, |d| d.count);

            assignments.push(HardwareAssignment {
                hardware: Hardware::side_mount_slide(),
                quantity: drawer_count, // 1 pair per drawer (sold as pairs)
                description: format!("{} drawer slide pairs", drawer_count),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: drawer_count,
                description: format!("{} drawer pulls", drawer_count),
            });
        }

        CabinetType::CornerCabinet => {
            // Corner cabinet: 1 door (diagonal face), hinges, pull, shelf pins
            let door_height = cabinet.height
                - cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);
            let hinges_per_door = hinge_count_for_height(door_height);

            assignments.push(HardwareAssignment {
                hardware: Hardware::blum_clip_top_110(),
                quantity: hinges_per_door,
                description: format!("1 door x {} hinges", hinges_per_door),
            });

            assignments.push(HardwareAssignment {
                hardware: Hardware::bar_pull_3in(),
                quantity: 1,
                description: "1 door pull".to_string(),
            });

            if cabinet.shelf_count > 0 {
                assignments.push(HardwareAssignment {
                    hardware: Hardware::shelf_pin_5mm(),
                    quantity: cabinet.shelf_count * 4,
                    description: format!("{} adjustable shelves x 4 pins each", cabinet.shelf_count),
                });
            }
        }
    }

    assignments
}

/// Determine hinge count based on door height (European 35mm hinge standard).
///
/// - < 40": 2 hinges
/// - 40-60": 3 hinges
/// - > 60": 4 hinges
fn hinge_count_for_height(height: f64) -> u32 {
    if height > 60.0 {
        4
    } else if height > 40.0 {
        3
    } else {
        2
    }
}

// =========================================================================
// Hardware → CAM Pipeline Integration (Phase 16c)
//
// These functions bridge the hardware catalog (auto-assignment) to the
// toolpath pipeline by generating concrete drill operations positioned
// on specific parts of a cabinet.
// =========================================================================

/// Compute the adjustable shelf pin zone (bottom_y, top_y) on a side panel.
///
/// The zone excludes fixed top/bottom panels (material thickness) and toe kick.
/// Returns `(zone_bottom, zone_top)` in panel-local Y coordinates.
pub fn shelf_pin_zone(cabinet: &Cabinet) -> (f64, f64) {
    let mt = cabinet.material_thickness;
    let tk_height = cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);

    // Bottom of zone: above toe kick + bottom panel + a little clearance
    let zone_bottom = tk_height + mt;
    // Top of zone: below top panel
    let zone_top = cabinet.height - mt;

    (zone_bottom, zone_top)
}

/// Compute standard European hinge Y positions on a side panel.
///
/// Hinges are inset 3" from the top and bottom of the door opening,
/// with additional hinges evenly spaced between them.
pub fn hinge_y_positions(door_opening_bottom: f64, door_height: f64, count: u32) -> Vec<f64> {
    if count == 0 {
        return Vec::new();
    }

    let inset = 3.0; // standard 3" inset from door opening edges
    let top_y = door_opening_bottom + door_height - inset;
    let bottom_y = door_opening_bottom + inset;

    if count == 1 {
        return vec![(top_y + bottom_y) / 2.0];
    }

    let mut positions = vec![bottom_y];
    if count > 2 {
        let span = top_y - bottom_y;
        let spacing = span / (count - 1) as f64;
        for i in 1..(count - 1) {
            positions.push(bottom_y + spacing * i as f64);
        }
    }
    positions.push(top_y);
    positions
}

/// Compute the Y coordinate of each drawer opening bottom on a side panel.
///
/// For a DrawerBank, drawers are stacked from bottom to top, each with equal
/// opening height. The first drawer sits above the toe kick + bottom panel.
pub fn drawer_opening_bottoms(cabinet: &Cabinet) -> Vec<f64> {
    let mt = cabinet.material_thickness;
    let tk_height = cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);
    let drawer_count = cabinet.drawers.as_ref().map_or(4, |d| d.count);

    // Usable interior height
    let interior_bottom = tk_height + mt;
    let interior_top = cabinet.height - mt;
    let interior_height = interior_top - interior_bottom;

    // Each drawer opening = (interior_height - divider_thicknesses) / count
    // Dividers between drawers: (count - 1) × material_thickness
    let dividers = (drawer_count.saturating_sub(1)) as f64 * mt;
    let opening_height = (interior_height - dividers) / drawer_count as f64;

    let mut bottoms = Vec::new();
    for i in 0..drawer_count {
        let bottom = interior_bottom + i as f64 * (opening_height + mt);
        bottoms.push(bottom);
    }
    bottoms
}

/// Compute the door opening height (interior height minus toe kick).
pub fn door_opening_height(cabinet: &Cabinet) -> f64 {
    let mt = cabinet.material_thickness;
    let tk_height = cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);
    cabinet.height - tk_height - 2.0 * mt
}

/// Generate all hardware drill operations for a cabinet, targeted at specific part labels.
///
/// This is the main integration point: it combines `auto_assign_hardware()` with
/// position calculation to produce `HardwareOp`s that can be injected into parts.
///
/// Operations generated:
/// - **Shelf pins**: On `left_side` and `right_side`, within the adjustable zone
/// - **Hinge mounting plates**: On `left_side`, at standard hinge Y positions
/// - **Drawer slide holes**: On `left_side` and `right_side`, one pair per drawer
pub fn generate_all_hardware_ops(cabinet: &Cabinet) -> Vec<HardwareOp> {
    let mut ops = Vec::new();
    let assignments = auto_assign_hardware(cabinet);

    for assignment in &assignments {
        match &assignment.hardware.kind {
            HardwareKind::ShelfPin(spec) => {
                // Shelf pin holes on both side panels
                let (zone_bottom, zone_top) = shelf_pin_zone(cabinet);
                let panel_width = cabinet.depth;

                for side in &["left_side", "right_side"] {
                    let pin_ops = HardwareApplicator::shelf_pin_holes(
                        spec, side, panel_width, zone_bottom, zone_top,
                    );
                    ops.extend(pin_ops);
                }
            }

            HardwareKind::Hinge(spec) => {
                // Hinge mounting plate holes on the left side panel
                // (hinges mount on the cabinet side, not the door — door cups are skipped
                //  because door panels are not yet generated)
                let mt = cabinet.material_thickness;
                let tk_height = cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);

                match cabinet.cabinet_type {
                    CabinetType::TallCabinet => {
                        // Two doors: upper and lower
                        let door_height = (cabinet.height - tk_height - 2.0 * mt) / 2.0;
                        let hinges_per_door = hinge_count_for_height(door_height);

                        // Lower door
                        let lower_bottom = tk_height + mt;
                        let lower_positions = hinge_y_positions(lower_bottom, door_height, hinges_per_door);
                        for y in &lower_positions {
                            ops.extend(HardwareApplicator::hinge_mounting_plate(
                                spec, "left_side", cabinet.depth, *y,
                            ));
                        }

                        // Upper door
                        let upper_bottom = lower_bottom + door_height;
                        let upper_positions = hinge_y_positions(upper_bottom, door_height, hinges_per_door);
                        for y in &upper_positions {
                            ops.extend(HardwareApplicator::hinge_mounting_plate(
                                spec, "left_side", cabinet.depth, *y,
                            ));
                        }
                    }
                    CabinetType::WallCabinet | CabinetType::SinkBase => {
                        // Two doors — hinges on left_side (left door) and right_side (right door)
                        let door_height = door_opening_height(cabinet);
                        let hinges_per_door = hinge_count_for_height(
                            door_height + 2.0 * mt, // gross door height for hinge count
                        );
                        let opening_bottom = tk_height + mt;
                        let positions = hinge_y_positions(opening_bottom, door_height, hinges_per_door);

                        for y in &positions {
                            ops.extend(HardwareApplicator::hinge_mounting_plate(
                                spec, "left_side", cabinet.depth, *y,
                            ));
                        }
                        for y in &positions {
                            ops.extend(HardwareApplicator::hinge_mounting_plate(
                                spec, "right_side", cabinet.depth, *y,
                            ));
                        }
                    }
                    _ => {
                        // Single door — hinges on left_side
                        let door_height = door_opening_height(cabinet);
                        let hinges_per_door = hinge_count_for_height(
                            door_height + 2.0 * mt,
                        );
                        let opening_bottom = tk_height + mt;
                        let positions = hinge_y_positions(opening_bottom, door_height, hinges_per_door);

                        for y in &positions {
                            ops.extend(HardwareApplicator::hinge_mounting_plate(
                                spec, "left_side", cabinet.depth, *y,
                            ));
                        }
                    }
                }
            }

            HardwareKind::Slide(spec) => {
                // Drawer slide holes on both side panels
                let bottoms = drawer_opening_bottoms(cabinet);
                let slide_length = cabinet.depth - 1.0; // slides are 1" shorter than depth

                for opening_bottom in &bottoms {
                    for side in &["left_side", "right_side"] {
                        ops.extend(HardwareApplicator::side_mount_slide_holes(
                            spec, side, *opening_bottom, slide_length,
                        ));
                    }
                }
            }

            // Pull holes target door/drawer fronts (not yet generated) — skip
            HardwareKind::Pull(_) => {}

            // Confirmat/CamLock are manual specification only — skip
            HardwareKind::Confirmat(_) | HardwareKind::CamLock(_) => {}

            // Edge banding has no boring pattern — skip
            HardwareKind::EdgeBand(_) => {}
        }
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cabinet::cabinet::*;
    use cm_cabinet::part::PartOperation;

    fn test_cabinet(cabinet_type: CabinetType) -> Cabinet {
        Cabinet {
            name: "test".to_string(),
            cabinet_type,
            width: 24.0,
            height: 30.0,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 2,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: None,
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_auto_assign_wall_cabinet() {
        let cab = test_cabinet(CabinetType::WallCabinet);
        let assignments = auto_assign_hardware(&cab);

        // Should have: hinges, pulls, shelf pins
        assert_eq!(assignments.len(), 3);

        // Hinges: 30" door → 2 per door × 2 doors = 4
        assert!(matches!(assignments[0].hardware.kind, HardwareKind::Hinge(_)));
        assert_eq!(assignments[0].quantity, 4);

        // Pulls: 2 doors
        assert!(matches!(assignments[1].hardware.kind, HardwareKind::Pull(_)));
        assert_eq!(assignments[1].quantity, 2);

        // Shelf pins: 2 shelves × 4 = 8
        assert!(matches!(assignments[2].hardware.kind, HardwareKind::ShelfPin(_)));
        assert_eq!(assignments[2].quantity, 8);
    }

    #[test]
    fn test_auto_assign_drawer_bank() {
        let mut cab = test_cabinet(CabinetType::DrawerBank);
        cab.drawers = Some(DrawerConfig {
            count: 4,
            opening_height: 0.0,
            slide_clearance: 0.5,
        });
        let assignments = auto_assign_hardware(&cab);

        // Should have: slides, pulls
        assert_eq!(assignments.len(), 2);

        // Slides: 4 pairs
        assert!(matches!(assignments[0].hardware.kind, HardwareKind::Slide(_)));
        assert_eq!(assignments[0].quantity, 4);

        // Pulls: 4 drawer pulls
        assert!(matches!(assignments[1].hardware.kind, HardwareKind::Pull(_)));
        assert_eq!(assignments[1].quantity, 4);
    }

    #[test]
    fn test_auto_assign_basic_box() {
        let cab = test_cabinet(CabinetType::BasicBox);
        let assignments = auto_assign_hardware(&cab);

        // BasicBox: just shelf pins
        assert_eq!(assignments.len(), 1);
        assert!(matches!(assignments[0].hardware.kind, HardwareKind::ShelfPin(_)));
        assert_eq!(assignments[0].quantity, 8);
    }

    #[test]
    fn test_auto_assign_base_cabinet_with_toe_kick() {
        let mut cab = test_cabinet(CabinetType::BaseCabinet);
        cab.height = 34.5;
        cab.toe_kick = Some(ToeKickConfig {
            height: 4.0,
            setback: 3.0,
        });
        let assignments = auto_assign_hardware(&cab);

        // Should have: hinges, pull, shelf pins
        assert_eq!(assignments.len(), 3);

        // Door height = 34.5 - 4.0 = 30.5 → 2 hinges
        assert_eq!(assignments[0].quantity, 2);
    }

    #[test]
    fn test_auto_assign_tall_cabinet_more_hinges() {
        let mut cab = test_cabinet(CabinetType::TallCabinet);
        cab.height = 84.0;
        cab.toe_kick = Some(ToeKickConfig {
            height: 4.0,
            setback: 3.0,
        });
        let assignments = auto_assign_hardware(&cab);

        // Door height = (84 - 4) / 2 = 40 → 2 hinges per door
        // 2 doors × 2 hinges = 4
        assert!(matches!(assignments[0].hardware.kind, HardwareKind::Hinge(_)));
        assert_eq!(assignments[0].quantity, 4);
    }

    #[test]
    fn test_auto_assign_tall_cabinet_very_tall() {
        let mut cab = test_cabinet(CabinetType::TallCabinet);
        cab.height = 96.0;
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
        let assignments = auto_assign_hardware(&cab);

        // Door height = (96 - 4) / 2 = 46 → 3 hinges per door (> 40")
        assert_eq!(assignments[0].quantity, 6); // 2 × 3
    }

    #[test]
    fn test_auto_assign_sink_base() {
        let mut cab = test_cabinet(CabinetType::SinkBase);
        cab.shelf_count = 0; // sink base typically has no shelves
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
        let assignments = auto_assign_hardware(&cab);

        // hinges + pulls, no shelf pins
        assert_eq!(assignments.len(), 2);
        assert!(matches!(assignments[0].hardware.kind, HardwareKind::Hinge(_)));
        assert!(matches!(assignments[1].hardware.kind, HardwareKind::Pull(_)));
    }

    #[test]
    fn test_auto_assign_corner_cabinet() {
        let mut cab = test_cabinet(CabinetType::CornerCabinet);
        cab.width = 36.0;
        cab.depth = 36.0;
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
        let assignments = auto_assign_hardware(&cab);

        // 1 door (hinges, pull) + shelf pins
        assert_eq!(assignments.len(), 3);
        // Door height = 30 - 4 = 26 → 2 hinges
        assert_eq!(assignments[0].quantity, 2);
    }

    #[test]
    fn test_auto_assign_vanity_base() {
        let mut cab = test_cabinet(CabinetType::VanityBase);
        cab.height = 31.5;
        cab.depth = 21.0;
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
        let assignments = auto_assign_hardware(&cab);

        // hinges, pull, shelf pins (like base cabinet)
        assert_eq!(assignments.len(), 3);
        // Door height = 31.5 - 4 = 27.5 → 2 hinges
        assert_eq!(assignments[0].quantity, 2);
    }

    #[test]
    fn test_hinge_count_thresholds() {
        assert_eq!(hinge_count_for_height(20.0), 2);
        assert_eq!(hinge_count_for_height(39.9), 2);
        assert_eq!(hinge_count_for_height(40.0), 2); // <= 40 is 2
        assert_eq!(hinge_count_for_height(40.1), 3);
        assert_eq!(hinge_count_for_height(60.0), 3); // <= 60 is 3
        assert_eq!(hinge_count_for_height(60.1), 4);
        assert_eq!(hinge_count_for_height(80.0), 4);
    }

    #[test]
    fn test_auto_assign_no_shelves() {
        let mut cab = test_cabinet(CabinetType::BasicBox);
        cab.shelf_count = 0;
        let assignments = auto_assign_hardware(&cab);
        assert!(assignments.is_empty(), "no shelves = no hardware for basic box");
    }

    // --- Additional hardware auto-assign edge-case tests ---

    #[test]
    fn test_auto_assign_many_shelves() {
        let mut cab = test_cabinet(CabinetType::BasicBox);
        cab.shelf_count = 10;
        let assignments = auto_assign_hardware(&cab);
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].quantity, 40, "10 shelves × 4 pins = 40");
    }

    #[test]
    fn test_auto_assign_very_tall_doors() {
        // 96" tall cabinet with no toe kick → each door = 48" → 3 hinges per door
        let mut cab = test_cabinet(CabinetType::TallCabinet);
        cab.height = 96.0;
        cab.toe_kick = None;
        let assignments = auto_assign_hardware(&cab);

        // Door height = 96 / 2 = 48 → 3 hinges × 2 doors = 6
        let hinge_assignment = &assignments[0];
        assert!(matches!(hinge_assignment.hardware.kind, HardwareKind::Hinge(_)));
        assert_eq!(hinge_assignment.quantity, 6, "48\" doors should get 3 hinges each");
    }

    #[test]
    fn test_auto_assign_drawer_bank_custom_count() {
        let mut cab = test_cabinet(CabinetType::DrawerBank);
        cab.drawers = Some(DrawerConfig {
            count: 6,
            opening_height: 0.0,
            slide_clearance: 0.5,
        });
        let assignments = auto_assign_hardware(&cab);

        // Slides: 6 pairs, Pulls: 6
        let slide = assignments.iter().find(|a| matches!(a.hardware.kind, HardwareKind::Slide(_))).unwrap();
        let pull = assignments.iter().find(|a| matches!(a.hardware.kind, HardwareKind::Pull(_))).unwrap();
        assert_eq!(slide.quantity, 6);
        assert_eq!(pull.quantity, 6);
    }

    #[test]
    fn test_auto_assign_drawer_bank_no_drawers_field() {
        // DrawerBank with no explicit drawers → should default to 4
        let cab = test_cabinet(CabinetType::DrawerBank);
        let assignments = auto_assign_hardware(&cab);

        let slide = assignments.iter().find(|a| matches!(a.hardware.kind, HardwareKind::Slide(_))).unwrap();
        assert_eq!(slide.quantity, 4, "default drawer count should be 4");
    }

    #[test]
    fn test_auto_assign_wall_cabinet_tall_doors_3_hinges() {
        let mut cab = test_cabinet(CabinetType::WallCabinet);
        cab.height = 42.0; // > 40" → 3 hinges per door
        let assignments = auto_assign_hardware(&cab);

        let hinge = &assignments[0];
        assert_eq!(hinge.quantity, 6, "42\" wall cabinet → 3 hinges × 2 doors = 6");
    }

    #[test]
    fn test_auto_assign_descriptions_not_empty() {
        let cab = test_cabinet(CabinetType::WallCabinet);
        let assignments = auto_assign_hardware(&cab);
        for a in &assignments {
            assert!(!a.description.is_empty(), "description should not be empty");
            assert!(!a.hardware.id.is_empty(), "hardware id should not be empty");
        }
    }

    #[test]
    fn test_auto_assign_all_cabinet_types_no_panic() {
        // Smoke test: auto_assign_hardware should not panic for any cabinet type
        let types = [
            CabinetType::BasicBox,
            CabinetType::BaseCabinet,
            CabinetType::WallCabinet,
            CabinetType::TallCabinet,
            CabinetType::SinkBase,
            CabinetType::DrawerBank,
            CabinetType::CornerCabinet,
            CabinetType::VanityBase,
        ];
        for ct in &types {
            let mut cab = test_cabinet(*ct);
            cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
            if *ct == CabinetType::DrawerBank {
                cab.drawers = Some(DrawerConfig { count: 3, opening_height: 0.0, slide_clearance: 0.5 });
            }
            let assignments = auto_assign_hardware(&cab);
            // Just verify it doesn't panic and produces some result
            // (BasicBox with 2 shelves will have shelf pins, others will have hinges etc.)
            let total_qty: u32 = assignments.iter().map(|a| a.quantity).sum();
            assert!(total_qty > 0 || cab.shelf_count == 0,
                "cabinet type {:?} should have at least some hardware (or 0 shelves)", ct);
        }
    }

    // ---- Phase 16c: Hardware → CAM pipeline tests ----

    #[test]
    fn test_generate_all_hardware_ops_basic_box() {
        let cab = test_cabinet(CabinetType::BasicBox);
        let ops = generate_all_hardware_ops(&cab);

        // BasicBox with 2 shelves → shelf pin holes on both sides
        assert!(!ops.is_empty(), "should generate shelf pin drill ops");

        // All ops should target left_side or right_side
        for op in &ops {
            assert!(
                op.target_part == "left_side" || op.target_part == "right_side",
                "shelf pins should target side panels, got '{}'", op.target_part
            );
            assert!(matches!(op.operation, PartOperation::Drill(_)));
        }

        // Should have ops on both sides
        let left_count = ops.iter().filter(|o| o.target_part == "left_side").count();
        let right_count = ops.iter().filter(|o| o.target_part == "right_side").count();
        assert!(left_count > 0, "should have left_side pins");
        assert_eq!(left_count, right_count, "both sides should have equal pin holes");
    }

    #[test]
    fn test_generate_all_hardware_ops_wall_cabinet() {
        let cab = test_cabinet(CabinetType::WallCabinet);
        let ops = generate_all_hardware_ops(&cab);

        // Wall cabinet: hinge mounting plates + shelf pins
        assert!(!ops.is_empty());

        // Should have some hinge mounting plate ops
        let hinge_ops: Vec<_> = ops.iter()
            .filter(|o| o.description.contains("hinge"))
            .collect();
        assert!(!hinge_ops.is_empty(), "wall cabinet should have hinge mounting plate ops");

        // Should have shelf pin ops
        let pin_ops: Vec<_> = ops.iter()
            .filter(|o| o.description.contains("shelf pin"))
            .collect();
        assert!(!pin_ops.is_empty(), "wall cabinet should have shelf pin ops");
    }

    #[test]
    fn test_generate_all_hardware_ops_drawer_bank() {
        let mut cab = test_cabinet(CabinetType::DrawerBank);
        cab.drawers = Some(DrawerConfig {
            count: 4,
            opening_height: 0.0,
            slide_clearance: 0.5,
        });
        let ops = generate_all_hardware_ops(&cab);

        // Drawer bank: slide holes on both sides
        let slide_ops: Vec<_> = ops.iter()
            .filter(|o| o.description.contains("slide"))
            .collect();
        assert!(!slide_ops.is_empty(), "drawer bank should have slide ops");

        // Both sides should have slide holes
        let left_slides = slide_ops.iter().filter(|o| o.target_part == "left_side").count();
        let right_slides = slide_ops.iter().filter(|o| o.target_part == "right_side").count();
        assert!(left_slides > 0, "should have left_side slide holes");
        assert_eq!(left_slides, right_slides, "both sides should have equal slide holes");
    }

    #[test]
    fn test_hardware_ops_target_correct_labels() {
        // All generated ops should target part labels that exist in the cabinet
        let cab = test_cabinet(CabinetType::WallCabinet);
        let ops = generate_all_hardware_ops(&cab);
        let valid_labels = ["left_side", "right_side", "top", "bottom", "shelf", "back"];

        for op in &ops {
            assert!(
                valid_labels.contains(&op.target_part.as_str()),
                "op targets '{}' which is not a valid part label", op.target_part
            );
        }
    }

    #[test]
    fn test_hinge_y_positions_2_3_4() {
        // 2 hinges: 3" from top and bottom of opening
        let positions = hinge_y_positions(4.0, 26.0, 2);
        assert_eq!(positions.len(), 2);
        assert!((positions[0] - 7.0).abs() < 1e-10, "bottom hinge at 4+3=7");
        assert!((positions[1] - 27.0).abs() < 1e-10, "top hinge at 4+26-3=27");

        // 3 hinges
        let positions = hinge_y_positions(4.0, 26.0, 3);
        assert_eq!(positions.len(), 3);
        assert!((positions[0] - 7.0).abs() < 1e-10);
        assert!((positions[1] - 17.0).abs() < 1e-10, "middle hinge centered");
        assert!((positions[2] - 27.0).abs() < 1e-10);

        // 4 hinges
        let positions = hinge_y_positions(0.0, 60.0, 4);
        assert_eq!(positions.len(), 4);
        assert!((positions[0] - 3.0).abs() < 1e-10);
        assert!((positions[3] - 57.0).abs() < 1e-10);
    }

    #[test]
    fn test_shelf_pin_zone_with_toe_kick() {
        let mut cab = test_cabinet(CabinetType::BaseCabinet);
        cab.height = 34.5;
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });

        let (zone_bottom, zone_top) = shelf_pin_zone(&cab);
        // Bottom = toe_kick (4.0) + material_thickness (0.75) = 4.75
        assert!((zone_bottom - 4.75).abs() < 1e-10, "zone_bottom should be {}, got {}", 4.75, zone_bottom);
        // Top = height (34.5) - material_thickness (0.75) = 33.75
        assert!((zone_top - 33.75).abs() < 1e-10, "zone_top should be {}, got {}", 33.75, zone_top);
    }

    #[test]
    fn test_shelf_pin_zone_without_toe_kick() {
        let cab = test_cabinet(CabinetType::BasicBox);
        let (zone_bottom, zone_top) = shelf_pin_zone(&cab);
        // BasicBox: no toe kick, height=30, mt=0.75
        assert!((zone_bottom - 0.75).abs() < 1e-10);
        assert!((zone_top - 29.25).abs() < 1e-10);
    }

    #[test]
    fn test_drawer_opening_bottoms() {
        let mut cab = test_cabinet(CabinetType::DrawerBank);
        cab.height = 30.0;
        cab.material_thickness = 0.75;
        cab.drawers = Some(DrawerConfig {
            count: 3,
            opening_height: 0.0,
            slide_clearance: 0.5,
        });
        cab.toe_kick = None;

        let bottoms = drawer_opening_bottoms(&cab);
        assert_eq!(bottoms.len(), 3);

        // Interior: 0.75 to 29.25 = 28.5" usable
        // 2 dividers × 0.75 = 1.5
        // Opening height = (28.5 - 1.5) / 3 = 9.0
        assert!((bottoms[0] - 0.75).abs() < 1e-10, "first drawer at bottom");
        assert!((bottoms[1] - (0.75 + 9.0 + 0.75)).abs() < 1e-10, "second drawer");
        assert!((bottoms[2] - (0.75 + 2.0 * (9.0 + 0.75))).abs() < 1e-10, "third drawer");
    }

    #[test]
    fn test_generate_all_hardware_ops_all_types_no_panic() {
        let types = [
            CabinetType::BasicBox,
            CabinetType::BaseCabinet,
            CabinetType::WallCabinet,
            CabinetType::TallCabinet,
            CabinetType::SinkBase,
            CabinetType::DrawerBank,
            CabinetType::CornerCabinet,
            CabinetType::VanityBase,
        ];
        for ct in &types {
            let mut cab = test_cabinet(*ct);
            cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });
            if *ct == CabinetType::DrawerBank {
                cab.drawers = Some(DrawerConfig { count: 3, opening_height: 0.0, slide_clearance: 0.5 });
            }
            let ops = generate_all_hardware_ops(&cab);
            // Just verify it doesn't panic
            for op in &ops {
                assert!(matches!(op.operation, PartOperation::Drill(_)),
                    "all hardware ops should be drills for {:?}", ct);
            }
        }
    }

    #[test]
    fn test_door_opening_height() {
        let mut cab = test_cabinet(CabinetType::BaseCabinet);
        cab.height = 34.5;
        cab.material_thickness = 0.75;
        cab.toe_kick = Some(ToeKickConfig { height: 4.0, setback: 3.0 });

        let doh = door_opening_height(&cab);
        // 34.5 - 4.0 - 2*0.75 = 29.0
        assert!((doh - 29.0).abs() < 1e-10, "door opening height should be 29.0, got {}", doh);
    }
}
