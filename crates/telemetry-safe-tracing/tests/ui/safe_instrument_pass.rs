#![deny(warnings)]

use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct AccountId(u64);

#[safe_instrument(
    level = "info",
    fields(user.id = %user_id, account.id = %account_id)
)]
fn record_login(user_id: UserId, account_id: AccountId, password: String) -> bool {
    let _ = (&user_id, &account_id, password);
    true
}

fn main() {
    let _ = record_login(UserId(1), AccountId(2), "hidden".to_owned());
}
