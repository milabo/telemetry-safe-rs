use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

#[derive(ToTelemetry)]
struct UserId(u64);

#[safe_instrument(fields(user.id = ?user_id))]
fn record_login(user_id: UserId) {}

fn main() {}
