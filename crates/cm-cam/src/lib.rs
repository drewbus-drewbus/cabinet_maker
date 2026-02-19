pub mod toolpath;
pub mod ops;

pub use toolpath::{Motion, Toolpath, ToolpathSegment, FilletStyle};
pub use toolpath::{arc_fit, optimize_rapid_order, apply_corner_fillets};
pub use ops::{
    generate_profile_cut, generate_dado_toolpath, generate_drill,
    generate_rabbet_toolpath, generate_drill_pattern, generate_shelf_pin_pattern,
    RabbetEdge, DrillHole, CamConfig,
};
