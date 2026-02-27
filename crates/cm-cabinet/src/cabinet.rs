use cm_core::geometry::{Point2D, Rect};
use serde::{Deserialize, Serialize};

use crate::part::{DadoOp, DadoOrientation, Part, PartOperation, PocketHoleOp, RabbetOp, Edge};

/// A parametric cabinet definition. Change any parameter and all parts
/// are regenerated automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cabinet {
    /// Human-readable name for this cabinet.
    pub name: String,

    /// Cabinet type.
    #[serde(default)]
    pub cabinet_type: CabinetType,

    /// Overall outer width.
    pub width: f64,

    /// Overall outer height.
    pub height: f64,

    /// Overall outer depth (front to back).
    pub depth: f64,

    /// Carcass material thickness.
    pub material_thickness: f64,

    /// Back panel material thickness (typically thinner, e.g., 1/4").
    #[serde(default = "default_back_thickness")]
    pub back_thickness: f64,

    /// Number of adjustable/fixed shelves.
    #[serde(default)]
    pub shelf_count: u32,

    /// Joinery method for shelves.
    #[serde(default)]
    pub shelf_joinery: ShelfJoinery,

    /// Dado depth as fraction of material thickness (default 0.5 = half thickness).
    #[serde(default = "default_dado_depth_fraction")]
    pub dado_depth_fraction: f64,

    /// Whether to include a back panel.
    #[serde(default = "default_true")]
    pub has_back: bool,

    /// Back panel joinery method.
    #[serde(default)]
    pub back_joinery: BackJoinery,

    /// Toe kick configuration (for base cabinets, tall cabinets, etc.).
    #[serde(default)]
    pub toe_kick: Option<ToeKickConfig>,

    /// Drawer configuration (for drawer bank cabinets).
    #[serde(default)]
    pub drawers: Option<DrawerConfig>,

    /// Stretcher configuration (for sink bases and cabinets with stretcher rails).
    #[serde(default)]
    pub stretchers: Option<StretcherConfig>,

    /// Construction method (frameless/Euro or face-frame).
    #[serde(default)]
    pub construction: ConstructionMethod,

    /// Face-frame configuration (only used when construction = FaceFrame).
    #[serde(default)]
    pub face_frame: Option<FaceFrameConfig>,

    /// Corner cabinet type (only used when cabinet_type = CornerCabinet).
    #[serde(default)]
    pub corner_type: Option<CornerType>,

    /// Plumbing cutout (only used for VanityBase / SinkBase).
    #[serde(default)]
    pub plumbing_cutout: Option<PlumbingCutout>,
}

fn default_back_thickness() -> f64 {
    0.25
}

fn default_dado_depth_fraction() -> f64 {
    0.5
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CabinetType {
    /// Simple open box (like a bookshelf).
    #[default]
    BasicBox,
    /// Base cabinet with toe kick (standard kitchen lower cabinet).
    BaseCabinet,
    /// Wall cabinet (shallower, no toe kick, upper kitchen cabinet).
    WallCabinet,
    /// Tall cabinet (pantry, utility — 84" or 96" tall).
    TallCabinet,
    /// Sink base (no bottom shelf, open front, stretchers).
    SinkBase,
    /// Drawer bank (base cabinet with drawer box slots).
    DrawerBank,
    /// L-shaped corner cabinet (36"x36" standard footprint).
    CornerCabinet,
    /// Bathroom vanity base (31.5" H, 21" D standard).
    VanityBase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShelfJoinery {
    /// Shelves sit in dado grooves cut into the sides.
    #[default]
    Dado,
    /// Shelves are simply butt-joined (screwed/nailed).
    Butt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackJoinery {
    /// Back sits in a rabbet along the rear edges.
    #[default]
    Rabbet,
    /// Back is simply nailed/screwed onto the back.
    NailedOn,
}

/// Toe kick configuration for base and tall cabinets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToeKickConfig {
    /// Height of the toe kick (typically 4.0").
    #[serde(default = "default_toe_kick_height")]
    pub height: f64,
    /// Setback (depth) of the toe kick from the cabinet face (typically 3.0").
    #[serde(default = "default_toe_kick_setback")]
    pub setback: f64,
}

fn default_toe_kick_height() -> f64 {
    4.0
}

fn default_toe_kick_setback() -> f64 {
    3.0
}

impl Default for ToeKickConfig {
    fn default() -> Self {
        Self {
            height: default_toe_kick_height(),
            setback: default_toe_kick_setback(),
        }
    }
}

/// Drawer configuration for drawer bank cabinets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrawerConfig {
    /// Number of drawer openings.
    #[serde(default = "default_drawer_count")]
    pub count: u32,
    /// Height of each drawer opening (if uniform). 0 = auto-calculate evenly.
    #[serde(default)]
    pub opening_height: f64,
    /// Clearance per side for drawer slides (typically 0.5" per side).
    #[serde(default = "default_slide_clearance")]
    pub slide_clearance: f64,
}

fn default_drawer_count() -> u32 {
    4
}

fn default_slide_clearance() -> f64 {
    0.5
}

impl Default for DrawerConfig {
    fn default() -> Self {
        Self {
            count: default_drawer_count(),
            opening_height: 0.0,
            slide_clearance: default_slide_clearance(),
        }
    }
}

/// Stretcher configuration for cabinets that use stretcher rails instead
/// of a full top panel (e.g., sink bases).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StretcherConfig {
    /// Width (height) of the front stretcher rail.
    #[serde(default = "default_stretcher_width")]
    pub front_width: f64,
    /// Whether to include a rear stretcher rail.
    #[serde(default = "default_true")]
    pub has_rear: bool,
}

fn default_stretcher_width() -> f64 {
    4.0
}

impl Default for StretcherConfig {
    fn default() -> Self {
        Self {
            front_width: default_stretcher_width(),
            has_rear: true,
        }
    }
}

/// Construction method for the cabinet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionMethod {
    /// Frameless (32mm/Euro system). No face frame; full-overlay doors.
    /// Shelf pin holes at 32mm spacing. Cup hinge boring on doors.
    #[default]
    Frameless,
    /// Face-frame construction. Additional stile and rail parts.
    /// Carcass slightly narrower; pocket hole joinery on frame.
    FaceFrame,
}

/// Configuration for face-frame construction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FaceFrameConfig {
    /// Width of face-frame stiles (vertical members). Typically 1.5".
    #[serde(default = "default_stile_width")]
    pub stile_width: f64,
    /// Width of face-frame rails (horizontal members). Typically 1.5".
    #[serde(default = "default_rail_width")]
    pub rail_width: f64,
    /// Overhang of face frame beyond carcass per side. Typically 1/16".
    #[serde(default = "default_frame_overhang")]
    pub overhang: f64,
    /// Whether to generate CNC pocket hole operations.
    #[serde(default)]
    pub cnc_pocket_holes: bool,
}

fn default_stile_width() -> f64 {
    1.5
}

fn default_rail_width() -> f64 {
    1.5
}

fn default_frame_overhang() -> f64 {
    0.0625 // 1/16"
}

impl Default for FaceFrameConfig {
    fn default() -> Self {
        Self {
            stile_width: default_stile_width(),
            rail_width: default_rail_width(),
            overhang: default_frame_overhang(),
            cnc_pocket_holes: false,
        }
    }
}

/// Corner cabinet type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CornerType {
    /// Diagonal face (standard 45-degree opening).
    Diagonal,
    /// Blind left (extends past adjacent left cabinet).
    BlindLeft,
    /// Blind right (extends past adjacent right cabinet).
    BlindRight,
}

/// Plumbing cutout definition for vanity/sink bases.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlumbingCutout {
    /// X position of cutout center from cabinet left edge.
    pub x: f64,
    /// Y position of cutout center from cabinet bottom.
    pub y: f64,
    /// Width of the cutout.
    pub width: f64,
    /// Height of the cutout.
    pub height: f64,
}

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}

/// A structural validation issue found in a cabinet design.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
}

/// Validate a cabinet design against structural/AWI/KCMA rules.
///
/// Returns a list of warnings and errors. Errors indicate designs that
/// cannot be built safely; warnings indicate suboptimal designs.
pub fn validate_cabinet(cabinet: &Cabinet) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Rule: Shelf span warning — width > 36" without center support
    if cabinet.width > 36.0 && cabinet.shelf_count > 0 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: format!(
                "Shelf span {:.1}\" exceeds 36\" — consider adding a center support or divider",
                cabinet.width
            ),
        });
    }

    // Rule: Material thickness vs span — 1/4" material can't span >12" as a shelf
    if cabinet.material_thickness <= 0.25 && cabinet.width > 12.0 && cabinet.shelf_count > 0 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: format!(
                "Material thickness {:.3}\" is too thin for {:.1}\" shelf span (max 12\")",
                cabinet.material_thickness, cabinet.width
            ),
        });
    }

    // Rule: Dado depth cannot exceed 50% of side thickness
    if cabinet.dado_depth_fraction > 0.5 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: format!(
                "Dado depth fraction {:.0}% exceeds 50% of material thickness — structural weakness",
                cabinet.dado_depth_fraction * 100.0
            ),
        });
    }

    // Rule: Toe kick height must be 3-4.5" (KCMA standard)
    if let Some(ref tk) = cabinet.toe_kick
        && (tk.height < 3.0 || tk.height > 4.5)
    {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: format!(
                "Toe kick height {:.1}\" outside KCMA standard range (3.0\"-4.5\")",
                tk.height
            ),
        });
    }

    // Rule: Drawer box height cannot exceed opening height minus slide clearance
    if let Some(ref drawers) = cabinet.drawers
        && drawers.opening_height > 0.0
    {
        let max_box_height = drawers.opening_height - 2.0 * drawers.slide_clearance;
        if max_box_height <= 0.0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: format!(
                    "Drawer opening {:.3}\" minus slide clearance leaves no room for drawer box",
                    drawers.opening_height
                ),
            });
        }
    }

    // Rule: 1/4" back on cabinet >48" tall needs mid-rail
    if cabinet.has_back && cabinet.back_thickness <= 0.25 && cabinet.height > 48.0 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: format!(
                "1/4\" back panel on {:.1}\" tall cabinet should have a mid-rail for rigidity",
                cabinet.height
            ),
        });
    }

    // Rule: Cabinet dimensions must be positive
    if cabinet.width <= 0.0 || cabinet.height <= 0.0 || cabinet.depth <= 0.0 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Cabinet dimensions must all be positive".into(),
        });
    }

    // Rule: Material thickness must be positive
    if cabinet.material_thickness <= 0.0 {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Material thickness must be positive".into(),
        });
    }

    issues
}

impl Cabinet {
    /// Generate all parts for this cabinet based on its parameters.
    pub fn generate_parts(&self) -> Vec<Part> {
        let mut parts = match self.cabinet_type {
            CabinetType::BasicBox => self.generate_basic_box(),
            CabinetType::BaseCabinet => self.generate_base_cabinet(),
            CabinetType::WallCabinet => self.generate_wall_cabinet(),
            CabinetType::TallCabinet => self.generate_tall_cabinet(),
            CabinetType::SinkBase => self.generate_sink_base(),
            CabinetType::DrawerBank => self.generate_drawer_bank(),
            CabinetType::CornerCabinet => self.generate_corner_cabinet(),
            CabinetType::VanityBase => self.generate_vanity_base(),
        };

        // If face-frame construction, adjust carcass and add frame parts
        if self.construction == ConstructionMethod::FaceFrame {
            self.apply_face_frame(&mut parts);
        }

        parts
    }

    /// Apply face-frame construction: narrow shelves slightly and add frame parts.
    fn apply_face_frame(&self, parts: &mut Vec<Part>) {
        let ff = self.face_frame.unwrap_or_default();

        // Shelves need to be narrower to clear stile overhang on each side
        for part in parts.iter_mut() {
            if part.label == "shelf" || part.label == "center_shelf" {
                let new_width = part.rect.width - 2.0 * ff.overhang;
                part.rect = Rect::new(Point2D::origin(), new_width, part.rect.height);
            }
        }

        // Face frame stiles (2): full height of the visible face
        let tk_height = self.toe_kick.map_or(0.0, |tk| tk.height);
        let stile_height = self.height - tk_height;
        let stile_ops = if ff.cnc_pocket_holes {
            // Pocket holes on the back face to attach to carcass
            let spacing = 8.0; // pocket holes every 8"
            let count = (stile_height / spacing).floor() as u32;
            let mut ops = Vec::new();
            for i in 0..count.max(2) {
                let y = stile_height * (i + 1) as f64 / (count.max(2) + 1) as f64;
                ops.push(PartOperation::PocketHole(PocketHoleOp {
                    x: ff.stile_width / 2.0,
                    y,
                    edge: Edge::Right, // toward carcass
                    cnc_operation: ff.cnc_pocket_holes,
                }));
            }
            ops
        } else {
            vec![]
        };

        parts.push(Part {
            label: "face_frame_stile".into(),
            rect: Rect::new(Point2D::origin(), ff.stile_width, stile_height),
            thickness: self.material_thickness,
            grain_direction: Default::default(),
            operations: stile_ops,
            quantity: 2,
        });

        // Face frame rails: span between stiles.
        // Number of rails = shelf_count + 2 (top rail + bottom rail) for basic boxes,
        // or just top + bottom for simple cases.
        let rail_count = self.shelf_count + 2;
        let rail_length = self.width - 2.0 * ff.stile_width + 2.0 * ff.overhang;

        let rail_ops = if ff.cnc_pocket_holes {
            // Pocket holes on each end of each rail
            vec![
                PartOperation::PocketHole(PocketHoleOp {
                    x: 0.5,
                    y: ff.rail_width / 2.0,
                    edge: Edge::Left,
                    cnc_operation: ff.cnc_pocket_holes,
                }),
                PartOperation::PocketHole(PocketHoleOp {
                    x: rail_length - 0.5,
                    y: ff.rail_width / 2.0,
                    edge: Edge::Right,
                    cnc_operation: ff.cnc_pocket_holes,
                }),
            ]
        } else {
            vec![]
        };

        parts.push(Part {
            label: "face_frame_rail".into(),
            rect: Rect::new(Point2D::origin(), rail_length, ff.rail_width),
            thickness: self.material_thickness,
            grain_direction: Default::default(),
            operations: rail_ops,
            quantity: rail_count,
        });
    }

    /// Common helper: dado depth from fraction.
    fn dado_depth(&self) -> f64 {
        self.material_thickness * self.dado_depth_fraction
    }

    /// Common helper: build side panel operations for dados (top, bottom, shelves)
    /// and back rabbet. Returns the ops vector.
    fn build_side_ops(
        &self,
        bottom_dado_y: f64,
        top_dado_y: f64,
        shelf_positions: &[f64],
        include_back_rabbet: bool,
    ) -> Vec<PartOperation> {
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        let mut ops: Vec<PartOperation> = Vec::new();

        // Bottom dado
        ops.push(PartOperation::Dado(DadoOp {
            position: bottom_dado_y,
            width: mt,
            depth: dado_depth,
            orientation: DadoOrientation::Horizontal,
        }));

        // Top dado
        ops.push(PartOperation::Dado(DadoOp {
            position: top_dado_y,
            width: mt,
            depth: dado_depth,
            orientation: DadoOrientation::Horizontal,
        }));

        // Shelf dados
        if self.shelf_joinery == ShelfJoinery::Dado {
            for &pos in shelf_positions {
                ops.push(PartOperation::Dado(DadoOp {
                    position: pos,
                    width: mt,
                    depth: dado_depth,
                    orientation: DadoOrientation::Horizontal,
                }));
            }
        }

        // Back rabbet
        if include_back_rabbet && self.has_back && self.back_joinery == BackJoinery::Rabbet {
            ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        ops
    }

    /// Common helper: generate evenly spaced shelf positions between two bounds.
    fn shelf_positions(&self, lower_bound: f64, upper_bound: f64) -> Vec<f64> {
        if self.shelf_count == 0 {
            return Vec::new();
        }
        let usable = upper_bound - lower_bound;
        let spacing = usable / (self.shelf_count + 1) as f64;
        (1..=self.shelf_count)
            .map(|i| lower_bound + spacing * i as f64)
            .collect()
    }

    /// Common helper: top/bottom panel width accounting for dado depth.
    fn horizontal_panel_width(&self) -> f64 {
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        self.width - 2.0 * mt + 2.0 * dado_depth
    }

    /// Common helper: generate shelf parts.
    fn generate_shelf_parts(&self, shelf_width: f64, shelf_depth: f64) -> Option<Part> {
        if self.shelf_count == 0 {
            return None;
        }
        Some(Part {
            label: "shelf".into(),
            rect: Rect::new(Point2D::origin(), shelf_width, shelf_depth),
            thickness: self.material_thickness,
            grain_direction: Default::default(),
            operations: vec![],
            quantity: self.shelf_count,
        })
    }

    /// Common helper: generate the back panel.
    fn generate_back_part(&self, back_height: f64) -> Option<Part> {
        if !self.has_back {
            return None;
        }
        let dado_depth = self.dado_depth();
        let mt = self.material_thickness;
        let back_width = match self.back_joinery {
            BackJoinery::Rabbet => self.width - 2.0 * (mt - dado_depth),
            BackJoinery::NailedOn => self.width,
        };
        let back_h = match self.back_joinery {
            BackJoinery::Rabbet => back_height - 2.0 * (mt - dado_depth),
            BackJoinery::NailedOn => back_height,
        };
        Some(Part {
            label: "back".into(),
            rect: Rect::new(Point2D::origin(), back_width, back_h),
            thickness: self.back_thickness,
            grain_direction: Default::default(),
            operations: vec![],
            quantity: 1,
        })
    }

    // -----------------------------------------------------------------------
    // BasicBox — simple open box (bookshelf)
    // -----------------------------------------------------------------------
    fn generate_basic_box(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();

        // --- Side panels ---
        let side_width = self.depth;
        let side_height = self.height;

        let shelf_positions = self.shelf_positions(mt, self.height - mt);
        let side_ops = self.build_side_ops(
            mt / 2.0,
            side_height - mt / 2.0,
            &shelf_positions,
            true,
        );

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // --- Top and bottom panels ---
        let tb_width = self.horizontal_panel_width();
        let tb_depth = self.depth;
        let mut tb_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            tb_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "bottom".into(),
            rect: Rect::new(Point2D::origin(), tb_width, tb_depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: tb_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "top".into(),
            rect: Rect::new(Point2D::origin(), tb_width, tb_depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: tb_ops,
            quantity: 1,
        });

        // --- Shelves ---
        let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            self.depth - self.back_thickness
        } else {
            self.depth
        };
        if let Some(shelf_part) = self.generate_shelf_parts(tb_width, shelf_depth) {
            parts.push(shelf_part);
        }

        // --- Back ---
        if let Some(back) = self.generate_back_part(self.height) {
            parts.push(back);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // BaseCabinet — toe kick, standard 34.5" H x 24" D kitchen lower
    // -----------------------------------------------------------------------
    fn generate_base_cabinet(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        let tk = self.toe_kick.unwrap_or_default();

        // Side panels run full height. The toe kick notch is cut from
        // the front-bottom corner of each side panel.
        let side_width = self.depth;
        let side_height = self.height;

        // Interior height: from bottom panel (above toe kick) to top
        let bottom_y = tk.height + mt / 2.0;
        let top_y = side_height - mt / 2.0;
        let shelf_positions = self.shelf_positions(tk.height + mt, self.height - mt);

        let mut side_ops = self.build_side_ops(bottom_y, top_y, &shelf_positions, true);

        // Toe kick notch: a dado-like cut at the bottom-front of the side panel.
        // We represent it as a rabbet on the bottom edge (the CNC will cut
        // the notch shape). The actual notch is tk.height tall x tk.setback deep.
        side_ops.push(PartOperation::Rabbet(RabbetOp {
            edge: Edge::Bottom,
            width: tk.setback,
            depth: mt, // through-cut for the notch
        }));

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // Bottom panel — sits above the toe kick
        let tb_width = self.horizontal_panel_width();
        let mut bottom_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            bottom_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "bottom".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: bottom_ops,
            quantity: 1,
        });

        // Top panel
        let mut top_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            top_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "top".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: top_ops,
            quantity: 1,
        });

        // Shelves
        let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            self.depth - self.back_thickness
        } else {
            self.depth
        };
        if let Some(shelf_part) = self.generate_shelf_parts(tb_width, shelf_depth) {
            parts.push(shelf_part);
        }

        // Back panel — from bottom panel to top (not including toe kick area)
        let back_height = self.height - tk.height;
        if let Some(back) = self.generate_back_part(back_height) {
            parts.push(back);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // WallCabinet — shallower, no toe kick, upper kitchen cabinet
    // -----------------------------------------------------------------------
    fn generate_wall_cabinet(&self) -> Vec<Part> {
        // Wall cabinets are structurally identical to BasicBox — they're
        // just shallower (12" standard) and typically have different
        // standard heights (30" or 42"). The parametric values handle this.
        self.generate_basic_box()
    }

    // -----------------------------------------------------------------------
    // TallCabinet — pantry/utility, 84" or 96" tall, with toe kick
    // -----------------------------------------------------------------------
    fn generate_tall_cabinet(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        let tk = self.toe_kick.unwrap_or_default();

        let side_width = self.depth;
        let side_height = self.height;

        let bottom_y = tk.height + mt / 2.0;
        let top_y = side_height - mt / 2.0;

        // For tall cabinets, add a fixed center shelf for rigidity if height > 60"
        let mut shelf_positions = self.shelf_positions(tk.height + mt, self.height - mt);
        let has_center_shelf = self.height > 60.0;
        let center_y = if has_center_shelf {
            let cy = (tk.height + mt + self.height - mt) / 2.0;
            // Only add center shelf if it's not already close to an existing shelf
            if !shelf_positions.iter().any(|&p| (p - cy).abs() < 4.0) {
                shelf_positions.push(cy);
                shelf_positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Some(cy)
            } else {
                None
            }
        } else {
            None
        };

        let mut side_ops = self.build_side_ops(bottom_y, top_y, &shelf_positions, true);

        // Toe kick notch
        side_ops.push(PartOperation::Rabbet(RabbetOp {
            edge: Edge::Bottom,
            width: tk.setback,
            depth: mt,
        }));

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // Bottom (above toe kick)
        let tb_width = self.horizontal_panel_width();
        let mut bottom_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            bottom_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "bottom".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: bottom_ops,
            quantity: 1,
        });

        // Top
        let mut top_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            top_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "top".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: top_ops,
            quantity: 1,
        });

        // Center shelf for rigidity (fixed, not adjustable)
        if center_y.is_some() {
            let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
                self.depth - self.back_thickness
            } else {
                self.depth
            };
            parts.push(Part {
                label: "center_shelf".into(),
                rect: Rect::new(Point2D::origin(), tb_width, shelf_depth),
                thickness: mt,
                grain_direction: Default::default(),
                operations: vec![],
                quantity: 1,
            });
        }

        // Adjustable shelves
        let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            self.depth - self.back_thickness
        } else {
            self.depth
        };
        if let Some(shelf_part) = self.generate_shelf_parts(tb_width, shelf_depth) {
            parts.push(shelf_part);
        }

        // Back panel — full height minus toe kick
        let back_height = self.height - tk.height;
        if let Some(back) = self.generate_back_part(back_height) {
            parts.push(back);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // SinkBase — no bottom shelf, open front, stretchers across top
    // -----------------------------------------------------------------------
    fn generate_sink_base(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        let tk = self.toe_kick.unwrap_or_default();
        let stretcher = self.stretchers.unwrap_or_default();

        let side_width = self.depth;
        let side_height = self.height;

        // Sink base has NO bottom panel and NO shelves.
        // Only a top dado for the stretcher mounting area.
        let top_y = side_height - mt / 2.0;

        let mut side_ops: Vec<PartOperation> = Vec::new();

        // Top dado only (for stretcher/cleat connection)
        side_ops.push(PartOperation::Dado(DadoOp {
            position: top_y,
            width: mt,
            depth: dado_depth,
            orientation: DadoOrientation::Horizontal,
        }));

        // Back rabbet
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            side_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        // Toe kick notch
        side_ops.push(PartOperation::Rabbet(RabbetOp {
            edge: Edge::Bottom,
            width: tk.setback,
            depth: mt,
        }));

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // Front stretcher (replaces full top panel)
        let stretcher_width = self.horizontal_panel_width();
        parts.push(Part {
            label: "front_stretcher".into(),
            rect: Rect::new(Point2D::origin(), stretcher_width, stretcher.front_width),
            thickness: mt,
            grain_direction: Default::default(),
            operations: vec![],
            quantity: 1,
        });

        // Rear stretcher (optional)
        if stretcher.has_rear {
            parts.push(Part {
                label: "rear_stretcher".into(),
                rect: Rect::new(Point2D::origin(), stretcher_width, stretcher.front_width),
                thickness: mt,
                grain_direction: Default::default(),
                operations: vec![],
                quantity: 1,
            });
        }

        // Back panel — from toe kick top to cabinet top
        let back_height = self.height - tk.height;
        if let Some(back) = self.generate_back_part(back_height) {
            parts.push(back);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // DrawerBank — base cabinet with drawer divider slots
    // -----------------------------------------------------------------------
    fn generate_drawer_bank(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();
        let tk = self.toe_kick.unwrap_or_default();
        let drawer = self.drawers.unwrap_or_default();

        let side_width = self.depth;
        let side_height = self.height;

        // Drawer dividers are horizontal panels between drawer openings.
        // We need (drawer_count - 1) dividers between openings, plus
        // bottom and top panels.
        let bottom_y = tk.height + mt / 2.0;
        let top_y = side_height - mt / 2.0;

        // Calculate divider positions
        let interior_height = top_y - bottom_y - mt; // space between top/bottom dados
        let divider_count = drawer.count.saturating_sub(1);
        let divider_spacing = interior_height / drawer.count as f64;
        let divider_positions: Vec<f64> = (1..=divider_count)
            .map(|i| bottom_y + mt / 2.0 + divider_spacing * i as f64)
            .collect();

        let mut side_ops = self.build_side_ops(bottom_y, top_y, &divider_positions, true);

        // Toe kick notch
        side_ops.push(PartOperation::Rabbet(RabbetOp {
            edge: Edge::Bottom,
            width: tk.setback,
            depth: mt,
        }));

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // Bottom panel
        let tb_width = self.horizontal_panel_width();
        let mut bottom_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            bottom_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "bottom".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: bottom_ops,
            quantity: 1,
        });

        // Top panel
        let mut top_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            top_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "top".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: top_ops,
            quantity: 1,
        });

        // Drawer dividers (horizontal panels between drawer openings)
        if divider_count > 0 {
            let divider_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
                self.depth - self.back_thickness
            } else {
                self.depth
            };

            parts.push(Part {
                label: "drawer_divider".into(),
                rect: Rect::new(Point2D::origin(), tb_width, divider_depth),
                thickness: mt,
                grain_direction: Default::default(),
                operations: vec![],
                quantity: divider_count,
            });
        }

        // Back panel
        let back_height = self.height - tk.height;
        if let Some(back) = self.generate_back_part(back_height) {
            parts.push(back);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // CornerCabinet — L-shaped cabinet for corner installation
    // -----------------------------------------------------------------------
    fn generate_corner_cabinet(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = self.dado_depth();

        // Corner cabinet: two connected boxes sharing a partition.
        // Standard 36"x36" footprint with the width representing one side.
        // The depth is the other side's length.
        let side_height = self.height;
        let tk = self.toe_kick.unwrap_or_default();

        let bottom_y = if self.toe_kick.is_some() { tk.height + mt / 2.0 } else { mt / 2.0 };
        let top_y = side_height - mt / 2.0;
        let shelf_positions = self.shelf_positions(
            if self.toe_kick.is_some() { tk.height + mt } else { mt },
            self.height - mt,
        );

        // Two side panels (different depths for L-shape)
        let side_ops_left = self.build_side_ops(bottom_y, top_y, &shelf_positions, true);

        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), self.depth, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops_left.clone(),
            quantity: 1,
        });

        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), self.depth, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops_left,
            quantity: 1,
        });

        // Shared partition (center divider)
        let partition_depth = self.depth;
        let partition_height = side_height;
        parts.push(Part {
            label: "partition".into(),
            rect: Rect::new(Point2D::origin(), partition_depth, partition_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: vec![],
            quantity: 1,
        });

        // Top and bottom panels
        let tb_width = self.horizontal_panel_width();
        let mut tb_ops = Vec::new();
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            tb_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right,
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        parts.push(Part {
            label: "bottom".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: tb_ops.clone(),
            quantity: 1,
        });
        parts.push(Part {
            label: "top".into(),
            rect: Rect::new(Point2D::origin(), tb_width, self.depth),
            thickness: mt,
            grain_direction: Default::default(),
            operations: tb_ops,
            quantity: 1,
        });

        // Two back panels (one for each wing of the L)
        if self.has_back {
            let back_height = if self.toe_kick.is_some() {
                self.height - tk.height
            } else {
                self.height
            };
            if let Some(mut back) = self.generate_back_part(back_height) {
                back.quantity = 2;
                parts.push(back);
            }
        }

        // Shelves
        let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            self.depth - self.back_thickness
        } else {
            self.depth
        };
        if let Some(shelf_part) = self.generate_shelf_parts(tb_width, shelf_depth) {
            parts.push(shelf_part);
        }

        parts
    }

    // -----------------------------------------------------------------------
    // VanityBase — bathroom vanity variant of BaseCabinet
    // -----------------------------------------------------------------------
    fn generate_vanity_base(&self) -> Vec<Part> {
        // Vanity is structurally identical to BaseCabinet with different
        // default dimensions (31.5" H, 21" D). The parametric values handle this.
        // We reuse BaseCabinet generation.
        self.generate_base_cabinet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cabinet() -> Cabinet {
        Cabinet {
            name: "Test Bookshelf".into(),
            cabinet_type: CabinetType::BasicBox,
            width: 36.0,
            height: 30.0,
            depth: 12.0,
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
    fn test_generates_correct_part_count() {
        let cab = test_cabinet();
        let parts = cab.generate_parts();
        // 2 sides + top + bottom + 1 shelf entry (qty 2) + back = 6 part entries
        assert_eq!(parts.len(), 6);
    }

    #[test]
    fn test_side_panel_dimensions() {
        let cab = test_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        // Side: depth x height = 12.0 x 30.0
        assert!((left.rect.width - 12.0).abs() < 1e-10);
        assert!((left.rect.height - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_top_bottom_width_accounts_for_dados() {
        let cab = test_cabinet();
        let parts = cab.generate_parts();
        let bottom = parts.iter().find(|p| p.label == "bottom").unwrap();
        // Width = 36 - 2*0.75 + 2*(0.75*0.5) = 36 - 1.5 + 0.75 = 35.25
        let expected = 36.0 - 2.0 * 0.75 + 2.0 * (0.75 * 0.5);
        assert!(
            (bottom.rect.width - expected).abs() < 1e-10,
            "expected {expected}, got {}",
            bottom.rect.width
        );
    }

    #[test]
    fn test_shelf_quantity() {
        let cab = test_cabinet();
        let parts = cab.generate_parts();
        let shelf = parts.iter().find(|p| p.label == "shelf").unwrap();
        assert_eq!(shelf.quantity, 2);
    }

    #[test]
    fn test_side_has_dados_for_shelves() {
        let cab = test_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        // Should have: bottom dado + top dado + 2 shelf dados + 1 rabbet = 5 ops
        assert_eq!(left.operations.len(), 5);

        let dado_count = left
            .operations
            .iter()
            .filter(|op| matches!(op, PartOperation::Dado(_)))
            .count();
        assert_eq!(dado_count, 4); // top + bottom + 2 shelves
    }

    // --- Base Cabinet Tests ---

    fn test_base_cabinet() -> Cabinet {
        Cabinet {
            name: "Kitchen Base".into(),
            cabinet_type: CabinetType::BaseCabinet,
            width: 24.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 1,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig { height: 4.0, setback: 3.0 }),
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_base_cabinet_part_count() {
        let cab = test_base_cabinet();
        let parts = cab.generate_parts();
        // 2 sides + bottom + top + shelf (qty 1) + back = 6
        assert_eq!(parts.len(), 6);
    }

    #[test]
    fn test_base_cabinet_has_toe_kick_notch() {
        let cab = test_base_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();

        let rabbet_count = left.operations.iter()
            .filter(|op| matches!(op, PartOperation::Rabbet(r) if r.edge == Edge::Bottom))
            .count();
        assert_eq!(rabbet_count, 1, "side should have toe kick notch (bottom rabbet)");
    }

    #[test]
    fn test_base_cabinet_dimensions() {
        let cab = test_base_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        assert!((left.rect.height - 34.5).abs() < 1e-10);
        assert!((left.rect.width - 24.0).abs() < 1e-10);
    }

    // --- Wall Cabinet Tests ---

    fn test_wall_cabinet() -> Cabinet {
        Cabinet {
            name: "Kitchen Wall".into(),
            cabinet_type: CabinetType::WallCabinet,
            width: 30.0,
            height: 30.0,
            depth: 12.0,
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
    fn test_wall_cabinet_part_count() {
        let cab = test_wall_cabinet();
        let parts = cab.generate_parts();
        // Same as BasicBox: 2 sides + top + bottom + shelf(qty 2) + back = 6
        assert_eq!(parts.len(), 6);
    }

    #[test]
    fn test_wall_cabinet_no_toe_kick() {
        let cab = test_wall_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        let bottom_rabbet = left.operations.iter()
            .any(|op| matches!(op, PartOperation::Rabbet(r) if r.edge == Edge::Bottom));
        assert!(!bottom_rabbet, "wall cabinet should have no toe kick notch");
    }

    // --- Tall Cabinet Tests ---

    fn test_tall_cabinet() -> Cabinet {
        Cabinet {
            name: "Pantry".into(),
            cabinet_type: CabinetType::TallCabinet,
            width: 24.0,
            height: 84.0,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 2, // 2 shelves: center is NOT already a shelf position
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_tall_cabinet_has_center_shelf() {
        let cab = test_tall_cabinet();
        let parts = cab.generate_parts();
        // 84" tall > 60", 2 shelves don't hit the exact center, so center shelf added
        let center = parts.iter().find(|p| p.label == "center_shelf");
        assert!(center.is_some(), "tall cabinet should have a center shelf for rigidity");
    }

    #[test]
    fn test_tall_cabinet_part_count() {
        let cab = test_tall_cabinet();
        let parts = cab.generate_parts();
        // 2 sides + bottom + top + center_shelf + shelf(qty 2) + back = 7
        assert_eq!(parts.len(), 7);
    }

    #[test]
    fn test_tall_cabinet_no_duplicate_center_shelf() {
        // With 3 shelves, the middle one hits the center — no separate center shelf
        let mut cab = test_tall_cabinet();
        cab.shelf_count = 3;
        let parts = cab.generate_parts();
        let center = parts.iter().find(|p| p.label == "center_shelf");
        assert!(center.is_none(), "3 shelves should already cover the center");
    }

    #[test]
    fn test_tall_cabinet_height() {
        let cab = test_tall_cabinet();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        assert!((left.rect.height - 84.0).abs() < 1e-10);
    }

    // --- Sink Base Tests ---

    fn test_sink_base() -> Cabinet {
        Cabinet {
            name: "Sink Base".into(),
            cabinet_type: CabinetType::SinkBase,
            width: 36.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: None,
            stretchers: Some(StretcherConfig::default()),
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_sink_base_no_bottom_no_shelves() {
        let cab = test_sink_base();
        let parts = cab.generate_parts();
        let bottom = parts.iter().find(|p| p.label == "bottom");
        let shelf = parts.iter().find(|p| p.label == "shelf");
        assert!(bottom.is_none(), "sink base should have no bottom panel");
        assert!(shelf.is_none(), "sink base should have no shelves");
    }

    #[test]
    fn test_sink_base_has_stretchers() {
        let cab = test_sink_base();
        let parts = cab.generate_parts();
        let front = parts.iter().find(|p| p.label == "front_stretcher");
        let rear = parts.iter().find(|p| p.label == "rear_stretcher");
        assert!(front.is_some(), "sink base should have front stretcher");
        assert!(rear.is_some(), "sink base should have rear stretcher");
    }

    #[test]
    fn test_sink_base_part_count() {
        let cab = test_sink_base();
        let parts = cab.generate_parts();
        // 2 sides + front_stretcher + rear_stretcher + back = 5
        assert_eq!(parts.len(), 5);
    }

    // --- Drawer Bank Tests ---

    fn test_drawer_bank() -> Cabinet {
        Cabinet {
            name: "Drawer Bank".into(),
            cabinet_type: CabinetType::DrawerBank,
            width: 18.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: Some(DrawerConfig { count: 4, opening_height: 0.0, slide_clearance: 0.5 }),
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_drawer_bank_has_dividers() {
        let cab = test_drawer_bank();
        let parts = cab.generate_parts();
        let divider = parts.iter().find(|p| p.label == "drawer_divider");
        assert!(divider.is_some(), "drawer bank should have dividers");
        assert_eq!(divider.unwrap().quantity, 3, "4 drawers need 3 dividers");
    }

    #[test]
    fn test_drawer_bank_part_count() {
        let cab = test_drawer_bank();
        let parts = cab.generate_parts();
        // 2 sides + bottom + top + drawer_divider(qty 3) + back = 6
        assert_eq!(parts.len(), 6);
    }

    #[test]
    fn test_drawer_bank_side_has_divider_dados() {
        let cab = test_drawer_bank();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        // Should have: bottom dado + top dado + 3 divider dados + back rabbet + toe kick rabbet = 7
        let dado_count = left.operations.iter()
            .filter(|op| matches!(op, PartOperation::Dado(_)))
            .count();
        assert_eq!(dado_count, 5, "should have top + bottom + 3 divider dados");
    }

    // --- Face-Frame Construction Tests ---

    fn test_face_frame_cabinet() -> Cabinet {
        Cabinet {
            name: "Face Frame Base".into(),
            cabinet_type: CabinetType::BaseCabinet,
            width: 24.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 1,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::FaceFrame,
            face_frame: Some(FaceFrameConfig {
                stile_width: 1.5,
                rail_width: 1.5,
                overhang: 0.0625,
                cnc_pocket_holes: false,
            }),
            corner_type: None,
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_face_frame_adds_stiles_and_rails() {
        let cab = test_face_frame_cabinet();
        let parts = cab.generate_parts();
        let stile = parts.iter().find(|p| p.label == "face_frame_stile");
        let rail = parts.iter().find(|p| p.label == "face_frame_rail");
        assert!(stile.is_some(), "face-frame cabinet should have stiles");
        assert!(rail.is_some(), "face-frame cabinet should have rails");
    }

    #[test]
    fn test_face_frame_stile_quantity_and_dimensions() {
        let cab = test_face_frame_cabinet();
        let parts = cab.generate_parts();
        let stile = parts.iter().find(|p| p.label == "face_frame_stile").unwrap();
        assert_eq!(stile.quantity, 2, "should have 2 stiles");
        // Stile height = total height - toe kick height = 34.5 - 4.0 = 30.5
        assert!((stile.rect.height - 30.5).abs() < 1e-10);
        assert!((stile.rect.width - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_face_frame_rail_count() {
        let cab = test_face_frame_cabinet();
        let parts = cab.generate_parts();
        let rail = parts.iter().find(|p| p.label == "face_frame_rail").unwrap();
        // shelf_count(1) + 2 (top + bottom) = 3 rails
        assert_eq!(rail.quantity, 3);
    }

    #[test]
    fn test_face_frame_rail_length() {
        let cab = test_face_frame_cabinet();
        let parts = cab.generate_parts();
        let rail = parts.iter().find(|p| p.label == "face_frame_rail").unwrap();
        // Rail length = width - 2*stile_width + 2*overhang = 24 - 3.0 + 0.125 = 21.125
        let expected = 24.0 - 2.0 * 1.5 + 2.0 * 0.0625;
        assert!(
            (rail.rect.width - expected).abs() < 1e-10,
            "rail length expected {expected}, got {}",
            rail.rect.width
        );
    }

    #[test]
    fn test_face_frame_narrows_shelves() {
        let cab = test_face_frame_cabinet();
        let parts = cab.generate_parts();

        // Also generate without face frame to compare
        let mut cab_no_ff = test_face_frame_cabinet();
        cab_no_ff.construction = ConstructionMethod::Frameless;
        let parts_no_ff = cab_no_ff.generate_parts();

        let shelf_ff = parts.iter().find(|p| p.label == "shelf").unwrap();
        let shelf_no_ff = parts_no_ff.iter().find(|p| p.label == "shelf").unwrap();

        // Face-frame shelves should be narrower by 2 * overhang
        let expected_diff = 2.0 * 0.0625;
        let actual_diff = shelf_no_ff.rect.width - shelf_ff.rect.width;
        assert!(
            (actual_diff - expected_diff).abs() < 1e-10,
            "shelf should be {expected_diff} narrower, was {actual_diff}"
        );
    }

    #[test]
    fn test_face_frame_pocket_holes() {
        let mut cab = test_face_frame_cabinet();
        cab.face_frame = Some(FaceFrameConfig {
            stile_width: 1.5,
            rail_width: 1.5,
            overhang: 0.0625,
            cnc_pocket_holes: true,
        });
        let parts = cab.generate_parts();

        let stile = parts.iter().find(|p| p.label == "face_frame_stile").unwrap();
        let pocket_holes: Vec<_> = stile.operations.iter()
            .filter(|op| matches!(op, PartOperation::PocketHole(_)))
            .collect();
        assert!(!pocket_holes.is_empty(), "stiles should have pocket holes when cnc_pocket_holes=true");

        let rail = parts.iter().find(|p| p.label == "face_frame_rail").unwrap();
        let rail_pockets: Vec<_> = rail.operations.iter()
            .filter(|op| matches!(op, PartOperation::PocketHole(_)))
            .collect();
        assert_eq!(rail_pockets.len(), 2, "each rail should have 2 pocket holes (one per end)");
    }

    #[test]
    fn test_frameless_no_face_frame_parts() {
        let cab = test_cabinet(); // Frameless by default
        let parts = cab.generate_parts();
        let stile = parts.iter().find(|p| p.label == "face_frame_stile");
        let rail = parts.iter().find(|p| p.label == "face_frame_rail");
        assert!(stile.is_none(), "frameless cabinet should have no stiles");
        assert!(rail.is_none(), "frameless cabinet should have no rails");
    }

    // --- Validation Tests ---

    #[test]
    fn test_validate_wide_shelf_warning() {
        let mut cab = test_cabinet();
        cab.width = 48.0; // exceeds 36"
        cab.shelf_count = 1;
        let issues = validate_cabinet(&cab);
        assert!(issues.iter().any(|i| i.severity == ValidationSeverity::Warning
            && i.message.contains("36\"")),
            "should warn about wide shelf span");
    }

    #[test]
    fn test_validate_thin_material_error() {
        let mut cab = test_cabinet();
        cab.material_thickness = 0.25;
        cab.width = 24.0; // > 12"
        cab.shelf_count = 1;
        let issues = validate_cabinet(&cab);
        assert!(issues.iter().any(|i| i.severity == ValidationSeverity::Error
            && i.message.contains("too thin")),
            "should error on thin material for wide span");
    }

    #[test]
    fn test_validate_dado_depth_error() {
        let mut cab = test_cabinet();
        cab.dado_depth_fraction = 0.75; // 75% > 50%
        let issues = validate_cabinet(&cab);
        assert!(issues.iter().any(|i| i.severity == ValidationSeverity::Error
            && i.message.contains("Dado depth")),
            "should error on deep dado");
    }

    #[test]
    fn test_validate_toe_kick_warning() {
        let mut cab = test_base_cabinet();
        cab.toe_kick = Some(ToeKickConfig { height: 2.0, setback: 3.0 }); // too short
        let issues = validate_cabinet(&cab);
        assert!(issues.iter().any(|i| i.severity == ValidationSeverity::Warning
            && i.message.contains("KCMA")),
            "should warn about non-standard toe kick");
    }

    #[test]
    fn test_validate_tall_thin_back() {
        let mut cab = test_cabinet();
        cab.height = 84.0;
        cab.back_thickness = 0.25;
        cab.has_back = true;
        let issues = validate_cabinet(&cab);
        assert!(issues.iter().any(|i| i.message.contains("mid-rail")),
            "should warn about thin back on tall cabinet");
    }

    #[test]
    fn test_validate_good_cabinet_no_errors() {
        let cab = test_cabinet(); // Standard bookshelf — no issues
        let issues = validate_cabinet(&cab);
        let errors: Vec<_> = issues.iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "good cabinet should have no errors");
    }

    // --- Corner Cabinet Tests ---

    fn test_corner_cabinet() -> Cabinet {
        Cabinet {
            name: "Corner Cabinet".into(),
            cabinet_type: CabinetType::CornerCabinet,
            width: 36.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 1,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: Some(CornerType::Diagonal),
            plumbing_cutout: None,
        }
    }

    #[test]
    fn test_corner_cabinet_has_partition() {
        let cab = test_corner_cabinet();
        let parts = cab.generate_parts();
        let partition = parts.iter().find(|p| p.label == "partition");
        assert!(partition.is_some(), "corner cabinet should have a partition");
    }

    #[test]
    fn test_corner_cabinet_part_count() {
        let cab = test_corner_cabinet();
        let parts = cab.generate_parts();
        // 2 sides + partition + top + bottom + back(qty 2) + shelf = 7
        assert!(parts.len() >= 6, "corner cabinet should have at least 6 part entries, got {}", parts.len());
    }

    #[test]
    fn test_corner_cabinet_has_two_backs() {
        let cab = test_corner_cabinet();
        let parts = cab.generate_parts();
        let back = parts.iter().find(|p| p.label == "back");
        assert!(back.is_some(), "corner cabinet should have back panels");
        assert_eq!(back.unwrap().quantity, 2, "corner cabinet should have 2 back panels");
    }

    // --- Vanity Base Tests ---

    fn test_vanity_base() -> Cabinet {
        Cabinet {
            name: "Vanity Base".into(),
            cabinet_type: CabinetType::VanityBase,
            width: 36.0,
            height: 31.5, // standard vanity height
            depth: 21.0,  // standard vanity depth
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: Some(PlumbingCutout {
                x: 18.0, y: 15.0, width: 6.0, height: 8.0,
            }),
        }
    }

    #[test]
    fn test_vanity_base_dimensions() {
        let cab = test_vanity_base();
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        assert!((left.rect.height - 31.5).abs() < 1e-10, "vanity height should be 31.5\"");
        assert!((left.rect.width - 21.0).abs() < 1e-10, "vanity depth should be 21\"");
    }

    #[test]
    fn test_vanity_base_part_count() {
        let cab = test_vanity_base();
        let parts = cab.generate_parts();
        // Same as base cabinet: 2 sides + bottom + top + back = 5
        assert!(parts.len() >= 4, "vanity should have at least 4 part entries, got {}", parts.len());
    }

    // --- Validation edge-case tests ---

    #[test]
    fn test_validate_multiple_issues() {
        // Cabinet with multiple validation issues at once
        let cab = Cabinet {
            name: "bad_cabinet".to_string(),
            cabinet_type: CabinetType::BaseCabinet,
            width: 48.0,  // wide shelf → warning
            height: 60.0,
            depth: 24.0,
            material_thickness: 0.25, // thin material + wide span → error
            back_thickness: 0.25,
            shelf_count: 2,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.6, // >50% → error
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig { height: 2.0, setback: 3.0 }), // <3" → warning
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        };
        let issues = validate_cabinet(&cab);
        assert!(issues.len() >= 3, "should have at least 3 issues, got {}", issues.len());

        let errors = issues.iter().filter(|i| i.severity == ValidationSeverity::Error).count();
        let warnings = issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count();
        assert!(errors >= 2, "should have at least 2 errors (thin material + dado depth)");
        assert!(warnings >= 1, "should have at least 1 warning (toe kick)");
    }

    #[test]
    fn test_validate_zero_dimensions() {
        let cab = Cabinet {
            name: "zero".to_string(),
            cabinet_type: CabinetType::BasicBox,
            width: 0.0,
            height: 30.0,
            depth: 12.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: false,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: None,
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        };
        let issues = validate_cabinet(&cab);
        let dimension_errors = issues.iter()
            .filter(|i| i.severity == ValidationSeverity::Error && i.message.contains("dimension"))
            .count();
        assert!(dimension_errors >= 1, "zero width should trigger dimension error");
    }

    #[test]
    fn test_validate_negative_thickness() {
        let cab = Cabinet {
            name: "neg".to_string(),
            cabinet_type: CabinetType::BasicBox,
            width: 24.0,
            height: 30.0,
            depth: 12.0,
            material_thickness: -0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: false,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: None,
            drawers: None,
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        };
        let issues = validate_cabinet(&cab);
        let thickness_errors = issues.iter()
            .filter(|i| i.severity == ValidationSeverity::Error && i.message.contains("thickness"))
            .count();
        assert!(thickness_errors >= 1, "negative thickness should trigger error");
    }

    #[test]
    fn test_validate_narrow_shelf_no_warning() {
        // 24" wide cabinet — no shelf span warning
        let cab = Cabinet {
            name: "narrow".to_string(),
            cabinet_type: CabinetType::BasicBox,
            width: 24.0,
            height: 30.0,
            depth: 12.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 3,
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
        };
        let issues = validate_cabinet(&cab);
        let span_warnings = issues.iter()
            .filter(|i| i.message.contains("span"))
            .count();
        assert_eq!(span_warnings, 0, "24\" cabinet should not trigger span warning");
    }

    #[test]
    fn test_validate_drawer_opening_too_small() {
        let cab = Cabinet {
            name: "tight_drawers".to_string(),
            cabinet_type: CabinetType::DrawerBank,
            width: 18.0,
            height: 34.5,
            depth: 24.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 0,
            shelf_joinery: ShelfJoinery::Dado,
            dado_depth_fraction: 0.5,
            has_back: true,
            back_joinery: BackJoinery::Rabbet,
            toe_kick: Some(ToeKickConfig::default()),
            drawers: Some(DrawerConfig {
                count: 4,
                opening_height: 0.5, // way too small
                slide_clearance: 0.5,
            }),
            stretchers: None,
            construction: ConstructionMethod::Frameless,
            face_frame: None,
            corner_type: None,
            plumbing_cutout: None,
        };
        let issues = validate_cabinet(&cab);
        let drawer_errors = issues.iter()
            .filter(|i| i.severity == ValidationSeverity::Error && i.message.contains("drawer"))
            .count();
        assert!(drawer_errors >= 1, "tiny drawer opening should trigger error");
    }

    #[test]
    fn test_validate_toe_kick_boundary_values() {
        // toe kick at exact boundaries
        let make_cab = |tk_height: f64| -> Cabinet {
            Cabinet {
                name: "tk".to_string(),
                cabinet_type: CabinetType::BaseCabinet,
                width: 24.0, height: 34.5, depth: 24.0,
                material_thickness: 0.75, back_thickness: 0.25,
                shelf_count: 0, shelf_joinery: ShelfJoinery::Dado,
                dado_depth_fraction: 0.5,
                has_back: true, back_joinery: BackJoinery::Rabbet,
                toe_kick: Some(ToeKickConfig { height: tk_height, setback: 3.0 }),
                drawers: None, stretchers: None,
                construction: ConstructionMethod::Frameless,
                face_frame: None, corner_type: None, plumbing_cutout: None,
            }
        };

        // At exact boundaries (3.0 and 4.5) — should NOT warn
        let issues_30 = validate_cabinet(&make_cab(3.0));
        let tk_warns_30 = issues_30.iter().filter(|i| i.message.contains("kick")).count();
        assert_eq!(tk_warns_30, 0, "toe kick at 3.0\" should be valid");

        let issues_45 = validate_cabinet(&make_cab(4.5));
        let tk_warns_45 = issues_45.iter().filter(|i| i.message.contains("kick")).count();
        assert_eq!(tk_warns_45, 0, "toe kick at 4.5\" should be valid");

        // Just outside boundaries — should warn
        let issues_29 = validate_cabinet(&make_cab(2.9));
        let tk_warns_29 = issues_29.iter().filter(|i| i.message.contains("kick")).count();
        assert!(tk_warns_29 > 0, "toe kick at 2.9\" should warn");

        let issues_46 = validate_cabinet(&make_cab(4.6));
        let tk_warns_46 = issues_46.iter().filter(|i| i.message.contains("kick")).count();
        assert!(tk_warns_46 > 0, "toe kick at 4.6\" should warn");
    }

    #[test]
    fn test_validate_exactly_36_inch_no_span_warning() {
        // Width exactly 36" — should NOT trigger span warning (> 36", not >=)
        let cab = Cabinet {
            name: "boundary".to_string(),
            cabinet_type: CabinetType::BasicBox,
            width: 36.0,
            height: 30.0,
            depth: 12.0,
            material_thickness: 0.75,
            back_thickness: 0.25,
            shelf_count: 1,
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
        };
        let issues = validate_cabinet(&cab);
        let span_warnings = issues.iter()
            .filter(|i| i.message.contains("span"))
            .count();
        assert_eq!(span_warnings, 0, "36\" should not trigger span warning");
    }

    // --- Corner cabinet additional tests ---

    #[test]
    fn test_corner_cabinet_shelf_dados() {
        let mut cab = test_corner_cabinet();
        cab.shelf_count = 2;
        let parts = cab.generate_parts();
        let left = parts.iter().find(|p| p.label == "left_side").unwrap();
        let dado_ops: Vec<_> = left.operations.iter()
            .filter(|op| matches!(op, PartOperation::Dado(_)))
            .collect();
        // Should have dados for bottom, top, and 2 shelves = 4
        assert!(dado_ops.len() >= 3, "side should have dados, got {}", dado_ops.len());
    }

    #[test]
    fn test_corner_cabinet_no_shelves() {
        let mut cab = test_corner_cabinet();
        cab.shelf_count = 0;
        let parts = cab.generate_parts();
        let shelf_parts: Vec<_> = parts.iter().filter(|p| p.label.contains("shelf")).collect();
        assert!(shelf_parts.is_empty(), "0 shelves should produce no shelf parts");
    }

    // --- VanityBase additional tests ---

    #[test]
    fn test_vanity_base_with_shelves() {
        let mut cab = test_vanity_base();
        cab.shelf_count = 2;
        let parts = cab.generate_parts();
        let shelves: Vec<_> = parts.iter().filter(|p| p.label.contains("shelf")).collect();
        assert!(!shelves.is_empty(), "vanity with shelves should generate shelf parts");
    }

    #[test]
    fn test_vanity_base_without_back() {
        let mut cab = test_vanity_base();
        cab.has_back = false;
        let parts = cab.generate_parts();
        let backs: Vec<_> = parts.iter().filter(|p| p.label == "back").collect();
        assert!(backs.is_empty(), "vanity without back should have no back part");
    }
}
