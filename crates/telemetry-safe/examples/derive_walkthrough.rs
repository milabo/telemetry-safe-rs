#![deny(warnings)]
#![allow(dead_code)]

use std::fmt::{self, Formatter};
use telemetry_safe::{ToTelemetry, telemetry};

#[derive(ToTelemetry)]
struct UserId(u64);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(ToTelemetry)]
struct LoginAttempt {
    user_id: UserId,
    outcome: OutcomeLabel,
    // Try removing the attribute.
    // The derive will stop compiling because raw strings are not
    // telemetry-safe by default.
    #[telemetry(display = "user-{}")]
    user_label: String,
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
        // Try changing this field to `#[telemetry(display)]` to see how
        // explicit Display escape hatches differ from fixed redaction labels.
        #[telemetry("[redacted]")]
        reason: String,
        #[telemetry(skip)]
        message: String,
    },
}

fn main() {
    let attempt = LoginAttempt {
        user_id: UserId(42),
        outcome: OutcomeLabel("accepted"),
        user_label: "User".to_string(),
        note: "internal-only".to_owned(),
        email: "user@example.com".to_owned(),
    };

    assert_eq!(
        telemetry(&attempt).to_string(),
        "LoginAttempt { user_id: UserId(42), outcome: accepted, user_label: user-User, note: [redacted] }"
    );

    let event = InternalEvent::Audit {
        code: UserId(7),
        reason: "restricted".to_owned(),
        message: "contains raw customer input".to_owned(),
    };

    assert_eq!(
        telemetry(&event).to_string(),
        "Audit { code: 7, reason: [redacted] }"
    );

    println!("{}", telemetry(&attempt));
    println!("{}", telemetry(&event));
}
