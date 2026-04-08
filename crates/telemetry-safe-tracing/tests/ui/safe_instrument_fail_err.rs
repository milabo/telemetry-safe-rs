use telemetry_safe_tracing::safe_instrument;

#[safe_instrument(err)]
fn do_work() {}

fn main() {}
