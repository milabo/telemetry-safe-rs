use telemetry_safe_tracing::safe_instrument;

#[safe_instrument(err)]
fn create_response() -> u64 {
    7
}

fn main() {}
