use cm_core::geometry::{Point2D, Rect};
use serde::{Deserialize, Serialize};

use crate::part::{DadoOp, DadoOrientation, Part, PartOperation, RabbetOp, Edge};

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

impl Cabinet {
    /// Generate all parts for this cabinet based on its parameters.
    pub fn generate_parts(&self) -> Vec<Part> {
        match self.cabinet_type {
            CabinetType::BasicBox => self.generate_basic_box(),
        }
    }

    fn generate_basic_box(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        let mt = self.material_thickness;
        let dado_depth = mt * self.dado_depth_fraction;

        // --- Side panels (left and right) ---
        // Sides run full height. Depth is the panel width (front-to-back).
        let side_width = self.depth;
        let side_height = self.height;

        let mut side_ops: Vec<PartOperation> = Vec::new();

        // Dado for bottom
        side_ops.push(PartOperation::Dado(DadoOp {
            position: mt / 2.0, // bottom dado, centered on material thickness from bottom
            width: mt,
            depth: dado_depth,
            orientation: DadoOrientation::Horizontal,
        }));

        // Dado for top
        side_ops.push(PartOperation::Dado(DadoOp {
            position: side_height - mt / 2.0,
            width: mt,
            depth: dado_depth,
            orientation: DadoOrientation::Horizontal,
        }));

        // Dados for shelves (evenly spaced between top and bottom)
        if self.shelf_count > 0 && self.shelf_joinery == ShelfJoinery::Dado {
            let usable_height = self.height - 2.0 * mt; // space between top and bottom
            let spacing = usable_height / (self.shelf_count + 1) as f64;
            for i in 1..=self.shelf_count {
                let y = mt + spacing * i as f64;
                side_ops.push(PartOperation::Dado(DadoOp {
                    position: y,
                    width: mt,
                    depth: dado_depth,
                    orientation: DadoOrientation::Horizontal,
                }));
            }
        }

        // Rabbet for back panel
        if self.has_back && self.back_joinery == BackJoinery::Rabbet {
            side_ops.push(PartOperation::Rabbet(RabbetOp {
                edge: Edge::Right, // back edge of side panel (when viewed from inside)
                width: self.back_thickness,
                depth: dado_depth,
            }));
        }

        // Left side
        parts.push(Part {
            label: "left_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops.clone(),
            quantity: 1,
        });

        // Right side (mirror of left — same operations)
        parts.push(Part {
            label: "right_side".into(),
            rect: Rect::new(Point2D::origin(), side_width, side_height),
            thickness: mt,
            grain_direction: Default::default(),
            operations: side_ops,
            quantity: 1,
        });

        // --- Top and bottom panels ---
        // Width fits between the two sides, accounting for dado depth.
        let tb_width = self.width - 2.0 * mt + 2.0 * dado_depth;
        let tb_depth = self.depth;
        let mut tb_ops = Vec::new();

        // Rabbet for back panel on top and bottom
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
        // Same width as top/bottom
        if self.shelf_count > 0 {
            let shelf_width = tb_width;
            let shelf_depth = if self.has_back && self.back_joinery == BackJoinery::Rabbet {
                // Shelves are slightly shallower to clear the back rabbet
                self.depth - self.back_thickness
            } else {
                self.depth
            };

            parts.push(Part {
                label: "shelf".into(),
                rect: Rect::new(Point2D::origin(), shelf_width, shelf_depth),
                thickness: mt,
                grain_direction: Default::default(),
                operations: vec![],
                quantity: self.shelf_count,
            });
        }

        // --- Back panel ---
        if self.has_back {
            let back_width = match self.back_joinery {
                BackJoinery::Rabbet => self.width - 2.0 * (mt - dado_depth),
                BackJoinery::NailedOn => self.width,
            };
            let back_height = match self.back_joinery {
                BackJoinery::Rabbet => self.height - 2.0 * (mt - dado_depth),
                BackJoinery::NailedOn => self.height,
            };

            parts.push(Part {
                label: "back".into(),
                rect: Rect::new(Point2D::origin(), back_width, back_height),
                thickness: self.back_thickness,
                grain_direction: Default::default(),
                operations: vec![],
                quantity: 1,
            });
        }

        parts
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
}
