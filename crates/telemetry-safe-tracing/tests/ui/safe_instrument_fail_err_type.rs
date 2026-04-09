use telemetry_safe_tracing::safe_instrument;

#[safe_instrument(err)]
fn create_response() -> Result<u64, String> {
    Err("secret".to_owned())
}

fn main() {}
