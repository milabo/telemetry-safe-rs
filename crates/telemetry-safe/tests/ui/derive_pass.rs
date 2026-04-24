#![deny(warnings)]

use telemetry_safe::{ToTelemetry, telemetry};
use std::fmt::{self, Formatter};

#[derive(ToTelemetry)]
struct UserId(u64);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(ToTelemetry)]
struct LoginAttempt {
    id: UserId,
    outcome: OutcomeLabel,
    #[telemetry(display = "user-{}")]
    id_label: UserId,
    #[telemetry("[redacted]")]
    note: String,
    #[telemetry(skip)]
    email: String,
}

struct OutcomeLabel(&'static str);

impl ToTelemetry for OutcomeLabel {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(ToTelemetry)]
enum InternalEvent {
    Audit {
        #[telemetry(display)]
        code: UserId,
        #[telemetry("[redacted]")]
        reason: String,
        #[telemetry(skip)]
        message: String,
    },
}

fn main() {
    let attempt = LoginAttempt {
        id: UserId(10),
        outcome: OutcomeLabel("accepted"),
        id_label: UserId(10),
        note: "internal".to_owned(),
        email: "user@example.com".to_owned(),
    };

    assert_eq!(
        telemetry(&attempt).to_string(),
        "LoginAttempt { id: UserId(10), outcome: accepted, id_label: user-10, note: [redacted] }"
    );

    let event = InternalEvent::Audit {
        code: UserId(20),
        reason: "restricted".to_owned(),
        message: "sensitive".to_owned(),
    };

    assert_eq!(
        telemetry(&event).to_string(),
        "Audit { code: 20, reason: [redacted] }"
    );
}
