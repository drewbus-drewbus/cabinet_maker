//! Joinery rules: maps joint declarations to concrete operations.

use cm_cabinet::part::{DadoOp, DadoOrientation, Edge, PartOperation, RabbetOp, PocketHoleOp};
use cm_core::material::MaterialType;
use serde::{Deserialize, Serialize};

/// A declared joint between two parts (before joinery method is resolved).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// Label of the part that receives the joint operation (e.g., "left_side").
    pub target_part: String,
    /// Label of the mating part (e.g., "shelf").
    pub mating_part: String,
    /// What kind of connection this is.
    pub kind: JointKind,
    /// Where on the target part the joint occurs.
    pub position: JointPosition,
    /// Thickness of the mating part (determines dado/rabbet width).
    pub mating_thickness: f64,
}

/// The type of structural connection between two parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointKind {
    /// A horizontal panel (shelf, top, bottom) meeting a side panel.
    ShelfToSide,
    /// Back panel meeting carcass edges.
    BackToCarcase,
    /// Face-frame rail meeting stile.
    RailToStile,
    /// Drawer box corner joint.
    DrawerCorner,
    /// Stretcher rail meeting side panel.
    StretcherToSide,
    /// Divider panel meeting side panel (like ShelfToSide but for drawer dividers).
    DividerToSide,
    /// Generic butt joint (no machining, just glue/screws).
    Butt,
}

/// Where on the target part the joint occurs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointPosition {
    /// Joint runs horizontally across the part at a given Y position.
    Horizontal(f64),
    /// Joint runs vertically along the part at a given X position.
    Vertical(f64),
    /// Joint is on a specific edge of the part.
    Edge(Edge),
}

/// A joinery method — the physical technique used to create a joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoineryMethod {
    /// Dado groove (slot across the face).
    Dado,
    /// Rabbet (groove along an edge).
    Rabbet,
    /// Lock rabbet (interlocking rabbet — good for drawer boxes).
    LockRabbet,
    /// Pocket hole screw joint.
    PocketHole,
    /// Simple butt joint (no CNC machining needed).
    Butt,
    /// Biscuit joint (slot for #20 biscuit).
    Biscuit,
    /// Dowel joint (holes for dowel pins).
    Dowel,
}

/// A rule that maps a joint kind + material type to a joinery method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoineryRule {
    /// Which joint kind this rule applies to.
    pub joint_kind: JointKind,
    /// Material type filter (None = applies to all materials).
    #[serde(default)]
    pub material_type: Option<MaterialType>,
    /// The joinery method to use.
    pub method: JoineryMethod,
    /// Depth as fraction of target part thickness (for dados/rabbets).
    #[serde(default = "default_depth_fraction")]
    pub depth_fraction: f64,
    /// Whether this operation should be CNC-machined.
    #[serde(default = "default_true")]
    pub cnc_operation: bool,
}

fn default_depth_fraction() -> f64 {
    0.5
}

fn default_true() -> bool {
    true
}

/// A resolved operation to be added to a part.
#[derive(Debug, Clone)]
pub struct ResolvedOperation {
    /// Which part label receives this operation.
    pub target_part: String,
    /// The operation to add.
    pub operation: PartOperation,
}

/// A complete set of joinery rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoineryRuleset {
    /// Ordered list of rules. First matching rule wins.
    pub rules: Vec<JoineryRule>,
}

impl Default for JoineryRuleset {
    /// Standard woodworking joinery defaults.
    fn default() -> Self {
        Self {
            rules: vec![
                // Shelf/top/bottom to side → dado in plywood
                JoineryRule {
                    joint_kind: JointKind::ShelfToSide,
                    material_type: Some(MaterialType::Plywood),
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Shelf to side in MDF → dado
                JoineryRule {
                    joint_kind: JointKind::ShelfToSide,
                    material_type: Some(MaterialType::Mdf),
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Shelf to side in hardwood → dado (shallower)
                JoineryRule {
                    joint_kind: JointKind::ShelfToSide,
                    material_type: Some(MaterialType::Hardwood),
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.333,
                    cnc_operation: true,
                },
                // Shelf to side in any other material → dado (default)
                JoineryRule {
                    joint_kind: JointKind::ShelfToSide,
                    material_type: None,
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Divider to side → dado
                JoineryRule {
                    joint_kind: JointKind::DividerToSide,
                    material_type: None,
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Back to carcass → rabbet
                JoineryRule {
                    joint_kind: JointKind::BackToCarcase,
                    material_type: None,
                    method: JoineryMethod::Rabbet,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Rail to stile (face frame) → pocket hole
                JoineryRule {
                    joint_kind: JointKind::RailToStile,
                    material_type: None,
                    method: JoineryMethod::PocketHole,
                    depth_fraction: 0.0,
                    cnc_operation: false, // typically off-CNC with Kreg jig
                },
                // Drawer corner → rabbet
                JoineryRule {
                    joint_kind: JointKind::DrawerCorner,
                    material_type: Some(MaterialType::Plywood),
                    method: JoineryMethod::Rabbet,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Drawer corner in hardwood → lock rabbet
                JoineryRule {
                    joint_kind: JointKind::DrawerCorner,
                    material_type: Some(MaterialType::Hardwood),
                    method: JoineryMethod::LockRabbet,
                    depth_fraction: 0.333,
                    cnc_operation: true,
                },
                // Drawer corner default → rabbet
                JoineryRule {
                    joint_kind: JointKind::DrawerCorner,
                    material_type: None,
                    method: JoineryMethod::Rabbet,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Stretcher to side → dado
                JoineryRule {
                    joint_kind: JointKind::StretcherToSide,
                    material_type: None,
                    method: JoineryMethod::Dado,
                    depth_fraction: 0.5,
                    cnc_operation: true,
                },
                // Butt joint → no operation
                JoineryRule {
                    joint_kind: JointKind::Butt,
                    material_type: None,
                    method: JoineryMethod::Butt,
                    depth_fraction: 0.0,
                    cnc_operation: false,
                },
            ],
        }
    }
}

impl JoineryRuleset {
    /// Load a ruleset from a TOML string (user override).
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serialize the ruleset to TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Find the first matching rule for a given joint kind and material type.
    pub fn find_rule(&self, kind: JointKind, material_type: MaterialType) -> Option<&JoineryRule> {
        self.rules.iter().find(|r| {
            r.joint_kind == kind
                && (r.material_type.is_none() || r.material_type == Some(material_type))
        })
    }

    /// Resolve a list of joints into concrete part operations.
    pub fn resolve(
        &self,
        joints: &[Joint],
        target_thickness: f64,
        material_type: MaterialType,
    ) -> Vec<ResolvedOperation> {
        let mut ops = Vec::new();

        for joint in joints {
            let Some(rule) = self.find_rule(joint.kind, material_type) else {
                continue;
            };

            let depth = target_thickness * rule.depth_fraction;

            match rule.method {
                JoineryMethod::Dado => {
                    let (position, orientation) = match joint.position {
                        JointPosition::Horizontal(y) => (y, DadoOrientation::Horizontal),
                        JointPosition::Vertical(x) => (x, DadoOrientation::Vertical),
                        JointPosition::Edge(_) => continue, // dados don't go on edges
                    };
                    ops.push(ResolvedOperation {
                        target_part: joint.target_part.clone(),
                        operation: PartOperation::Dado(DadoOp {
                            position,
                            width: joint.mating_thickness,
                            depth,
                            orientation,
                        }),
                    });
                }
                JoineryMethod::Rabbet | JoineryMethod::LockRabbet => {
                    let edge = match joint.position {
                        JointPosition::Edge(e) => e,
                        // If position is horizontal/vertical, infer edge
                        JointPosition::Horizontal(_) => Edge::Bottom,
                        JointPosition::Vertical(_) => Edge::Right,
                    };
                    ops.push(ResolvedOperation {
                        target_part: joint.target_part.clone(),
                        operation: PartOperation::Rabbet(RabbetOp {
                            edge,
                            width: joint.mating_thickness,
                            depth,
                        }),
                    });
                }
                JoineryMethod::PocketHole => {
                    let edge = match joint.position {
                        JointPosition::Edge(e) => e,
                        JointPosition::Horizontal(_) => Edge::Bottom,
                        JointPosition::Vertical(_) => Edge::Left,
                    };
                    // Pocket holes go on the mating part (rail), not the target (stile)
                    ops.push(ResolvedOperation {
                        target_part: joint.mating_part.clone(),
                        operation: PartOperation::PocketHole(PocketHoleOp {
                            x: 0.5, // near end of rail
                            y: joint.mating_thickness / 2.0,
                            edge,
                            cnc_operation: rule.cnc_operation,
                        }),
                    });
                }
                JoineryMethod::Butt | JoineryMethod::Biscuit | JoineryMethod::Dowel => {
                    // No CNC operations for butt joints.
                    // Biscuit/dowel could generate slot/drill ops in the future.
                }
            }
        }

        ops
    }
}

impl Joint {
    /// Convenience: shelf-to-side joint at a horizontal position.
    pub fn shelf_to_side(
        target: &str,
        mating: &str,
        y_position: f64,
        mating_thickness: f64,
    ) -> Self {
        Self {
            target_part: target.to_string(),
            mating_part: mating.to_string(),
            kind: JointKind::ShelfToSide,
            position: JointPosition::Horizontal(y_position),
            mating_thickness,
        }
    }

    /// Convenience: back-to-carcass joint on a given edge.
    pub fn back_to_carcass(
        target: &str,
        mating: &str,
        edge: Edge,
        mating_thickness: f64,
    ) -> Self {
        Self {
            target_part: target.to_string(),
            mating_part: mating.to_string(),
            kind: JointKind::BackToCarcase,
            position: JointPosition::Edge(edge),
            mating_thickness,
        }
    }

    /// Convenience: face-frame rail to stile.
    pub fn rail_to_stile(
        stile: &str,
        rail: &str,
        mating_thickness: f64,
    ) -> Self {
        Self {
            target_part: stile.to_string(),
            mating_part: rail.to_string(),
            kind: JointKind::RailToStile,
            position: JointPosition::Edge(Edge::Right),
            mating_thickness,
        }
    }

    /// Convenience: drawer corner joint.
    pub fn drawer_corner(
        target: &str,
        mating: &str,
        edge: Edge,
        mating_thickness: f64,
    ) -> Self {
        Self {
            target_part: target.to_string(),
            mating_part: mating.to_string(),
            kind: JointKind::DrawerCorner,
            position: JointPosition::Edge(edge),
            mating_thickness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ruleset_has_rules() {
        let rules = JoineryRuleset::default();
        assert!(!rules.rules.is_empty());
    }

    #[test]
    fn test_shelf_to_side_resolves_dado_in_plywood() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::shelf_to_side("left_side", "shelf", 10.0, 0.75),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Plywood);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].target_part, "left_side");

        match &ops[0].operation {
            PartOperation::Dado(d) => {
                assert!((d.position - 10.0).abs() < 1e-10);
                assert!((d.width - 0.75).abs() < 1e-10);
                assert!((d.depth - 0.375).abs() < 1e-10); // 50% of 0.75
                assert_eq!(d.orientation, DadoOrientation::Horizontal);
            }
            other => panic!("expected Dado, got {:?}", other),
        }
    }

    #[test]
    fn test_shelf_to_side_hardwood_shallower_dado() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::shelf_to_side("left_side", "shelf", 10.0, 0.75),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Hardwood);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Dado(d) => {
                // Hardwood: 33.3% depth
                assert!((d.depth - 0.75 * 0.333).abs() < 1e-3);
            }
            other => panic!("expected Dado, got {:?}", other),
        }
    }

    #[test]
    fn test_back_to_carcass_resolves_rabbet() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::back_to_carcass("left_side", "back", Edge::Right, 0.25),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Plywood);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Rabbet(r) => {
                assert_eq!(r.edge, Edge::Right);
                assert!((r.width - 0.25).abs() < 1e-10);
                assert!((r.depth - 0.375).abs() < 1e-10);
            }
            other => panic!("expected Rabbet, got {:?}", other),
        }
    }

    #[test]
    fn test_rail_to_stile_resolves_pocket_hole() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::rail_to_stile("stile", "rail", 1.5),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Hardwood);
        assert_eq!(ops.len(), 1);
        // Pocket hole goes on the mating part (rail), not the target (stile)
        assert_eq!(ops[0].target_part, "rail");

        match &ops[0].operation {
            PartOperation::PocketHole(ph) => {
                assert!(!ph.cnc_operation, "face frame pocket holes default to off-CNC");
            }
            other => panic!("expected PocketHole, got {:?}", other),
        }
    }

    #[test]
    fn test_drawer_corner_plywood_rabbet() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::drawer_corner("drawer_front", "drawer_side", Edge::Left, 0.5),
        ];

        let ops = rules.resolve(&joints, 0.5, MaterialType::Plywood);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Rabbet(r) => {
                assert_eq!(r.edge, Edge::Left);
            }
            other => panic!("expected Rabbet, got {:?}", other),
        }
    }

    #[test]
    fn test_drawer_corner_hardwood_lock_rabbet() {
        let rules = JoineryRuleset::default();
        let rule = rules.find_rule(JointKind::DrawerCorner, MaterialType::Hardwood).unwrap();
        assert_eq!(rule.method, JoineryMethod::LockRabbet);
    }

    #[test]
    fn test_butt_joint_produces_no_operations() {
        let rules = JoineryRuleset::default();
        let joints = vec![Joint {
            target_part: "side".into(),
            mating_part: "shelf".into(),
            kind: JointKind::Butt,
            position: JointPosition::Horizontal(10.0),
            mating_thickness: 0.75,
        }];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Plywood);
        assert!(ops.is_empty(), "butt joint should not generate CNC operations");
    }

    #[test]
    fn test_multiple_joints_resolve_correctly() {
        let rules = JoineryRuleset::default();
        let joints = vec![
            Joint::shelf_to_side("left_side", "bottom", 0.375, 0.75),
            Joint::shelf_to_side("left_side", "top", 29.625, 0.75),
            Joint::shelf_to_side("left_side", "shelf", 15.0, 0.75),
            Joint::back_to_carcass("left_side", "back", Edge::Right, 0.25),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Plywood);
        assert_eq!(ops.len(), 4);

        let dados: Vec<_> = ops.iter()
            .filter(|o| matches!(o.operation, PartOperation::Dado(_)))
            .collect();
        assert_eq!(dados.len(), 3);

        let rabbets: Vec<_> = ops.iter()
            .filter(|o| matches!(o.operation, PartOperation::Rabbet(_)))
            .collect();
        assert_eq!(rabbets.len(), 1);
    }

    #[test]
    fn test_ruleset_serialization_round_trip() {
        let rules = JoineryRuleset::default();
        let toml = rules.to_toml().unwrap();
        let rules2 = JoineryRuleset::from_toml(&toml).unwrap();
        assert_eq!(rules.rules.len(), rules2.rules.len());
    }

    #[test]
    fn test_custom_ruleset() {
        // User can override defaults
        let rules = JoineryRuleset {
            rules: vec![
                JoineryRule {
                    joint_kind: JointKind::ShelfToSide,
                    material_type: None,
                    method: JoineryMethod::Butt,
                    depth_fraction: 0.0,
                    cnc_operation: false,
                },
            ],
        };

        let joints = vec![
            Joint::shelf_to_side("side", "shelf", 10.0, 0.75),
        ];

        let ops = rules.resolve(&joints, 0.75, MaterialType::Plywood);
        assert!(ops.is_empty(), "butt joint rule should suppress dado");
    }
}
