use telemetry_safe::ToTelemetry;

#[derive(ToTelemetry)]
struct LegacyDisplaySyntax {
    #[telemetry("{}")]
    id: u64,
}

fn main() {}
