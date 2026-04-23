use telemetry_safe::ToTelemetry;

#[derive(ToTelemetry)]
struct InvalidFormat {
    #[telemetry("{}-{}")]
    id: u64,
}

fn main() {}
