//! Hardware catalog with automatic boring pattern generation.

use cm_cabinet::part::{DrillOp, Edge, PartOperation};
use serde::{Deserialize, Serialize};

/// A hardware item that generates boring patterns on cabinet parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    /// Hardware identifier (e.g., "blum-clip-top-110").
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// The kind of hardware and its boring specification.
    pub kind: HardwareKind,
    /// Unit price in dollars (optional, for BOM costing).
    #[serde(default)]
    pub unit_price: Option<f64>,
    /// Unit weight in ounces (optional, for weight estimation).
    #[serde(default)]
    pub unit_weight_oz: Option<f64>,
}

/// The type-specific boring specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HardwareKind {
    /// European concealed hinge (35mm cup).
    Hinge(HingeSpec),
    /// Drawer slide (side-mount or undermount).
    Slide(SlideSpec),
    /// Shelf pin system.
    ShelfPin(ShelfPinSpec),
    /// Drawer/door pull/knob.
    Pull(PullSpec),
    /// Confirmat (Euro assembly) screw.
    Confirmat(ConfirmatSpec),
    /// Cam lock and bolt fastener.
    CamLock(CamLockSpec),
    /// Edge banding (tracking only — no boring pattern).
    EdgeBand(EdgeBandSpec),
}

/// Specification for European concealed hinges (e.g., Blum Clip Top).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HingeSpec {
    /// Diameter of the cup bore (typically 35mm / 1.378").
    #[serde(default = "default_cup_diameter")]
    pub cup_diameter: f64,
    /// Depth of the cup bore.
    #[serde(default = "default_cup_depth")]
    pub cup_depth: f64,
    /// Distance from door edge to cup bore center (typically 22mm / 0.866").
    #[serde(default = "default_cup_setback")]
    pub cup_setback: f64,
    /// Mounting plate screw hole diameter.
    #[serde(default = "default_screw_hole_diameter")]
    pub mounting_hole_diameter: f64,
    /// Mounting plate screw hole spacing (center to center).
    #[serde(default = "default_mounting_spacing")]
    pub mounting_spacing: f64,
    /// Distance from cabinet front edge to mounting plate center.
    #[serde(default = "default_plate_setback")]
    pub plate_setback: f64,
    /// Opening angle (for documentation).
    #[serde(default = "default_opening_angle")]
    pub opening_angle: f64,
}

fn default_cup_diameter() -> f64 { 1.378 } // 35mm
fn default_cup_depth() -> f64 { 0.5 }
fn default_cup_setback() -> f64 { 0.866 } // 22mm
fn default_screw_hole_diameter() -> f64 { 0.125 } // #8 pilot hole
fn default_mounting_spacing() -> f64 { 1.260 } // 32mm
fn default_plate_setback() -> f64 { 1.457 } // 37mm
fn default_opening_angle() -> f64 { 110.0 }

impl Default for HingeSpec {
    fn default() -> Self {
        Self {
            cup_diameter: default_cup_diameter(),
            cup_depth: default_cup_depth(),
            cup_setback: default_cup_setback(),
            mounting_hole_diameter: default_screw_hole_diameter(),
            mounting_spacing: default_mounting_spacing(),
            plate_setback: default_plate_setback(),
            opening_angle: default_opening_angle(),
        }
    }
}

/// Specification for drawer slides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideSpec {
    /// Slide mounting type.
    #[serde(default)]
    pub mount: SlideMount,
    /// Clearance per side between drawer box and cabinet opening.
    #[serde(default = "default_slide_clearance")]
    pub clearance_per_side: f64,
    /// Screw hole diameter for mounting.
    #[serde(default = "default_screw_hole_diameter")]
    pub screw_hole_diameter: f64,
    /// Screw hole spacing along the slide length.
    #[serde(default = "default_slide_screw_spacing")]
    pub screw_spacing: f64,
    /// Distance from bottom of opening to slide center (side-mount).
    #[serde(default = "default_slide_bottom_offset")]
    pub bottom_offset: f64,
    /// Slide length (matches drawer depth minus overhang).
    #[serde(default)]
    pub length: f64,
}

fn default_slide_clearance() -> f64 { 0.5 }
fn default_slide_screw_spacing() -> f64 { 6.0 }
fn default_slide_bottom_offset() -> f64 { 1.5 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SlideMount {
    /// Slide mounts on the cabinet side panel.
    #[default]
    SideMount,
    /// Slide mounts under the drawer, on the cabinet bottom or stretcher.
    Undermount,
}

impl Default for SlideSpec {
    fn default() -> Self {
        Self {
            mount: SlideMount::SideMount,
            clearance_per_side: default_slide_clearance(),
            screw_hole_diameter: default_screw_hole_diameter(),
            screw_spacing: default_slide_screw_spacing(),
            bottom_offset: default_slide_bottom_offset(),
            length: 0.0,
        }
    }
}

/// Specification for shelf pin system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfPinSpec {
    /// Pin hole diameter (typically 5mm / 0.197").
    #[serde(default = "default_pin_diameter")]
    pub pin_diameter: f64,
    /// Pin hole depth.
    #[serde(default = "default_pin_depth")]
    pub pin_depth: f64,
    /// Vertical spacing between pin rows (32mm system standard).
    #[serde(default = "default_pin_spacing")]
    pub row_spacing: f64,
    /// Distance from front edge to front column of pins.
    #[serde(default = "default_pin_front_setback")]
    pub front_setback: f64,
    /// Distance from rear edge to rear column of pins.
    #[serde(default = "default_pin_rear_setback")]
    pub rear_setback: f64,
    /// Distance from top of adjustable zone to first pin row.
    #[serde(default = "default_pin_margin")]
    pub margin_top: f64,
    /// Distance from bottom of adjustable zone to last pin row.
    #[serde(default = "default_pin_margin")]
    pub margin_bottom: f64,
}

fn default_pin_diameter() -> f64 { 0.197 } // 5mm
fn default_pin_depth() -> f64 { 0.375 }
fn default_pin_spacing() -> f64 { 1.260 } // 32mm
fn default_pin_front_setback() -> f64 { 2.0 }
fn default_pin_rear_setback() -> f64 { 2.0 }
fn default_pin_margin() -> f64 { 2.0 }

impl Default for ShelfPinSpec {
    fn default() -> Self {
        Self {
            pin_diameter: default_pin_diameter(),
            pin_depth: default_pin_depth(),
            row_spacing: default_pin_spacing(),
            front_setback: default_pin_front_setback(),
            rear_setback: default_pin_rear_setback(),
            margin_top: default_pin_margin(),
            margin_bottom: default_pin_margin(),
        }
    }
}

/// Specification for drawer/door pulls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullSpec {
    /// Hole spacing (center to center, for bar pulls). 0 for knobs.
    #[serde(default)]
    pub hole_spacing: f64,
    /// Hole diameter.
    #[serde(default = "default_pull_hole_diameter")]
    pub hole_diameter: f64,
    /// Whether the hole goes through the panel.
    #[serde(default = "default_true")]
    pub through_hole: bool,
}

fn default_pull_hole_diameter() -> f64 { 0.203 } // #10 clearance
fn default_true() -> bool { true }

impl Default for PullSpec {
    fn default() -> Self {
        Self {
            hole_spacing: 3.0,
            hole_diameter: default_pull_hole_diameter(),
            through_hole: true,
        }
    }
}

/// Specification for confirmat (Euro assembly) screws.
///
/// Confirmat screws require a 5mm pilot through the face panel and a 7mm
/// pilot into the edge of the mating panel, plus a 10mm counterbore for
/// the screw head.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmatSpec {
    /// Screw shank diameter (typically 7mm / 0.276").
    #[serde(default = "default_confirmat_shank")]
    pub screw_diameter: f64,
    /// Head diameter for counterbore (typically 10mm / 0.394").
    #[serde(default = "default_confirmat_head")]
    pub head_diameter: f64,
    /// Pilot hole diameter through face panel (typically 5mm / 0.197").
    #[serde(default = "default_confirmat_pilot")]
    pub pilot_diameter: f64,
    /// Counterbore depth for screw head (typically 2mm / 0.079").
    #[serde(default = "default_confirmat_counterbore_depth")]
    pub counterbore_depth: f64,
    /// Screw length / pilot depth into mating panel (typically 50mm / 1.969").
    #[serde(default = "default_confirmat_depth")]
    pub depth: f64,
}

fn default_confirmat_shank() -> f64 { 0.276 } // 7mm
fn default_confirmat_head() -> f64 { 0.394 } // 10mm
fn default_confirmat_pilot() -> f64 { 0.197 } // 5mm
fn default_confirmat_counterbore_depth() -> f64 { 0.079 } // 2mm
fn default_confirmat_depth() -> f64 { 1.969 } // 50mm

impl Default for ConfirmatSpec {
    fn default() -> Self {
        Self {
            screw_diameter: default_confirmat_shank(),
            head_diameter: default_confirmat_head(),
            pilot_diameter: default_confirmat_pilot(),
            counterbore_depth: default_confirmat_counterbore_depth(),
            depth: default_confirmat_depth(),
        }
    }
}

/// Specification for cam lock and bolt fastener (e.g., Rafix, Minifix).
///
/// Cam locks use a 15mm cam housing bored into the face of one panel
/// and a 6mm bolt inserted through the edge of the mating panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CamLockSpec {
    /// Cam housing bore diameter (typically 15mm / 0.591").
    #[serde(default = "default_cam_diameter")]
    pub cam_diameter: f64,
    /// Cam housing bore depth (typically 12.5mm / 0.492").
    #[serde(default = "default_cam_depth")]
    pub cam_depth: f64,
    /// Bolt hole diameter (typically 8mm / 0.315" clearance for 6mm bolt).
    #[serde(default = "default_cam_bolt_hole")]
    pub bolt_hole_diameter: f64,
    /// Bolt length (typically 34mm / 1.339").
    #[serde(default = "default_cam_bolt_length")]
    pub bolt_length: f64,
    /// Distance from panel edge to cam center (typically 34mm / 1.339").
    #[serde(default = "default_cam_setback")]
    pub cam_setback: f64,
}

fn default_cam_diameter() -> f64 { 0.591 } // 15mm
fn default_cam_depth() -> f64 { 0.492 } // 12.5mm
fn default_cam_bolt_hole() -> f64 { 0.315 } // 8mm
fn default_cam_bolt_length() -> f64 { 1.339 } // 34mm
fn default_cam_setback() -> f64 { 1.339 } // 34mm

impl Default for CamLockSpec {
    fn default() -> Self {
        Self {
            cam_diameter: default_cam_diameter(),
            cam_depth: default_cam_depth(),
            bolt_hole_diameter: default_cam_bolt_hole(),
            bolt_length: default_cam_bolt_length(),
            cam_setback: default_cam_setback(),
        }
    }
}

/// Edge banding specification (tracking only — does not generate boring patterns).
///
/// Used for BOM generation and material costing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeBandSpec {
    /// Band thickness (e.g., 0.5mm / 0.020" or 2mm / 0.079").
    #[serde(default = "default_edge_band_thickness")]
    pub thickness: f64,
    /// Material type.
    #[serde(default)]
    pub material_type: EdgeBandMaterial,
    /// Whether the banding is pre-glued (hot air application).
    #[serde(default = "default_true")]
    pub pre_glued: bool,
}

fn default_edge_band_thickness() -> f64 { 0.079 } // 2mm

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeBandMaterial {
    #[default]
    Pvc,
    Abs,
    Wood,
}

impl Default for EdgeBandSpec {
    fn default() -> Self {
        Self {
            thickness: default_edge_band_thickness(),
            material_type: EdgeBandMaterial::Pvc,
            pre_glued: true,
        }
    }
}

/// An operation generated by the hardware applicator.
#[derive(Debug, Clone)]
pub struct HardwareOp {
    /// Target part label.
    pub target_part: String,
    /// The drill operation to add.
    pub operation: PartOperation,
    /// Description of what this hole is for.
    pub description: String,
}

/// Applies hardware boring patterns to cabinet parts.
pub struct HardwareApplicator;

impl HardwareApplicator {
    /// Generate hinge cup bore and mounting plate holes on a door panel.
    ///
    /// Returns operations for one hinge location. Call once per hinge.
    pub fn hinge_bore(
        spec: &HingeSpec,
        part_label: &str,
        part_height: f64,
        hinge_y: f64,
        hinge_edge: Edge,
    ) -> Vec<HardwareOp> {
        let mut ops = Vec::new();

        // Cup bore on the door
        let cup_x = match hinge_edge {
            Edge::Left => spec.cup_setback,
            Edge::Right => part_height - spec.cup_setback, // if mounted on right
            _ => spec.cup_setback,
        };

        ops.push(HardwareOp {
            target_part: part_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x: cup_x,
                y: hinge_y,
                diameter: spec.cup_diameter,
                depth: spec.cup_depth,
            }),
            description: "35mm hinge cup bore".to_string(),
        });

        ops
    }

    /// Generate mounting plate screw holes on the cabinet side panel.
    pub fn hinge_mounting_plate(
        spec: &HingeSpec,
        side_label: &str,
        side_depth: f64,
        hinge_y: f64,
    ) -> Vec<HardwareOp> {
        let mut ops = Vec::new();

        // Two screw holes for mounting plate, spaced vertically
        let x = side_depth - spec.plate_setback;
        let half_spacing = spec.mounting_spacing / 2.0;

        ops.push(HardwareOp {
            target_part: side_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x,
                y: hinge_y - half_spacing,
                diameter: spec.mounting_hole_diameter,
                depth: 0.25,
            }),
            description: "hinge mounting plate screw hole".to_string(),
        });

        ops.push(HardwareOp {
            target_part: side_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x,
                y: hinge_y + half_spacing,
                diameter: spec.mounting_hole_diameter,
                depth: 0.25,
            }),
            description: "hinge mounting plate screw hole".to_string(),
        });

        ops
    }

    /// Generate side-mount drawer slide screw holes on a cabinet side panel.
    pub fn side_mount_slide_holes(
        spec: &SlideSpec,
        side_label: &str,
        opening_bottom_y: f64,
        slide_length: f64,
    ) -> Vec<HardwareOp> {
        let mut ops = Vec::new();

        let y = opening_bottom_y + spec.bottom_offset;
        let num_holes = (slide_length / spec.screw_spacing).floor() as u32;

        for i in 0..num_holes.max(2) {
            let x = spec.screw_spacing * (i + 1) as f64;
            if x > slide_length {
                break;
            }

            ops.push(HardwareOp {
                target_part: side_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x,
                    y,
                    diameter: spec.screw_hole_diameter,
                    depth: 0.25,
                }),
                description: "drawer slide screw hole".to_string(),
            });
        }

        ops
    }

    /// Generate shelf pin holes on a side panel.
    ///
    /// Creates two columns (front and rear) of evenly spaced pin holes
    /// within the adjustable zone.
    pub fn shelf_pin_holes(
        spec: &ShelfPinSpec,
        side_label: &str,
        panel_width: f64,
        zone_bottom_y: f64,
        zone_top_y: f64,
    ) -> Vec<HardwareOp> {
        let mut ops = Vec::new();

        let start_y = zone_bottom_y + spec.margin_bottom;
        let end_y = zone_top_y - spec.margin_top;

        if end_y <= start_y {
            return ops;
        }

        let num_rows = ((end_y - start_y) / spec.row_spacing).floor() as u32 + 1;

        let front_x = spec.front_setback;
        let rear_x = panel_width - spec.rear_setback;

        for i in 0..num_rows {
            let y = start_y + spec.row_spacing * i as f64;
            if y > end_y {
                break;
            }

            // Front column
            ops.push(HardwareOp {
                target_part: side_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x: front_x,
                    y,
                    diameter: spec.pin_diameter,
                    depth: spec.pin_depth,
                }),
                description: "shelf pin hole (front)".to_string(),
            });

            // Rear column
            ops.push(HardwareOp {
                target_part: side_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x: rear_x,
                    y,
                    diameter: spec.pin_diameter,
                    depth: spec.pin_depth,
                }),
                description: "shelf pin hole (rear)".to_string(),
            });
        }

        ops
    }

    /// Generate confirmat screw bore on the face panel (pilot + counterbore).
    ///
    /// The face panel gets a pilot hole through it (for the screw shank) and
    /// a shallow counterbore on the outside face for the screw head.
    pub fn confirmat_bore(
        spec: &ConfirmatSpec,
        face_label: &str,
        x: f64,
        y: f64,
        panel_thickness: f64,
    ) -> Vec<HardwareOp> {
        vec![
            // Pilot hole through face panel
            HardwareOp {
                target_part: face_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x,
                    y,
                    diameter: spec.pilot_diameter,
                    depth: panel_thickness,
                }),
                description: "confirmat pilot hole (through face panel)".to_string(),
            },
            // Counterbore for screw head on outside face
            HardwareOp {
                target_part: face_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x,
                    y,
                    diameter: spec.head_diameter,
                    depth: spec.counterbore_depth,
                }),
                description: "confirmat head counterbore".to_string(),
            },
        ]
    }

    /// Generate confirmat screw pilot in the edge of the mating panel.
    pub fn confirmat_edge_bore(
        spec: &ConfirmatSpec,
        edge_label: &str,
        x: f64,
        y: f64,
    ) -> Vec<HardwareOp> {
        vec![HardwareOp {
            target_part: edge_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x,
                y,
                diameter: spec.screw_diameter,
                depth: spec.depth,
            }),
            description: "confirmat edge pilot hole".to_string(),
        }]
    }

    /// Generate cam lock housing bore on a panel face.
    ///
    /// The cam bore is a 15mm pocket on the face of the panel that houses
    /// the rotating cam. A bolt hole is drilled in the mating panel edge.
    pub fn cam_lock_bore(
        spec: &CamLockSpec,
        face_label: &str,
        x: f64,
        y: f64,
    ) -> Vec<HardwareOp> {
        vec![HardwareOp {
            target_part: face_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x,
                y,
                diameter: spec.cam_diameter,
                depth: spec.cam_depth,
            }),
            description: "cam lock housing bore (15mm)".to_string(),
        }]
    }

    /// Generate cam lock bolt hole in the mating panel edge.
    pub fn cam_lock_bolt_hole(
        spec: &CamLockSpec,
        edge_label: &str,
        x: f64,
        y: f64,
    ) -> Vec<HardwareOp> {
        vec![HardwareOp {
            target_part: edge_label.to_string(),
            operation: PartOperation::Drill(DrillOp {
                x,
                y,
                diameter: spec.bolt_hole_diameter,
                depth: spec.bolt_length,
            }),
            description: "cam lock bolt hole".to_string(),
        }]
    }

    /// Generate pull/knob holes on a door or drawer front.
    pub fn pull_holes(
        spec: &PullSpec,
        part_label: &str,
        part_width: f64,
        part_height: f64,
        pull_center_x: f64,
        pull_center_y: f64,
    ) -> Vec<HardwareOp> {
        let mut ops = Vec::new();
        let depth = if spec.through_hole {
            // Nominal through-hole depth; actual depth set from part thickness
            part_width.min(part_height) // just needs to be > thickness
        } else {
            0.375
        };

        if spec.hole_spacing == 0.0 {
            // Knob: single hole
            ops.push(HardwareOp {
                target_part: part_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x: pull_center_x,
                    y: pull_center_y,
                    diameter: spec.hole_diameter,
                    depth,
                }),
                description: "knob mounting hole".to_string(),
            });
        } else {
            // Bar pull: two holes
            let half = spec.hole_spacing / 2.0;
            ops.push(HardwareOp {
                target_part: part_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x: pull_center_x - half,
                    y: pull_center_y,
                    diameter: spec.hole_diameter,
                    depth,
                }),
                description: "pull mounting hole (left)".to_string(),
            });
            ops.push(HardwareOp {
                target_part: part_label.to_string(),
                operation: PartOperation::Drill(DrillOp {
                    x: pull_center_x + half,
                    y: pull_center_y,
                    diameter: spec.hole_diameter,
                    depth,
                }),
                description: "pull mounting hole (right)".to_string(),
            });
        }

        ops
    }
}

// --- Built-in Hardware Catalog ---

impl Hardware {
    /// Blum Clip Top 110-degree hinge.
    pub fn blum_clip_top_110() -> Self {
        Self {
            id: "blum-clip-top-110".into(),
            description: "Blum Clip Top 110° concealed hinge".into(),
            kind: HardwareKind::Hinge(HingeSpec::default()),
            unit_price: Some(3.50),
            unit_weight_oz: Some(3.2),
        }
    }

    /// Standard 5mm shelf pin system (32mm spacing).
    pub fn shelf_pin_5mm() -> Self {
        Self {
            id: "5mm-shelf-pin".into(),
            description: "5mm shelf pin system (32mm spacing)".into(),
            kind: HardwareKind::ShelfPin(ShelfPinSpec::default()),
            unit_price: Some(0.15),
            unit_weight_oz: Some(0.1),
        }
    }

    /// Standard side-mount drawer slide.
    pub fn side_mount_slide() -> Self {
        Self {
            id: "side-mount-slide".into(),
            description: "Side-mount ball bearing drawer slide".into(),
            kind: HardwareKind::Slide(SlideSpec::default()),
            unit_price: Some(12.00),
            unit_weight_oz: Some(16.0),
        }
    }

    /// Undermount drawer slide (Blum Tandem style).
    pub fn undermount_slide() -> Self {
        Self {
            id: "undermount-slide".into(),
            description: "Undermount full-extension drawer slide".into(),
            kind: HardwareKind::Slide(SlideSpec {
                mount: SlideMount::Undermount,
                clearance_per_side: 0.125,
                ..Default::default()
            }),
            unit_price: Some(35.00),
            unit_weight_oz: Some(24.0),
        }
    }

    /// Standard bar pull (3" center to center).
    pub fn bar_pull_3in() -> Self {
        Self {
            id: "bar-pull-3in".into(),
            description: "3\" center bar pull".into(),
            kind: HardwareKind::Pull(PullSpec {
                hole_spacing: 3.0,
                ..Default::default()
            }),
            unit_price: Some(4.00),
            unit_weight_oz: Some(2.0),
        }
    }

    /// Knob (single hole).
    pub fn knob() -> Self {
        Self {
            id: "knob".into(),
            description: "Cabinet knob".into(),
            kind: HardwareKind::Pull(PullSpec {
                hole_spacing: 0.0,
                ..Default::default()
            }),
            unit_price: Some(2.50),
            unit_weight_oz: Some(1.5),
        }
    }

    /// Standard 7x50mm confirmat screw.
    pub fn confirmat_7x50() -> Self {
        Self {
            id: "confirmat-7x50".into(),
            description: "7x50mm confirmat assembly screw".into(),
            kind: HardwareKind::Confirmat(ConfirmatSpec::default()),
            unit_price: Some(0.25),
            unit_weight_oz: Some(0.3),
        }
    }

    /// 15mm cam lock fastener (Minifix/Rafix style).
    pub fn cam_lock_15mm() -> Self {
        Self {
            id: "cam-lock-15mm".into(),
            description: "15mm cam lock fastener".into(),
            kind: HardwareKind::CamLock(CamLockSpec::default()),
            unit_price: Some(1.50),
            unit_weight_oz: Some(0.5),
        }
    }

    /// 2mm PVC edge banding (pre-glued).
    pub fn pvc_edge_2mm() -> Self {
        Self {
            id: "pvc-edge-2mm".into(),
            description: "2mm PVC edge banding (pre-glued)".into(),
            kind: HardwareKind::EdgeBand(EdgeBandSpec::default()),
            unit_price: Some(0.50),   // per linear foot
            unit_weight_oz: Some(0.3), // per linear foot
        }
    }

    /// 0.5mm PVC edge banding (pre-glued, thin).
    pub fn pvc_edge_thin() -> Self {
        Self {
            id: "pvc-edge-0.5mm".into(),
            description: "0.5mm PVC edge banding (pre-glued)".into(),
            kind: HardwareKind::EdgeBand(EdgeBandSpec {
                thickness: 0.020,
                material_type: EdgeBandMaterial::Pvc,
                pre_glued: true,
            }),
            unit_price: Some(0.30),
            unit_weight_oz: Some(0.1),
        }
    }

    /// Wood veneer edge banding (2mm, not pre-glued).
    pub fn wood_edge_2mm() -> Self {
        Self {
            id: "wood-edge-2mm".into(),
            description: "2mm wood veneer edge banding".into(),
            kind: HardwareKind::EdgeBand(EdgeBandSpec {
                thickness: 0.079,
                material_type: EdgeBandMaterial::Wood,
                pre_glued: false,
            }),
            unit_price: Some(1.00),
            unit_weight_oz: Some(0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hinge_bore_generates_cup() {
        let spec = HingeSpec::default();
        let ops = HardwareApplicator::hinge_bore(&spec, "door", 30.0, 3.0, Edge::Left);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 1.378).abs() < 0.01, "should be 35mm bore");
                assert!((d.x - 0.866).abs() < 0.01, "cup setback should be 22mm");
                assert!((d.y - 3.0).abs() < 1e-10);
            }
            other => panic!("expected Drill, got {:?}", other),
        }
    }

    #[test]
    fn test_hinge_mounting_plate_holes() {
        let spec = HingeSpec::default();
        let ops = HardwareApplicator::hinge_mounting_plate(&spec, "left_side", 24.0, 3.0);
        assert_eq!(ops.len(), 2, "mounting plate should have 2 screw holes");

        for op in &ops {
            match &op.operation {
                PartOperation::Drill(d) => {
                    assert!((d.diameter - 0.125).abs() < 0.01, "pilot hole diameter");
                }
                other => panic!("expected Drill, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_shelf_pin_holes_32mm_spacing() {
        let spec = ShelfPinSpec::default();
        let ops = HardwareApplicator::shelf_pin_holes(
            &spec, "left_side", 12.0, 4.0, 26.0,
        );

        // Zone: 4+2=6 to 26-2=24. Range = 18". At 1.26" spacing = ~15 rows.
        // 2 columns × 15 rows = 30 holes.
        assert!(ops.len() >= 20, "should have many shelf pin holes, got {}", ops.len());

        // All ops should be Drill
        for op in &ops {
            assert!(matches!(op.operation, PartOperation::Drill(_)));
        }

        // Check 32mm spacing between consecutive rows
        let front_holes: Vec<_> = ops.iter()
            .filter(|o| o.description.contains("front"))
            .collect();
        if front_holes.len() >= 2 {
            let y1 = match &front_holes[0].operation {
                PartOperation::Drill(d) => d.y,
                _ => unreachable!(),
            };
            let y2 = match &front_holes[1].operation {
                PartOperation::Drill(d) => d.y,
                _ => unreachable!(),
            };
            assert!((y2 - y1 - spec.row_spacing).abs() < 1e-10, "rows should be 32mm apart");
        }
    }

    #[test]
    fn test_side_mount_slide_holes() {
        let spec = SlideSpec::default();
        let ops = HardwareApplicator::side_mount_slide_holes(
            &spec, "left_side", 4.0, 22.0,
        );
        assert!(!ops.is_empty(), "should generate slide screw holes");

        for op in &ops {
            assert!(matches!(op.operation, PartOperation::Drill(_)));
        }
    }

    #[test]
    fn test_pull_bar_generates_two_holes() {
        let spec = PullSpec {
            hole_spacing: 3.0,
            hole_diameter: 0.203,
            through_hole: true,
        };
        let ops = HardwareApplicator::pull_holes(&spec, "drawer_front", 18.0, 6.0, 9.0, 3.0);
        assert_eq!(ops.len(), 2, "bar pull should generate 2 holes");
    }

    #[test]
    fn test_knob_generates_one_hole() {
        let spec = PullSpec {
            hole_spacing: 0.0,
            hole_diameter: 0.203,
            through_hole: true,
        };
        let ops = HardwareApplicator::pull_holes(&spec, "door", 18.0, 30.0, 17.0, 15.0);
        assert_eq!(ops.len(), 1, "knob should generate 1 hole");
    }

    #[test]
    fn test_builtin_catalog() {
        let hinge = Hardware::blum_clip_top_110();
        assert_eq!(hinge.id, "blum-clip-top-110");
        assert!(matches!(hinge.kind, HardwareKind::Hinge(_)));

        let pins = Hardware::shelf_pin_5mm();
        assert!(matches!(pins.kind, HardwareKind::ShelfPin(_)));

        let slide = Hardware::side_mount_slide();
        assert!(matches!(slide.kind, HardwareKind::Slide(_)));

        let undermount = Hardware::undermount_slide();
        if let HardwareKind::Slide(s) = &undermount.kind {
            assert_eq!(s.mount, SlideMount::Undermount);
        }

        let pull = Hardware::bar_pull_3in();
        if let HardwareKind::Pull(p) = &pull.kind {
            assert!((p.hole_spacing - 3.0).abs() < 1e-10);
        }

        let knob = Hardware::knob();
        if let HardwareKind::Pull(p) = &knob.kind {
            assert!((p.hole_spacing).abs() < 1e-10);
        }
    }

    #[test]
    fn test_hardware_serialization() {
        let hinge = Hardware::blum_clip_top_110();
        let toml = toml::to_string_pretty(&hinge).unwrap();
        let hinge2: Hardware = toml::from_str(&toml).unwrap();
        assert_eq!(hinge.id, hinge2.id);
    }

    #[test]
    fn test_shelf_pin_no_holes_if_zone_too_small() {
        let spec = ShelfPinSpec {
            margin_top: 5.0,
            margin_bottom: 5.0,
            ..Default::default()
        };
        // Zone from 0 to 8, margins eat 10 → no room
        let ops = HardwareApplicator::shelf_pin_holes(&spec, "side", 12.0, 0.0, 8.0);
        assert!(ops.is_empty(), "tiny zone should produce no pin holes");
    }

    // --- Phase 14: New hardware type tests ---

    #[test]
    fn test_confirmat_bore_generates_pilot_and_counterbore() {
        let spec = ConfirmatSpec::default();
        let ops = HardwareApplicator::confirmat_bore(&spec, "top", 6.0, 3.0, 0.75);
        assert_eq!(ops.len(), 2, "confirmat bore should generate pilot + counterbore");

        // First op: pilot hole through face panel
        match &ops[0].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 0.197).abs() < 0.01, "pilot should be 5mm");
                assert!((d.depth - 0.75).abs() < 1e-10, "pilot should go through panel");
            }
            other => panic!("expected Drill, got {:?}", other),
        }

        // Second op: counterbore for screw head
        match &ops[1].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 0.394).abs() < 0.01, "counterbore should be 10mm");
                assert!((d.depth - 0.079).abs() < 0.01, "counterbore depth should be 2mm");
            }
            other => panic!("expected Drill, got {:?}", other),
        }
    }

    #[test]
    fn test_confirmat_edge_bore() {
        let spec = ConfirmatSpec::default();
        let ops = HardwareApplicator::confirmat_edge_bore(&spec, "shelf", 6.0, 0.375);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 0.276).abs() < 0.01, "edge bore should be 7mm");
                assert!((d.depth - 1.969).abs() < 0.01, "depth should be 50mm");
            }
            other => panic!("expected Drill, got {:?}", other),
        }
    }

    #[test]
    fn test_cam_lock_bore_diameter() {
        let spec = CamLockSpec::default();
        let ops = HardwareApplicator::cam_lock_bore(&spec, "side", 1.339, 6.0);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 0.591).abs() < 0.01, "cam bore should be 15mm");
                assert!((d.depth - 0.492).abs() < 0.01, "cam depth should be 12.5mm");
            }
            other => panic!("expected Drill, got {:?}", other),
        }
    }

    #[test]
    fn test_cam_lock_bolt_hole() {
        let spec = CamLockSpec::default();
        let ops = HardwareApplicator::cam_lock_bolt_hole(&spec, "shelf_edge", 6.0, 0.375);
        assert_eq!(ops.len(), 1);

        match &ops[0].operation {
            PartOperation::Drill(d) => {
                assert!((d.diameter - 0.315).abs() < 0.01, "bolt hole should be 8mm");
                assert!((d.depth - 1.339).abs() < 0.01, "bolt depth should be 34mm");
            }
            other => panic!("expected Drill, got {:?}", other),
        }
    }

    #[test]
    fn test_confirmat_spec_defaults() {
        let spec = ConfirmatSpec::default();
        assert!((spec.screw_diameter - 0.276).abs() < 0.01);
        assert!((spec.head_diameter - 0.394).abs() < 0.01);
        assert!((spec.pilot_diameter - 0.197).abs() < 0.01);
        assert!((spec.depth - 1.969).abs() < 0.01);
    }

    #[test]
    fn test_cam_lock_spec_defaults() {
        let spec = CamLockSpec::default();
        assert!((spec.cam_diameter - 0.591).abs() < 0.01);
        assert!((spec.cam_depth - 0.492).abs() < 0.01);
        assert!((spec.bolt_hole_diameter - 0.315).abs() < 0.01);
        assert!((spec.cam_setback - 1.339).abs() < 0.01);
    }

    #[test]
    fn test_edge_band_spec_defaults() {
        let spec = EdgeBandSpec::default();
        assert!((spec.thickness - 0.079).abs() < 0.01);
        assert_eq!(spec.material_type, EdgeBandMaterial::Pvc);
        assert!(spec.pre_glued);
    }

    #[test]
    fn test_builtin_confirmat() {
        let hw = Hardware::confirmat_7x50();
        assert_eq!(hw.id, "confirmat-7x50");
        assert!(matches!(hw.kind, HardwareKind::Confirmat(_)));
    }

    #[test]
    fn test_builtin_cam_lock() {
        let hw = Hardware::cam_lock_15mm();
        assert_eq!(hw.id, "cam-lock-15mm");
        assert!(matches!(hw.kind, HardwareKind::CamLock(_)));
    }

    #[test]
    fn test_builtin_edge_bands() {
        let thick = Hardware::pvc_edge_2mm();
        assert!(matches!(thick.kind, HardwareKind::EdgeBand(_)));
        if let HardwareKind::EdgeBand(spec) = &thick.kind {
            assert!((spec.thickness - 0.079).abs() < 0.01);
            assert!(spec.pre_glued);
        }

        let thin = Hardware::pvc_edge_thin();
        if let HardwareKind::EdgeBand(spec) = &thin.kind {
            assert!((spec.thickness - 0.020).abs() < 0.01);
        }

        let wood = Hardware::wood_edge_2mm();
        if let HardwareKind::EdgeBand(spec) = &wood.kind {
            assert_eq!(spec.material_type, EdgeBandMaterial::Wood);
            assert!(!spec.pre_glued);
        }
    }

    #[test]
    fn test_confirmat_serialization() {
        let hw = Hardware::confirmat_7x50();
        let toml_str = toml::to_string_pretty(&hw).unwrap();
        let hw2: Hardware = toml::from_str(&toml_str).unwrap();
        assert_eq!(hw.id, hw2.id);
        assert!(matches!(hw2.kind, HardwareKind::Confirmat(_)));
    }

    #[test]
    fn test_cam_lock_serialization() {
        let hw = Hardware::cam_lock_15mm();
        let toml_str = toml::to_string_pretty(&hw).unwrap();
        let hw2: Hardware = toml::from_str(&toml_str).unwrap();
        assert_eq!(hw.id, hw2.id);
        assert!(matches!(hw2.kind, HardwareKind::CamLock(_)));
    }

    #[test]
    fn test_edge_band_serialization() {
        let hw = Hardware::pvc_edge_2mm();
        let toml_str = toml::to_string_pretty(&hw).unwrap();
        let hw2: Hardware = toml::from_str(&toml_str).unwrap();
        assert_eq!(hw.id, hw2.id);
        assert!(matches!(hw2.kind, HardwareKind::EdgeBand(_)));
    }

    // --- Phase 16d: unit_price and unit_weight_oz tests ---

    #[test]
    fn test_all_builtin_hardware_have_prices() {
        let items: Vec<Hardware> = vec![
            Hardware::blum_clip_top_110(),
            Hardware::shelf_pin_5mm(),
            Hardware::side_mount_slide(),
            Hardware::undermount_slide(),
            Hardware::bar_pull_3in(),
            Hardware::knob(),
            Hardware::confirmat_7x50(),
            Hardware::cam_lock_15mm(),
            Hardware::pvc_edge_2mm(),
            Hardware::pvc_edge_thin(),
            Hardware::wood_edge_2mm(),
        ];
        for hw in &items {
            assert!(hw.unit_price.is_some(), "{} should have a unit price", hw.id);
            assert!(hw.unit_price.unwrap() > 0.0, "{} price should be positive", hw.id);
        }
    }

    #[test]
    fn test_all_builtin_hardware_have_weights() {
        let items: Vec<Hardware> = vec![
            Hardware::blum_clip_top_110(),
            Hardware::shelf_pin_5mm(),
            Hardware::side_mount_slide(),
            Hardware::undermount_slide(),
            Hardware::bar_pull_3in(),
            Hardware::knob(),
            Hardware::confirmat_7x50(),
            Hardware::cam_lock_15mm(),
            Hardware::pvc_edge_2mm(),
            Hardware::pvc_edge_thin(),
            Hardware::wood_edge_2mm(),
        ];
        for hw in &items {
            assert!(hw.unit_weight_oz.is_some(), "{} should have a unit weight", hw.id);
            assert!(hw.unit_weight_oz.unwrap() > 0.0, "{} weight should be positive", hw.id);
        }
    }

    #[test]
    fn test_hardware_price_values() {
        assert!((Hardware::blum_clip_top_110().unit_price.unwrap() - 3.50).abs() < 1e-10);
        assert!((Hardware::shelf_pin_5mm().unit_price.unwrap() - 0.15).abs() < 1e-10);
        assert!((Hardware::side_mount_slide().unit_price.unwrap() - 12.00).abs() < 1e-10);
        assert!((Hardware::undermount_slide().unit_price.unwrap() - 35.00).abs() < 1e-10);
        assert!((Hardware::bar_pull_3in().unit_price.unwrap() - 4.00).abs() < 1e-10);
        assert!((Hardware::knob().unit_price.unwrap() - 2.50).abs() < 1e-10);
        assert!((Hardware::confirmat_7x50().unit_price.unwrap() - 0.25).abs() < 1e-10);
        assert!((Hardware::cam_lock_15mm().unit_price.unwrap() - 1.50).abs() < 1e-10);
    }

    #[test]
    fn test_hardware_weight_values() {
        assert!((Hardware::blum_clip_top_110().unit_weight_oz.unwrap() - 3.2).abs() < 1e-10);
        assert!((Hardware::shelf_pin_5mm().unit_weight_oz.unwrap() - 0.1).abs() < 1e-10);
        assert!((Hardware::side_mount_slide().unit_weight_oz.unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn test_hardware_price_serialization_round_trip() {
        let hw = Hardware::blum_clip_top_110();
        let json = serde_json::to_string(&hw).unwrap();
        let hw2: Hardware = serde_json::from_str(&json).unwrap();
        assert_eq!(hw.unit_price, hw2.unit_price);
        assert_eq!(hw.unit_weight_oz, hw2.unit_weight_oz);
    }

    #[test]
    fn test_hardware_no_price_defaults_to_none() {
        // Deserializing hardware without price/weight should give None
        let json = r#"{"id":"custom","description":"custom item","kind":{"type":"shelf_pin","pin_diameter":0.197,"pin_depth":0.375,"row_spacing":1.26,"front_setback":2.0,"rear_setback":2.0,"margin_top":2.0,"margin_bottom":2.0}}"#;
        let hw: Hardware = serde_json::from_str(json).unwrap();
        assert_eq!(hw.unit_price, None);
        assert_eq!(hw.unit_weight_oz, None);
    }
}
