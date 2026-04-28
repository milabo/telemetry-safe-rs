use telemetry_safe::ToTelemetry;

#[derive(ToTelemetry)]
struct InvalidFormat {
    #[telemetry(display = "{}-{}")]
    id: u64,
}

fn main() {}
