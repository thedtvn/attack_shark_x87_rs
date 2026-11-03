//! Internal macros, one per file.
//!
//! - [`enum_meta!`]: declare a fieldless enum once; generates `COUNT`, `ALL`,
//!   `variant_name()` and `from_variant_name()`.
//! - [`define_keys!`]: the physical key table (name + index per key), built on
//!   `enum_meta!`.

mod define_keys;
mod enum_meta;

pub(crate) use define_keys::define_keys;
pub(crate) use enum_meta::enum_meta;

#[cfg(test)]
mod tests {
    super::enum_meta! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        enum Sample {
            A = 3,
            /// Documented variant.
            B = 7,
            C,
        }
    }

    #[test]
    fn enum_meta_generates_list_and_names() {
        assert_eq!(Sample::COUNT, 3);
        assert_eq!(Sample::ALL, [Sample::A, Sample::B, Sample::C]);
        assert_eq!(Sample::B as u8, 7);
        assert_eq!(Sample::C as u8, 8, "implicit discriminant follows the previous");
        for v in Sample::ALL {
            assert_eq!(Sample::from_variant_name(v.variant_name()), Some(v));
        }
        assert_eq!(Sample::A.variant_name(), "A");
        assert_eq!(Sample::from_variant_name("a"), None, "exact match");
    }
}
