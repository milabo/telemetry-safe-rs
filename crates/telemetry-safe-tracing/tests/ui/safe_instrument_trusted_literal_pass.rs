#![deny(warnings)]

use telemetry_safe_tracing::{safe_instrument, trusted_literal};

#[safe_instrument(fields(message_type = %trusted_literal(message_type)))]
fn record_event(message_type: &'static str) {}

fn main() {
    record_event("signup");
}
