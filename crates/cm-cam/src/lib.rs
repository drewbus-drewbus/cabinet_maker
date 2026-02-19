pub mod toolpath;
pub mod ops;
pub mod error;

pub use toolpath::{Motion, Toolpath, ToolpathSegment, FilletStyle};
pub use toolpath::{arc_fit, optimize_rapid_order, apply_corner_fillets};
pub use ops::{
    generate_profile_cut, generate_dado_toolpath, generate_drill,
    generate_rabbet_toolpath, generate_drill_pattern, generate_shelf_pin_pattern,
    generate_dovetail_toolpath, generate_box_joint_toolpath, generate_mortise_toolpath,
    generate_tenon_toolpath, generate_dowel_holes,
    RabbetEdge, DovetailEdge, DrillHole, CamConfig,
};
pub use error::CamError;
