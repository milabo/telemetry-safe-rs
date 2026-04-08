//! `tracing` integration for `telemetry-safe`.
//!
//! This crate is intentionally thin so backend-specific macro work can evolve
//! without pulling `tracing` into the core safety model.

pub use telemetry_safe::{
    TelemetryDebug, TelemetryDisplay, ToTelemetry, telemetry, telemetry_debug,
};
pub use telemetry_safe_tracing_macros::safe_instrument;
pub use tracing;
