# telemetry-safe

`telemetry-safe` is the main facade crate for defining which values are allowed
to flow into telemetry.

It re-exports:

- `ToTelemetry`
- `telemetry(&value)`
- `telemetry_debug(&value)`
- `#[derive(ToTelemetry)]`

Use this crate when you want compile-time enforcement that only explicitly
approved representations can be emitted through logging, tracing, metrics, or
other observability paths.

## Why this crate exists

Plain `tracing`, logging, or metrics code makes it easy to accidentally send
PII through `Debug`, `Display`, or default instrumentation behavior.

`telemetry-safe` takes an opt-in, type-driven approach instead:

- values must implement `ToTelemetry` before they can be emitted
- raw `String` / `&str` are not blanket-approved
- unsafe paths fail at compile time rather than depending on review discipline

## Basic example

```rust
use telemetry_safe::{telemetry, ToTelemetry};
use std::fmt::{self, Formatter};

#[derive(ToTelemetry)]
struct UserId(u64);

struct OutcomeLabel(&'static str);

impl ToTelemetry for OutcomeLabel {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(ToTelemetry)]
struct LoginAttempt {
    user_id: UserId,
    outcome: OutcomeLabel,
    #[telemetry(skip)]
    email: String,
}

let attempt = LoginAttempt {
    user_id: UserId(42),
    outcome: OutcomeLabel("accepted"),
    email: "user@example.com".to_owned(),
};

assert_eq!(
    telemetry(&attempt).to_string(),
    "LoginAttempt { user_id: UserId(42), outcome: accepted }",
);
```

## Related crates

- `telemetry-safe-core`
  - the minimal backend-agnostic core
- `telemetry-safe-tracing`
  - `tracing` integration, including `#[safe_instrument]`

For the full project overview and workspace-level documentation, see the
repository README at <https://github.com/milabo/telemetry-safe-rs>.
