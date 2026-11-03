//! Internal macros.

/// Define a key enum from a single table of `Variant => ("name", index)`.
///
/// Each key's display name and LED / key-matrix index are written once, in
/// one place. The macro generates:
///
/// - the enum (variants in table order, so derived `Ord` is table order),
/// - `COUNT` and `ALL` (every key, in table order),
/// - `index()`, `name()`, `from_index()` and `from_name()` (case-insensitive).
///
/// A duplicated index is a compile error.
macro_rules! define_keys {
    (
        $(#[$meta:meta])*
        $vis:vis enum $Enum:ident {
            $( $Variant:ident => ($name:literal, $index:literal) ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $Enum {
            $(
                #[doc = concat!("Key ", $name, " (LED / key-matrix index ", stringify!($index), ").")]
                $Variant,
            )+
        }

        impl $Enum {
            /// Number of keys.
            pub const COUNT: usize = [$(stringify!($Variant)),+].len();

            /// Every key, in table order.
            pub const ALL: [$Enum; Self::COUNT] = [$($Enum::$Variant),+];

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
