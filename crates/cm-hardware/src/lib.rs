//! Hardware Library for Cabinet Maker.
//!
//! Provides a catalog of cabinet hardware (hinges, drawer slides, shelf pins, pulls)
//! with automatic boring pattern generation. After joinery rules apply operations,
//! `HardwareApplicator` adds drill operations and adjusts part dimensions
//! (e.g., drawer width reduced by slide clearance).
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

pub mod catalog;

pub use catalog::{
    Hardware, HardwareKind, HingeSpec, SlideSpec, ShelfPinSpec, PullSpec,
    HardwareApplicator, HardwareOp,
};
