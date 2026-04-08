use telemetry_safe::{ToTelemetry, telemetry};

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct LoginAttempt {
    id: UserId,
    outcome: &'static str,
    #[telemetry(skip)]
    email: String,
}

fn main() {
    let attempt = LoginAttempt {
        id: UserId(10),
        outcome: "accepted",
        email: "user@example.com".to_owned(),
    };

    assert_eq!(
        telemetry(&attempt).to_string(),
        "LoginAttempt { id: UserId(10), outcome: accepted }"
    );
}
