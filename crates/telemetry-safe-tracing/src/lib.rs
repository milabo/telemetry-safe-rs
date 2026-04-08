//! `tracing` integration for `telemetry-safe`.
//!
//! This crate is intentionally thin so backend-specific macro work can evolve
//! without pulling `tracing` into the core safety model.

pub use telemetry_safe::{
    TelemetryDebug, TelemetryDisplay, ToTelemetry, telemetry, telemetry_debug,
};
pub use telemetry_safe_tracing_macros::safe_instrument;
pub use tracing;

// Proc macros expand through this public path so downstream crates do not need
// to know which helper crate owns the implementation details.
extern crate self as telemetry_safe_tracing;
