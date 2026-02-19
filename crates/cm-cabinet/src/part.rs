use cm_core::geometry::Rect;
use serde::{Deserialize, Serialize};

/// A single rectangular panel to be cut from sheet goods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    /// Unique label for this part (e.g., "left_side", "shelf_1").
    pub label: String,

    /// The 2D rectangle representing this part (width x height in project units).
    /// Width is along the grain direction.
    pub rect: Rect,

    /// Material thickness (used to determine cut depth for profile cuts).
    pub thickness: f64,

    /// Grain direction matters for nesting on sheet goods.
    #[serde(default)]
    pub grain_direction: GrainDirection,

    /// Operations to perform on this part (dados, rabbets, holes) before profile cutting.
    #[serde(default)]
    pub operations: Vec<PartOperation>,

    /// Quantity needed.
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GrainDirection {
    /// Grain runs along the width (X axis) of the part.
    #[default]
    LengthWise,
    /// Grain runs along the height (Y axis) of the part.
    WidthWise,
}

/// An operation to perform on a part beyond the profile cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PartOperation {
    /// A dado (groove across the grain or along it).
    Dado(DadoOp),
    /// A rabbet (groove along an edge).
    Rabbet(RabbetOp),
    /// A drill hole.
    Drill(DrillOp),
    /// A pocket hole (angled bore for pocket screws).
    PocketHole(PocketHoleOp),
    /// A dovetail joint cut along an edge.
    Dovetail(DovetailOp),
    /// A box (finger) joint cut along an edge.
    BoxJoint(BoxJointOp),
    /// A mortise (rectangular pocket) cut into the face of a panel.
    Mortise(MortiseOp),
    /// A tenon (protruding tongue) formed by shoulder cuts on the end of a part.
    Tenon(TenonOp),
    /// Dowel holes for dowel joinery.
    Dowel(DowelOp),
}

/// A dado cut: a rectangular groove in the face of a panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DadoOp {
    /// Position along the part's height (Y) where the dado center is.
    pub position: f64,
    /// Width of the dado (typically matches mating part thickness).
    pub width: f64,
    /// Depth of the dado (typically half the panel thickness).
    pub depth: f64,
    /// Orientation: does the dado run along the width (horizontal) or height (vertical)?
    #[serde(default)]
    pub orientation: DadoOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DadoOrientation {
    /// Dado runs across the width of the part (most common: shelf dados in side panels).
    #[default]
    Horizontal,
    /// Dado runs along the height of the part.
    Vertical,
}

/// A rabbet cut along an edge of the panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbetOp {
    /// Which edge of the panel.
    pub edge: Edge,
    /// Width of the rabbet.
    pub width: f64,
    /// Depth of the rabbet.
    pub depth: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// A drill operation (single hole).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillOp {
    /// X position of hole center relative to part origin.
    pub x: f64,
    /// Y position of hole center relative to part origin.
    pub y: f64,
    /// Hole diameter.
    pub diameter: f64,
    /// Hole depth.
    pub depth: f64,
}

/// A pocket hole operation (angled bore for pocket screws).
/// Pocket holes are often drilled off-CNC with a Kreg jig, so they
/// appear on the cut list but may not generate CNC toolpaths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocketHoleOp {
    /// X position of hole center relative to part origin.
    pub x: f64,
    /// Y position of hole center relative to part origin.
    pub y: f64,
    /// Which edge the pocket hole is oriented toward.
    pub edge: Edge,
    /// Whether to generate a CNC toolpath for this pocket hole.
    /// When false, the operation appears on the cut list only.
    #[serde(default)]
    pub cnc_operation: bool,
}

/// Dovetail joint style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DovetailStyle {
    /// Through dovetail — tails visible from both faces.
    Through,
    /// Half-blind dovetail — tails hidden on one face (e.g., drawer fronts).
    HalfBlind,
}

/// A dovetail joint cut along an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DovetailOp {
    /// Which edge the dovetails are cut on.
    pub edge: Edge,
    /// Number of tails.
    pub tail_count: u32,
    /// Width of each tail.
    pub tail_width: f64,
    /// Width of each pin (material between tails).
    pub pin_width: f64,
    /// Depth of the joint (how far tails protrude).
    pub depth: f64,
    /// Angle of the dovetail (typically 7-14 degrees; 8 for hardwood, 14 for softwood).
    pub angle: f64,
    /// Through or half-blind dovetail.
    pub style: DovetailStyle,
}

/// A box (finger) joint cut along an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxJointOp {
    /// Which edge the fingers are cut on.
    pub edge: Edge,
    /// Width of each finger.
    pub finger_width: f64,
    /// Depth of each finger (= mating part thickness).
    pub depth: f64,
    /// Number of fingers.
    pub finger_count: u32,
}

/// A mortise (rectangular pocket) cut into the face of a panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortiseOp {
    /// X position of the mortise center.
    pub x: f64,
    /// Y position of the mortise center.
    pub y: f64,
    /// Width of the mortise (along X).
    pub width: f64,
    /// Length of the mortise (along Y).
    pub length: f64,
    /// Depth of the mortise.
    pub depth: f64,
}

/// A tenon formed by shoulder cuts on the end of a part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonOp {
    /// Which edge the tenon protrudes from.
    pub edge: Edge,
    /// Thickness of the tenon.
    pub thickness: f64,
    /// Width of the tenon.
    pub width: f64,
    /// Length (how far the tenon protrudes).
    pub length: f64,
    /// Shoulder depth (how much material is removed around the tenon).
    pub shoulder_depth: f64,
}

/// A single dowel hole position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowelHole {
    /// X position of the dowel hole center.
    pub x: f64,
    /// Y position of the dowel hole center.
    pub y: f64,
}

/// Dowel hole pattern for dowel joinery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowelOp {
    /// Positions of the dowel holes.
    pub holes: Vec<DowelHole>,
    /// Diameter of the dowels.
    pub dowel_diameter: f64,
    /// Depth of each dowel hole.
    pub depth: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_core::geometry::Point2D;

    /// Test serde round-trip for all 9 PartOperation variants via JSON.
    #[test]
    fn test_part_operation_serde_dado() {
        let op = PartOperation::Dado(DadoOp {
            position: 10.0,
            width: 0.75,
            depth: 0.375,
            orientation: DadoOrientation::Horizontal,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"dado"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Dado(d) = op2 {
            assert!((d.position - 10.0).abs() < 1e-10);
            assert!((d.width - 0.75).abs() < 1e-10);
            assert!((d.depth - 0.375).abs() < 1e-10);
            assert_eq!(d.orientation, DadoOrientation::Horizontal);
        } else {
            panic!("expected Dado");
        }
    }

    #[test]
    fn test_part_operation_serde_rabbet() {
        let op = PartOperation::Rabbet(RabbetOp {
            edge: Edge::Right,
            width: 0.25,
            depth: 0.375,
        });
        let json = serde_json::to_string(&op).unwrap();
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Rabbet(r) = op2 {
            assert_eq!(r.edge, Edge::Right);
            assert!((r.width - 0.25).abs() < 1e-10);
        } else {
            panic!("expected Rabbet");
        }
    }

    #[test]
    fn test_part_operation_serde_drill() {
        let op = PartOperation::Drill(DrillOp {
            x: 2.0, y: 5.0, diameter: 0.197, depth: 0.375,
        });
        let json = serde_json::to_string(&op).unwrap();
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Drill(d) = op2 {
            assert!((d.x - 2.0).abs() < 1e-10);
            assert!((d.diameter - 0.197).abs() < 1e-10);
        } else {
            panic!("expected Drill");
        }
    }

    #[test]
    fn test_part_operation_serde_pocket_hole() {
        let op = PartOperation::PocketHole(PocketHoleOp {
            x: 1.0, y: 0.375, edge: Edge::Bottom, cnc_operation: false,
        });
        let json = serde_json::to_string(&op).unwrap();
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::PocketHole(ph) = op2 {
            assert_eq!(ph.edge, Edge::Bottom);
            assert!(!ph.cnc_operation);
        } else {
            panic!("expected PocketHole");
        }
    }

    #[test]
    fn test_part_operation_serde_dovetail() {
        let op = PartOperation::Dovetail(DovetailOp {
            edge: Edge::Left,
            tail_count: 4,
            tail_width: 0.5,
            pin_width: 0.25,
            depth: 0.75,
            angle: 8.0,
            style: DovetailStyle::Through,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"dovetail"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Dovetail(d) = op2 {
            assert_eq!(d.edge, Edge::Left);
            assert_eq!(d.tail_count, 4);
            assert_eq!(d.style, DovetailStyle::Through);
            assert!((d.angle - 8.0).abs() < 1e-10);
        } else {
            panic!("expected Dovetail");
        }
    }

    #[test]
    fn test_part_operation_serde_dovetail_half_blind() {
        let op = PartOperation::Dovetail(DovetailOp {
            edge: Edge::Right,
            tail_count: 3,
            tail_width: 0.6,
            pin_width: 0.3,
            depth: 0.5,
            angle: 14.0,
            style: DovetailStyle::HalfBlind,
        });
        let json = serde_json::to_string(&op).unwrap();
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Dovetail(d) = op2 {
            assert_eq!(d.style, DovetailStyle::HalfBlind);
            assert!((d.angle - 14.0).abs() < 1e-10);
        } else {
            panic!("expected Dovetail");
        }
    }

    #[test]
    fn test_part_operation_serde_box_joint() {
        let op = PartOperation::BoxJoint(BoxJointOp {
            edge: Edge::Bottom,
            finger_width: 0.5,
            depth: 0.75,
            finger_count: 8,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"box_joint"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::BoxJoint(b) = op2 {
            assert_eq!(b.edge, Edge::Bottom);
            assert_eq!(b.finger_count, 8);
            assert!((b.finger_width - 0.5).abs() < 1e-10);
        } else {
            panic!("expected BoxJoint");
        }
    }

    #[test]
    fn test_part_operation_serde_mortise() {
        let op = PartOperation::Mortise(MortiseOp {
            x: 1.0, y: 5.0, width: 0.375, length: 1.0, depth: 1.0,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"mortise"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Mortise(m) = op2 {
            assert!((m.x - 1.0).abs() < 1e-10);
            assert!((m.width - 0.375).abs() < 1e-10);
            assert!((m.depth - 1.0).abs() < 1e-10);
        } else {
            panic!("expected Mortise");
        }
    }

    #[test]
    fn test_part_operation_serde_tenon() {
        let op = PartOperation::Tenon(TenonOp {
            edge: Edge::Left,
            thickness: 0.375,
            width: 1.0,
            length: 1.0,
            shoulder_depth: 0.1875,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"tenon"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Tenon(t) = op2 {
            assert_eq!(t.edge, Edge::Left);
            assert!((t.thickness - 0.375).abs() < 1e-10);
            assert!((t.shoulder_depth - 0.1875).abs() < 1e-10);
        } else {
            panic!("expected Tenon");
        }
    }

    #[test]
    fn test_part_operation_serde_dowel() {
        let op = PartOperation::Dowel(DowelOp {
            holes: vec![
                DowelHole { x: 0.5, y: 1.0 },
                DowelHole { x: 0.5, y: 2.0 },
                DowelHole { x: 0.5, y: 3.0 },
            ],
            dowel_diameter: 0.315,
            depth: 0.5,
        });
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(r#""type":"dowel"#));
        let op2: PartOperation = serde_json::from_str(&json).unwrap();
        if let PartOperation::Dowel(d) = op2 {
            assert_eq!(d.holes.len(), 3);
            assert!((d.dowel_diameter - 0.315).abs() < 1e-10);
            assert!((d.holes[1].y - 2.0).abs() < 1e-10);
        } else {
            panic!("expected Dowel");
        }
    }

    #[test]
    fn test_part_struct_serde_round_trip() {
        let part = Part {
            label: "left_side".to_string(),
            rect: Rect::from_dimensions(12.0, 30.0),
            thickness: 0.75,
            grain_direction: GrainDirection::LengthWise,
            operations: vec![
                PartOperation::Dado(DadoOp {
                    position: 10.0, width: 0.75, depth: 0.375,
                    orientation: DadoOrientation::Horizontal,
                }),
                PartOperation::Rabbet(RabbetOp {
                    edge: Edge::Right, width: 0.25, depth: 0.375,
                }),
                PartOperation::Drill(DrillOp {
                    x: 2.0, y: 5.0, diameter: 0.197, depth: 0.375,
                }),
            ],
            quantity: 2,
        };
        let json = serde_json::to_string_pretty(&part).unwrap();
        let part2: Part = serde_json::from_str(&json).unwrap();

        assert_eq!(part.label, part2.label);
        assert!((part.rect.width - part2.rect.width).abs() < 1e-10);
        assert_eq!(part.operations.len(), part2.operations.len());
        assert_eq!(part.quantity, part2.quantity);
        assert_eq!(part.grain_direction, part2.grain_direction);
    }

    #[test]
    fn test_part_with_advanced_joinery_ops_serde() {
        let part = Part {
            label: "drawer_side".to_string(),
            rect: Rect::from_dimensions(6.0, 4.0),
            thickness: 0.5,
            grain_direction: GrainDirection::WidthWise,
            operations: vec![
                PartOperation::Dovetail(DovetailOp {
                    edge: Edge::Left, tail_count: 3, tail_width: 0.5,
                    pin_width: 0.25, depth: 0.5, angle: 8.0,
                    style: DovetailStyle::Through,
                }),
                PartOperation::BoxJoint(BoxJointOp {
                    edge: Edge::Right, finger_width: 0.25, depth: 0.5, finger_count: 8,
                }),
                PartOperation::Mortise(MortiseOp {
                    x: 1.0, y: 2.0, width: 0.25, length: 0.75, depth: 0.5,
                }),
                PartOperation::Tenon(TenonOp {
                    edge: Edge::Top, thickness: 0.25, width: 0.75, length: 0.5,
                    shoulder_depth: 0.125,
                }),
                PartOperation::Dowel(DowelOp {
                    holes: vec![DowelHole { x: 0.25, y: 1.0 }, DowelHole { x: 0.25, y: 3.0 }],
                    dowel_diameter: 0.315, depth: 0.375,
                }),
            ],
            quantity: 1,
        };
        let json = serde_json::to_string(&part).unwrap();
        let part2: Part = serde_json::from_str(&json).unwrap();

        assert_eq!(part2.operations.len(), 5);
        assert!(matches!(part2.operations[0], PartOperation::Dovetail(_)));
        assert!(matches!(part2.operations[1], PartOperation::BoxJoint(_)));
        assert!(matches!(part2.operations[2], PartOperation::Mortise(_)));
        assert!(matches!(part2.operations[3], PartOperation::Tenon(_)));
        assert!(matches!(part2.operations[4], PartOperation::Dowel(_)));
    }

    #[test]
    fn test_grain_direction_default() {
        assert_eq!(GrainDirection::default(), GrainDirection::LengthWise);
    }

    #[test]
    fn test_dado_orientation_default() {
        assert_eq!(DadoOrientation::default(), DadoOrientation::Horizontal);
    }

    #[test]
    fn test_part_default_quantity() {
        let json = r#"{"label":"test","rect":{"origin":{"x":0,"y":0},"width":10,"height":5},"thickness":0.75}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert_eq!(part.quantity, 1);
    }

    #[test]
    fn test_edge_all_variants_serde() {
        for (edge, expected) in [
            (Edge::Top, "top"),
            (Edge::Bottom, "bottom"),
            (Edge::Left, "left"),
            (Edge::Right, "right"),
        ] {
            let json = serde_json::to_string(&edge).unwrap();
            assert!(json.contains(expected), "Edge::{:?} should serialize as '{}'", edge, expected);
            let edge2: Edge = serde_json::from_str(&json).unwrap();
            assert_eq!(edge, edge2);
        }
    }

    #[test]
    fn test_dovetail_style_serde() {
        for (style, expected) in [
            (DovetailStyle::Through, "through"),
            (DovetailStyle::HalfBlind, "half_blind"),
        ] {
            let json = serde_json::to_string(&style).unwrap();
            assert!(json.contains(expected));
            let style2: DovetailStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(style, style2);
        }
    }
}
