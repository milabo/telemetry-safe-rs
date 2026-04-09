#![deny(warnings)]

use std::fmt::{self, Formatter};
use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

#[derive(ToTelemetry)]
struct ResponseId(u64);

struct DomainError;

impl ToTelemetry for DomainError {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("denied")
    }
}

#[safe_instrument(ret, err, fields(flow.id = %ResponseId(7)))]
async fn create_response(ok: bool) -> Result<ResponseId, DomainError> {
    if ok {
        Ok(ResponseId(1))
    } else {
        Err(DomainError)
    }
}

fn main() {
    let _ = create_response(true);
}
