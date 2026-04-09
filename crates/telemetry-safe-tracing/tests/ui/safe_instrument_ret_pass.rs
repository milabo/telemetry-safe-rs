#![deny(warnings)]

use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

#[derive(ToTelemetry)]
struct ResponseId(u64);

#[safe_instrument(ret)]
fn create_response() -> ResponseId {
    ResponseId(42)
}

fn main() {
    let _ = create_response();
}
