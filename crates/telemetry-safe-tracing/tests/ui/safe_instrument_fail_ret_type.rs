use telemetry_safe_tracing::safe_instrument;

#[safe_instrument(ret)]
fn create_response() -> String {
    "secret".to_owned()
}

fn main() {}
