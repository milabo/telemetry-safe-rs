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

#[doc(hidden)]
pub mod __private {
    use crate::tracing::{Span, field};
    use crate::{ToTelemetry, telemetry};

    /// Records return values through `ToTelemetry` so `safe_instrument(ret)`
    /// never falls back to ambient `Debug` or `Display`.
    pub fn record_ret<T: ToTelemetry>(span: &Span, value: &T) {
        span.record("ret", field::display(telemetry(value)));
    }

    /// Records only the error side of a `Result`, keeping `err` tied to the
    /// explicit telemetry boundary instead of `tracing`'s default formatting.
    pub fn record_err<R: SafeInstrumentResult>(span: &Span, result: &R) {
        result.record_err(span);
    }

    pub trait SafeInstrumentResult {
        fn record_err(&self, span: &Span);
    }

    impl<T, E: ToTelemetry> SafeInstrumentResult for Result<T, E> {
        fn record_err(&self, span: &Span) {
            if let Err(err) = self {
                span.record("err", field::display(telemetry(err)));
            }
        }
    }
}

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
