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

/// Marks a string literal as intentionally safe to emit from `safe_instrument`.
///
/// This helper is feature-gated because allowing literals is a product-level
/// tradeoff. Requiring an explicit wrapper keeps that choice visible in call sites.
#[cfg(feature = "trusted-literal")]
#[must_use]
pub fn trusted_literal(value: &'static str) -> TrustedLiteral {
    TrustedLiteral(value)
}

/// A marker wrapper for compile-time string literals that a product has opted
/// into treating as telemetry-safe.
#[cfg(feature = "trusted-literal")]
#[derive(Clone, Copy)]
pub struct TrustedLiteral(&'static str);

#[cfg(feature = "trusted-literal")]
impl telemetry_safe::ToTelemetry for TrustedLiteral {
    fn fmt_telemetry(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
