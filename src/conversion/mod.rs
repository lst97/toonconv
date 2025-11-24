//! JSON to TOON conversion module
//!
//! This module contains the core conversion logic, configuration, and statistics.

pub mod batch;
pub mod config;
pub mod engine;
pub mod limits;
pub mod memory_opt;
pub mod stats;

pub use config::{ConversionConfig, DelimiterType, QuoteStrategy};

pub use engine::{convert_json_to_toon, ToonData};

use crate::error::ConversionError;

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Result type for operations that return TOON data
pub type ToonConversionResult = ConversionResult<ToonData>;
