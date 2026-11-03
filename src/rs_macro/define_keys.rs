//! `define_keys!`: the physical key table, written once.

/// Define a key enum from a single table of `Variant => ("name", index)`.
///
/// Each key's display name and LED / key-matrix index are written once, in
/// one place. Builds on [`enum_meta!`](super::enum_meta), so the enum also
/// gets `COUNT`, `ALL`, `variant_name()` and `from_variant_name()`. Adds:
///
/// - `index()`, `name()`,
/// - `from_index()` and `from_name()` (case-insensitive display name).
///
/// A duplicated index is a compile error.
macro_rules! define_keys {
    (
        $(#[$meta:meta])*
        $vis:vis enum $Enum:ident {
            $( $Variant:ident => ($name:literal, $index:literal) ),+ $(,)?
        }
    ) => {
        $crate::rs_macro::enum_meta! {
            $(#[$meta])*
            $vis enum $Enum {
                $(
                    #[doc = concat!("Key ", $name, " (LED / key-matrix index ", stringify!($index), ").")]
                    $Variant,
                )+
            }
        }

        impl $Enum {
            /// LED / key-matrix index.
            pub const fn index(self) -> u8 {
                match self {
                    $($Enum::$Variant => $index,)+
                }
            }

            /// Display name, as used by the vendor app and the web app.
            pub const fn name(self) -> &'static str {
                match self {
                    $($Enum::$Variant => $name,)+
                }
            }

            /// The key at an LED / key-matrix index, if any.
            pub const fn from_index(index: u8) -> Option<Self> {
                match index {
                    $($index => Some($Enum::$Variant),)+
                    _ => None,
                }
            }

            /// Look up a key by display name (case-insensitive).
            pub fn from_name(name: &str) -> Option<Self> {
                Self::ALL.into_iter().find(|k| k.name().eq_ignore_ascii_case(name))
            }
        }

        // Every index must be unique: checked when the crate compiles.
        const _: () = {
            let indices: [u8; $Enum::COUNT] = [$($index),+];
            let mut i = 0;
            while i < indices.len() {
                let mut j = i + 1;
                while j < indices.len() {
                    assert!(indices[i] != indices[j], "duplicate key index in define_keys!");
                    j += 1;
                }
                i += 1;
            }
        };
    };
}

pub(crate) use define_keys;
