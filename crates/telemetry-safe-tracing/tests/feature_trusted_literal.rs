#![cfg(feature = "trusted-literal")]

use telemetry_safe_tracing::{safe_instrument, trusted_literal};

#[safe_instrument(fields(message_type = %trusted_literal(message_type)))]
fn record_event(message_type: &'static str) {}

#[test]
fn trusted_literal_marks_static_strings_explicitly() {
    record_event("signup");
}
