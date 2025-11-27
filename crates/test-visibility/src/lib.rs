#![allow(dead_code)]
//! Test crate for visibility levels in rustdoc JSON
//!
//! This crate contains items with various visibility modifiers to test
//! how the docsrs tool handles different visibility levels.

/// A fully public struct
pub struct PublicStruct {
    /// A public field
    pub public_field: String,
    /// A private field (should not be visible in docs)
    private_field: i32,
}

/// A public tuple struct with mixed visibility fields
pub struct PublicTupleStruct(pub String, i32);

/// A crate-visible struct
pub(crate) struct CrateVisibleStruct {
    pub field: String,
}

/// A private struct (should not appear in public docs)
struct PrivateStruct {
    field: String,
}

/// A public enum
pub enum PublicEnum {
    /// Public variant
    Variant1,
    /// Another public variant
    Variant2(String),
}

/// A crate-visible enum
pub(crate) enum CrateVisibleEnum {
    Variant,
}

/// A public function
pub fn public_function() -> String {
    String::from("public")
}

/// A crate-visible function
pub(crate) fn crate_visible_function() -> String {
    String::from("crate")
}

/// A private function
fn private_function() -> String {
    String::from("private")
}

/// Public module with nested visibility
pub mod public_module {
    /// Public item in public module
    pub struct NestedPublic;

    /// Crate-visible item in public module
    pub(crate) struct NestedCrateVisible;

    /// Super-visible item (visible to parent module)
    pub(super) struct NestedSuperVisible;

    /// Private item in public module
    struct NestedPrivate;

    /// Nested submodule
    pub mod inner {
        /// Public item in nested module
        pub struct DeeplyNested;

        /// Item visible to the outer module
        pub(in crate::public_module) struct VisibleToOuterModule;
    }
}

/// Crate-visible module
pub(crate) mod crate_module {
    /// Public item in crate-visible module
    pub struct ItemInCrateModule;
}

/// Private module (should not appear in docs)
mod private_module {
    pub struct ItemInPrivateModule;
}

/// A trait to test trait visibility
pub trait PublicTrait {
    /// Associated type
    type Item;

    /// Trait method
    fn method(&self) -> Self::Item;
}

/// Crate-visible trait
pub(crate) trait CrateVisibleTrait {
    fn method(&self);
}

/// Implementation block
impl PublicStruct {
    /// Public constructor
    pub fn new(public_field: String, private_field: i32) -> Self {
        Self {
            public_field,
            private_field,
        }
    }

    /// Crate-visible method
    pub(crate) fn crate_method(&self) {}

    /// Private method
    fn private_method(&self) {}
}

/// Public constant
pub const PUBLIC_CONST: i32 = 42;

/// Crate-visible constant
pub(crate) const CRATE_CONST: i32 = 24;

/// Private constant
const PRIVATE_CONST: i32 = 12;

/// Public type alias
pub type PublicAlias = PublicStruct;

/// Crate-visible type alias
pub(crate) type CrateAlias = CrateVisibleStruct;
