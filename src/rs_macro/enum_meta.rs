//! `enum_meta!`: declare a fieldless enum once and get its variant list and
//! names generated, instead of hand-writing `ALL`, name matches and lookups.

/// Define a fieldless enum and generate, from the single variant list:
///
/// - `COUNT`: number of variants,
/// - `ALL`: every variant, in declaration order,
/// - `variant_name()`: the Rust variant name, e.g. `"Receiver24G"` (a stable
///   identifier for bindings, config files and logs),
/// - `from_variant_name()`: the reverse lookup (exact match).
///
/// Attributes (`#[derive]`, `#[repr]`, docs) and explicit discriminants
/// (`Name = 0x07`) are passed through unchanged; each variant may have its
/// own doc comments and attributes.
///
/// ```ignore
/// enum_meta! {
///     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
///     #[repr(u8)]
///     pub enum Effect {
///         FixedOn = 1,
///         /// Per-key colours.
///         Custom = 21,
///     }
/// }
/// assert_eq!(Effect::ALL, [Effect::FixedOn, Effect::Custom]);
/// assert_eq!(Effect::Custom.variant_name(), "Custom");
/// ```
macro_rules! enum_meta {
    (
        $(#[$meta:meta])*
        $vis:vis enum $Enum:ident {
            $(
                $(#[$vmeta:meta])*
                $Variant:ident $(= $value:expr)?
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $Enum {
            $(
                $(#[$vmeta])*
                $Variant $(= $value)?,
            )+
        }

        impl $Enum {
            /// Number of variants.
            pub const COUNT: usize = [$(stringify!($Variant)),+].len();

            /// Every variant, in declaration order.
            pub const ALL: [$Enum; Self::COUNT] = [$($Enum::$Variant),+];

            /// Rust variant name, e.g. for bindings, config files and logs.
            pub const fn variant_name(self) -> &'static str {
                match self {
                    $($Enum::$Variant => stringify!($Variant),)+
                }
            }

            /// The variant with this Rust name (exact match), if any.
            pub fn from_variant_name(name: &str) -> Option<Self> {
                match name {
                    $(stringify!($Variant) => Some($Enum::$Variant),)+
                    _ => None,
                }
            }
        }
    };
}

pub(crate) use enum_meta;
