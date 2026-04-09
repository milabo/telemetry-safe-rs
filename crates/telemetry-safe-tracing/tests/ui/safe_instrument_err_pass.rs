#![deny(warnings)]

use std::fmt::{self, Formatter};
use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

struct DomainError;

impl ToTelemetry for DomainError {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("denied")
    }
}

#[safe_instrument(err)]
fn create_response() -> Result<u64, DomainError> {
    Err(DomainError)
}

fn main() {
    let _ = create_response();
}
