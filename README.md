# telemetry-safe

[![CI](https://github.com/milabo/telemetry-safe-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/milabo/telemetry-safe-rs/actions/workflows/ci.yml)

`telemetry-safe` is a Rust library for allowing only explicitly approved representations to flow into telemetry.

With plain `tracing`, logging, or metrics code, it is easy to accidentally send function arguments or raw fields that contain personally identifiable information (PII).  
This crate makes that path opt-in at the type level: only values that implement `ToTelemetry` can enter telemetry output, so mistakes fail at compile time instead of relying on review discipline.

This matters most when telemetry is sent to external backends such as Datadog or OpenTelemetry.  
At that point, leakage is not just noisy logging. It becomes a security, compliance, and audit-cost problem.  
`telemetry-safe` intentionally prioritizes “only approved values can escape” over convenience.

## Goals

- Make telemetry output opt-in rather than opt-out
- Reject code that could emit PII at compile time
- Stay useful beyond `tracing`, so the same safe boundary can be reused for logging, metrics, and error reporting

## Why review discipline is not enough

`tracing` already provides `skip_all` and `fields(...)`, so a careful team can get fairly close to a safe setup.  
The problem is that this still assumes people remember to do the right thing every time. Forgetfulness, copy-paste drift, and review misses are still possible.

`telemetry-safe` takes a different stance: do not depend on attention and conventions when the compiler can enforce the boundary.  
The goal is not just “mark the safe values explicitly”, but “values that have not been declared safe should fail to compile”.

## What this gives you

- Security
  - Reduces the number of paths by which PII can reach observability backends
- Compile-time enforcement
  - Unsafe paths fail as type errors instead of depending on process
- DDD-friendly modeling
  - Safe representations can be defined at the value-object level, such as `UserId` or `OrderId`
- Backend independence
  - The same safety boundary can be reused across tracing, logging, metrics, and error reporting

## Which crate should you use?

For most users, there are only two crates to think about:

- `telemetry-safe`
  - The facade crate that exposes `ToTelemetry`, `telemetry(&value)`, and `#[derive(ToTelemetry)]`
  - Start here if you want to define safe representations at the type level
- `telemetry-safe-tracing`
  - The `tracing` integration crate that exposes `#[safe_instrument]` and `trusted_literal`
  - Add this when you want to use the same safety model inside `tracing`

There are internal crates such as `telemetry-safe-core` and `telemetry-safe-derive`, but most users should not need to depend on them directly.

## License

This project is available under either of the following licenses, at your option:

- MIT, see [LICENSE-MIT](https://github.com/milabo/telemetry-safe-rs/blob/main/LICENSE-MIT)
- Apache License 2.0, see [LICENSE-APACHE](https://github.com/milabo/telemetry-safe-rs/blob/main/LICENSE-APACHE)

## Installation

### 1. Start with `ToTelemetry`

```toml
[dependencies]
telemetry-safe = "0.1"
```

This gives you:

- `ToTelemetry`
- `telemetry(&value)`
- `telemetry_debug(&value)`
- `#[derive(ToTelemetry)]`

### 2. Add `safe_instrument` for `tracing`

```toml
[dependencies]
telemetry-safe = "0.1"
telemetry-safe-tracing = "0.1"
tracing = "0.1"
```

This lets you reuse the same `ToTelemetry` boundary inside `#[safe_instrument]`.  
In practice, the migration path is usually: define safe domain types first, then apply them to tracing.

### 3. Explicitly allow trusted string literals

```toml
[dependencies]
telemetry-safe = "0.1"
telemetry-safe-tracing = { version = "0.1", features = ["trusted-literal"] }
tracing = "0.1"
```

The `trusted-literal` feature is not enabled by default.  
It exists so a product can explicitly say “we allow `&'static str` literals, but only behind a visible marker”.  
That decision shows up in dependency configuration instead of being hidden in call sites.

## Getting started

### Start with `telemetry-safe`

```rust
use telemetry_safe::{telemetry, ToTelemetry};
use std::fmt::{self, Formatter};

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct LoginAttempt {
    user_id: UserId,
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

`telemetry(&value)` acts as a `Display` adapter.  
When integrating with `tracing`, the expected shape is `%telemetry(&value)`.

As a rule of thumb:

- import the derive macro through `telemetry_safe::ToTelemetry`
- import helper functions through `telemetry_safe::prelude::*`

### Add `telemetry-safe-tracing`

```rust,ignore
use std::fmt::{self, Formatter};
use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

struct UserId(u64);

impl ToTelemetry for UserId {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

#[safe_instrument(fields(user.id = %user_id))]
fn load_profile(user_id: UserId) {}
```

`safe_instrument` never records function arguments implicitly.  
Values reach the span only through explicit `%expr` fields and safe `ret` / `err` handling.

## Example that fails to compile

If you put `String` or `&str` directly inside a domain type, the derive macro requires `ToTelemetry`.  
That means raw strings with no explicit safety decision cannot flow into telemetry by default.

```compile_fail
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
```

## Current API surface

- `ToTelemetry`
  - Trait that defines a representation approved for external telemetry
- `telemetry(&value)`
  - `Display` adapter
- `telemetry_debug(&value)`
  - `Debug` adapter
- `#[derive(ToTelemetry)]`
  - Derive macro for structs and enums
- `#[telemetry(skip)]`
  - Omits a field from telemetry output
- `#[telemetry("{}")]`
  - Explicitly adopts the type’s `Display` output

## Safety policy

`telemetry-safe` is not a thin convenience wrapper for emitting telemetry.  
Its purpose is to make unsafe output paths structurally unavailable, using types and macros rather than review checklists.

That goal requires intentionally rejecting some otherwise convenient patterns.

- No blanket approval for `String` or `&str`
  - Strings must not blur the boundary between safe identifiers and unreviewed user input
- No implicit approval for fixed strings either
  - If literals are allowed, they must pass through an explicit marker such as feature-gated `trusted_literal`
- No blind trust in ambient `Debug` or `Display`
  - Existing impls may already contain PII, and trait names alone do not prove safety
- Prefer fail-closed behavior over backend-specific convenience
  - A permissive shortcut tends to become the most dangerous escape hatch in the whole API

This policy is applied most strictly in `telemetry-safe-tracing` and `#[safe_instrument]`.

- Default argument recording is always disabled
- `fields(...)` only allows explicit `%expr` opt-in
- `?expr` is not allowed
- `ret` and `err` are allowed only with safe semantics that require `ToTelemetry`
- `&'static str` is not allowed implicitly

`err` and `ret` are convenient, but the standard `tracing` semantics delegate to ambient `Debug` / `Display` for the whole error or return value.  
That makes accidental leakage much more likely.  
For that reason, `safe_instrument(err)` and `safe_instrument(ret)` are implemented with separate semantics that always require `ToTelemetry`.

Likewise, the default behavior of `tracing::instrument` is to record function arguments through `Debug`, so `safe_instrument` always behaves as if `skip_all` were present.  
Telemetry values must be explicitly opted in through `fields(...)`, `ret`, or `err`.

```rust,ignore
use std::fmt::{self, Formatter};
use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

struct DomainError;

impl ToTelemetry for DomainError {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("denied")
    }
}

#[safe_instrument(err)]
fn do_work() -> Result<(), DomainError> {
    Err(DomainError)
}
```

In this example, `DomainError: ToTelemetry` is required.  
An unchecked type or raw string would fail to compile.

If your product chooses to allow only fixed string literals, enable the `trusted-literal` feature on `telemetry-safe-tracing` and use an explicit marker such as `%trusted_literal("signup")`.  
Even then, this is limited to `&'static str`; general `&str` still does not pass.

## Workspace layout

- `crates/telemetry-safe-core`
  - Minimal core crate containing the trait and adapters
- `crates/telemetry-safe-derive`
  - `#[derive(ToTelemetry)]` and field attributes
- `crates/telemetry-safe`
  - Facade crate intended for normal application use
- `crates/telemetry-safe-tracing`
  - Public `tracing` integration entry point
- `crates/telemetry-safe-tracing-macros`
  - Attribute macro implementation crate for `safe_instrument` and related macros

The tracing integration lives in a separate crate so the core safety model can stay backend-agnostic.  
That separation also avoids letting proc-macro and `tracing` constraints leak into the core API.

## Design notes

- `fmt`-based rendering avoids unnecessary allocation on high-frequency telemetry paths
- `Debug` is not accepted wholesale because existing impls may already contain PII
- `String` is not blanket-approved because “safe string” and “unchecked string” must remain distinct
- `&'static str` is not blanket-approved because ownership and borrowing have nothing to do with safety approval
- The main crate stays backend-agnostic first; `tracing` support is intentionally layered on top

## How to try this in your product

The safest rollout path is usually:

1. Introduce value objects such as `UserId` or `OrderId` for the identifiers you already emit
2. Implement `ToTelemetry` for them, or derive it where appropriate
3. Mark PII-bearing fields such as `String`, `EmailAddress`, or `Name` with `#[telemetry(skip)]`
4. Replace direct `%value`, `format!`, or ad-hoc logging output with `telemetry(&value)`

This order makes the unresolved safety boundary visible as compile errors.  
That feedback is often the most useful part of the migration.
