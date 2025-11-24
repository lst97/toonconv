//! TOON validation module

pub mod circular_refs;
pub mod toon_compliance;

pub use circular_refs::CircularRefDetector;
pub use toon_compliance::ToonValidator;
