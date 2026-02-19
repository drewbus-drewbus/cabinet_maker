pub mod dxf_import;

use cm_cabinet::part::Part;
use cm_core::geometry::Point2D;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to read DXF file: {0}")]
    DxfRead(String),
    #[error("no closed rectangles found in DXF")]
    NoRectanglesFound,
    #[error("invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Import mode: how to interpret DXF layers/entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Smart mode: DXF layers map to operations.
    /// - Layer "PARTS" / "OUTLINES" -> part profile cuts
    /// - Layer "DADOS" -> dado grooves
    /// - Layer "DRILLS" -> drill holes (circle diameter = hole diameter)
    /// - Layer "RABBETS" -> rabbet operations
    LayerBased,
    /// Simple mode: every closed polyline/rectangle = a part.
    /// User specifies thickness and operations separately.
    Raw,
}

/// Options for DXF import.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Import mode (layer-based or raw).
    pub mode: ImportMode,
    /// Material thickness for all imported parts.
    pub thickness: f64,
    /// Default dado depth (fraction of thickness).
    pub dado_depth_fraction: f64,
    /// Default drill depth.
    pub drill_depth: f64,
    /// Default rabbet width.
    pub rabbet_width: f64,
    /// Default rabbet depth.
    pub rabbet_depth: f64,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Raw,
            thickness: 0.75,
            dado_depth_fraction: 0.5,
            drill_depth: 0.5,
            rabbet_width: 0.25,
            rabbet_depth: 0.375,
        }
    }
}

/// Result of importing a DXF file.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Parts extracted from the file.
    pub parts: Vec<Part>,
    /// Warnings during import (non-fatal).
    pub warnings: Vec<String>,
    /// Number of entities skipped (unsupported geometry).
    pub skipped_entities: usize,
}

/// Import parts from a DXF file.
pub fn import_dxf(
    path: &std::path::Path,
    options: &ImportOptions,
) -> Result<ImportResult, ImportError> {
    dxf_import::import_from_file(path, options)
}

/// A rectangle extracted from DXF geometry.
#[derive(Debug, Clone)]
pub struct ExtractedRect {
    /// Origin (bottom-left corner).
    pub origin: Point2D,
    /// Width of the rectangle.
    pub width: f64,
    /// Height of the rectangle.
    pub height: f64,
    /// Layer name from DXF.
    pub layer: String,
}

/// A circle extracted from DXF geometry.
#[derive(Debug, Clone)]
pub struct ExtractedCircle {
    /// Center point.
    pub center: Point2D,
    /// Diameter.
    pub diameter: f64,
    /// Layer name from DXF.
    pub layer: String,
}

/// A line extracted from DXF geometry.
#[derive(Debug, Clone)]
pub struct ExtractedLine {
    /// Start point.
    pub start: Point2D,
    /// End point.
    pub end: Point2D,
    /// Layer name from DXF.
    pub layer: String,
}
