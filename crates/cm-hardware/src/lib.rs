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

pub use catalog::{
    Hardware, HardwareKind, HingeSpec, SlideSpec, ShelfPinSpec, PullSpec,
    ConfirmatSpec, CamLockSpec, EdgeBandSpec, EdgeBandMaterial,
    HardwareApplicator, HardwareOp,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cabinet::cabinet::*;

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
}
