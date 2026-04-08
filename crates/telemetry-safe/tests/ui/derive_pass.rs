#![deny(warnings)]

use telemetry_safe::{ToTelemetry, telemetry};
use std::fmt::{self, Formatter};

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct LoginAttempt {
    id: UserId,
    outcome: OutcomeLabel,
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
        code: UserId,
        #[telemetry(skip)]
        message: String,
    },
}

fn main() {
    let attempt = LoginAttempt {
        id: UserId(10),
        outcome: OutcomeLabel("accepted"),
        email: "user@example.com".to_owned(),
    };

    assert_eq!(
        telemetry(&attempt).to_string(),
        "LoginAttempt { id: UserId(10), outcome: accepted }"
    );

    let event = InternalEvent::Audit {
        code: UserId(20),
        message: "sensitive".to_owned(),
    };

    assert_eq!(telemetry(&event).to_string(), "Audit { code: UserId(20) }");
}
