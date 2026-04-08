#![doc = include_str!("../../../README.md")]

// Derive macros refer to the public crate path so downstream crates and this
// crate's own tests expand identically.
extern crate self as telemetry_safe;

pub use telemetry_safe_core::{
    TelemetryDebug, TelemetryDisplay, ToTelemetry, prelude, telemetry, telemetry_debug,
};
pub use telemetry_safe_derive::ToTelemetry;

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use super::{ToTelemetry, telemetry};
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(ToTelemetry)]
    struct AccountSnapshot {
        id: u64,
        state: &'static str,
        #[telemetry(skip)]
        secret_note: &'static str,
    }

    #[derive(ToTelemetry)]
    enum Outcome {
        Accepted { account: AccountSnapshot },
        Rejected(RejectReason),
    }

    #[derive(ToTelemetry)]
    struct RejectReason {
        code: &'static str,
    }

    #[test]
    fn derived_struct_skips_sensitive_fields() {
        let snapshot = AccountSnapshot {
            id: 42,
            state: "active",
            secret_note: "pii",
        };

        assert_eq!(
            telemetry(&snapshot).to_string(),
            r#"AccountSnapshot { id: 42, state: active }"#
        );
    }

    #[test]
    fn derived_enum_formats_variants() {
        let outcome = Outcome::Rejected(RejectReason { code: "policy" });

        assert_eq!(
            telemetry(&outcome).to_string(),
            r#"Rejected(RejectReason { code: policy })"#
        );
    }

    #[test]
    fn collections_require_safe_elements() {
        let mut map = BTreeMap::new();
        map.insert(
            1_u64,
            AccountSnapshot {
                id: 7,
                state: "pending",
                secret_note: "hidden",
            },
        );

        let mut set = BTreeSet::new();
        set.insert(1_u64);

        assert_eq!(
            telemetry(&map).to_string(),
            r#"{1: AccountSnapshot { id: 7, state: pending }}"#
        );
        assert_eq!(telemetry(&set).to_string(), "{1}");
    }

    #[test]
    fn display_wrapper_can_be_used_in_format_args() {
        let snapshot = AccountSnapshot {
            id: 1,
            state: "active",
            secret_note: "hidden",
        };

        let rendered = format!("account={}", telemetry(&snapshot));

        assert_eq!(rendered, "account=AccountSnapshot { id: 1, state: active }");
    }
}
