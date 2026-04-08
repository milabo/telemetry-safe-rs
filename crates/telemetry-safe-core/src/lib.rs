//! Core primitives for compile-time safe telemetry formatting.

use std::fmt::{self, Debug, Display, Formatter};

/// Formats a value only through an explicitly approved telemetry representation.
pub trait ToTelemetry {
    /// Writes the representation that may leave the process boundary.
    ///
    /// `fmt` keeps adapters allocation-free so high-volume telemetry paths do not
    /// need a parallel "safe String" API just to satisfy backends like tracing.
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

/// Wraps a value so telemetry-safe formatting can be used where `Display` is expected.
#[must_use]
pub struct TelemetryDisplay<'a, T: ?Sized>(&'a T);

impl<'a, T: ToTelemetry + ?Sized> TelemetryDisplay<'a, T> {
    /// Creates a `Display` adapter for telemetry backends that accept `%value`.
    pub fn new(value: &'a T) -> Self {
        Self(value)
    }
}

impl<T: ToTelemetry + ?Sized> Display for TelemetryDisplay<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt_telemetry(f)
    }
}

/// Wraps a value so derive output can feed `debug_*` builders without exposing `Debug`.
#[must_use]
pub struct TelemetryDebug<'a, T: ?Sized>(&'a T);

impl<'a, T: ToTelemetry + ?Sized> TelemetryDebug<'a, T> {
    /// Creates a `Debug` adapter for APIs that only accept `?value`-style inputs.
    ///
    /// Keeping this separate from `Display` avoids accidentally widening the
    /// surface area to the ambient `Debug` implementation of the wrapped type.
    pub fn new(value: &'a T) -> Self {
        Self(value)
    }
}

impl<T: ToTelemetry + ?Sized> Debug for TelemetryDebug<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt_telemetry(f)
    }
}

/// Exposes a telemetry-safe value as `Display`.
pub fn telemetry<T: ToTelemetry + ?Sized>(value: &T) -> TelemetryDisplay<'_, T> {
    TelemetryDisplay::new(value)
}

/// Exposes a telemetry-safe value as `Debug`.
pub fn telemetry_debug<T: ToTelemetry + ?Sized>(value: &T) -> TelemetryDebug<'_, T> {
    TelemetryDebug::new(value)
}

macro_rules! impl_to_telemetry_via_display {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ToTelemetry for $ty {
                fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    Display::fmt(self, f)
                }
            }
        )*
    };
}

impl_to_telemetry_via_display!(
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6
);

impl<T: ToTelemetry + ?Sized> ToTelemetry for &T {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        (*self).fmt_telemetry(f)
    }
}

impl<T: ToTelemetry + ?Sized> ToTelemetry for Box<T> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt_telemetry(f)
    }
}

impl<T: ToTelemetry> ToTelemetry for Option<T> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Some(value) => f
                .debug_tuple("Some")
                .field(&telemetry_debug(value))
                .finish(),
            None => f.write_str("None"),
        }
    }
}

impl<T: ToTelemetry, E: ToTelemetry> ToTelemetry for Result<T, E> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ok(value) => f.debug_tuple("Ok").field(&telemetry_debug(value)).finish(),
            Err(err) => f.debug_tuple("Err").field(&telemetry_debug(err)).finish(),
        }
    }
}

impl<T: ToTelemetry> ToTelemetry for [T] {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // The adapter keeps recursive safety checks on each element while still
        // producing the familiar collection syntax backend operators expect.
        let mut list = f.debug_list();
        for item in self {
            list.entry(&telemetry_debug(item));
        }
        list.finish()
    }
}

impl<T: ToTelemetry, const N: usize> ToTelemetry for [T; N] {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt_telemetry(f)
    }
}

impl<T: ToTelemetry> ToTelemetry for Vec<T> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt_telemetry(f)
    }
}

impl<K: ToTelemetry, V: ToTelemetry> ToTelemetry for std::collections::BTreeMap<K, V> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (key, value) in self {
            map.entry(&telemetry_debug(key), &telemetry_debug(value));
        }
        map.finish()
    }
}

impl<T: ToTelemetry> ToTelemetry for std::collections::BTreeSet<T> {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut set = f.debug_set();
        for item in self {
            set.entry(&telemetry_debug(item));
        }
        set.finish()
    }
}

/// Re-exports the small surface most applications need at call sites.
pub mod prelude {
    pub use crate::{ToTelemetry, telemetry, telemetry_debug};
}

#[cfg(test)]
mod tests {
    use super::{ToTelemetry, telemetry};
    use std::fmt::{self, Formatter};

    struct Token(u64);

    impl ToTelemetry for Token {
        fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "token-{}", self.0)
        }
    }

    #[test]
    fn primitives_and_manual_types_are_displayable() {
        assert_eq!(telemetry(&123_u64).to_string(), "123");
        assert_eq!(telemetry(&Token(7)).to_string(), "token-7");
        assert_eq!(telemetry(&Some(Token(2))).to_string(), "Some(token-2)");
    }

    #[test]
    fn collections_use_safe_rendering_recursively() {
        let values = vec![Token(1), Token(2)];
        assert_eq!(telemetry(&values).to_string(), "[token-1, token-2]");
    }
}
