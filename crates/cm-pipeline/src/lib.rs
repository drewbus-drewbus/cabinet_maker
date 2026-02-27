pub mod bom;
pub mod error;
pub mod estimate;
pub mod generate;
pub mod visualize;

pub use bom::{
    Bom, BomTotals, CabinetBom, CostSummary, EdgeBandEntry, HardwareEntry, SheetPartEntry,
    generate_bom,
};
pub use error::PipelineError;
pub use estimate::{QuickEstimate, CabinetEstimate, quick_estimate};
pub use generate::{
    GenerateConfig, GenerateResult, MaterialGroupOutput,
    ProgressEvent, ProgressReporter, NullReporter,
    generate_pipeline, strip_nesting_id,
    generate_sheet_toolpaths, generate_operation_toolpath,
};
pub use visualize::{
    AnnotatedToolpath, OperationType, ToolpathVisualizationDto,
    generate_annotated_toolpaths,
};
