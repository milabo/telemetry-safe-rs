use telemetry_safe::{ToTelemetry, telemetry};

#[derive(ToTelemetry)]
struct UnsafePayload {
    raw: String,
}

fn main() {
    let payload = UnsafePayload {
        raw: "secret".to_owned(),
    };

    let _ = telemetry(&payload).to_string();
}
