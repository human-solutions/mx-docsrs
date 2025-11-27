//! Test crate for re-export patterns in rustdoc JSON
//!
//! This crate contains various re-export patterns to test how the docsrs
//! tool handles `pub use` statements and item discovery through re-exports.

// ============================================================================
// Internal definitions (not directly exposed at crate root)
// ============================================================================

mod inner {
    /// A struct defined in inner module
    pub struct InnerStruct {
        pub field: String,
    }

    /// An enum defined in inner module
    pub enum InnerEnum {
        Variant1,
        Variant2(i32),
    }

    /// A function defined in inner module
    pub fn inner_function() -> String {
        String::from("inner")
    }

    /// A trait defined in inner module
    pub trait InnerTrait {
        fn method(&self);
    }

    /// A constant in inner module
    pub const INNER_CONST: i32 = 100;

    /// A type alias in inner module
    pub type InnerAlias = InnerStruct;
}

mod deeply {
    pub mod nested {
        pub mod module {
            /// A deeply nested struct
            pub struct DeeplyNestedItem {
                pub value: usize,
            }
        }
    }
}

// ============================================================================
// Simple re-exports
// ============================================================================

/// Re-export struct with original name
pub use inner::InnerStruct;

/// Re-export enum with original name
pub use inner::InnerEnum;

/// Re-export function with original name
pub use inner::inner_function;

// ============================================================================
// Renamed re-exports
// ============================================================================

/// Re-export struct with new name
pub use inner::InnerStruct as RenamedStruct;

/// Re-export function with new name
pub use inner::inner_function as renamed_function;

// ============================================================================
// Multiple item re-exports
// ============================================================================

/// Re-export multiple items in one statement
pub use inner::{INNER_CONST, InnerAlias, InnerTrait};

// ============================================================================
// Glob re-exports
// ============================================================================

/// Module that re-exports everything from inner
pub mod reexported {
    pub use crate::inner::*;
}

// ============================================================================
// Nested re-exports (re-exporting from deep modules)
// ============================================================================

/// Re-export from deeply nested module
pub use deeply::nested::module::DeeplyNestedItem;

// ============================================================================
// External crate re-exports
// ============================================================================

/// Re-export from std
pub use std::collections::HashMap;

/// Re-export from std with rename
pub use std::vec::Vec as MyVec;

// ============================================================================
// Re-export chains
// ============================================================================

mod intermediate {
    // This re-exports something that was already re-exported
    pub use crate::InnerStruct as IntermediateStruct;
}

/// Re-export of a re-export
pub use intermediate::IntermediateStruct as ChainedReexport;

// ============================================================================
// Re-exports from crate root
// ============================================================================

/// A struct defined at crate root
pub struct RootStruct {
    pub data: String,
}

/// Module that re-exports items from crate root
pub mod reroot {
    /// Re-export from parent (crate root)
    pub use crate::RootStruct;
}

// ============================================================================
// Private re-exports (shouldn't appear in public API)
// ============================================================================

#[allow(unused_imports)]
mod private_reexports {
    use crate::inner::InnerStruct as PrivateReexport;
}

// ============================================================================
// Selective re-exports from modules
// ============================================================================

pub mod selective {
    #[allow(dead_code)]
    mod internal {
        pub struct Foo;
        pub struct Bar;
        pub struct Baz;
    }

    // Only re-export some items from internal
    pub use internal::{Bar, Foo};
    // Baz is not re-exported
}

// ============================================================================
// Re-exports with different visibility than original
// ============================================================================

pub mod visibility_change {
    mod private_module {
        pub struct Item;
    }

    // Re-export public item from private module
    pub use private_module::Item as PublicItem;
}

// ============================================================================
// Type alias re-exports
// ============================================================================

pub mod type_aliases {
    use crate::inner::InnerStruct;

    /// Type alias
    pub type MyType = InnerStruct;
}

/// Re-export a type alias
pub use type_aliases::MyType;

// ============================================================================
// Trait re-exports with implementations
// ============================================================================

pub mod traits {
    pub trait MyTrait {
        fn do_something(&self);
    }

    pub struct TraitImpl;

    impl MyTrait for TraitImpl {
        fn do_something(&self) {}
    }
}

/// Re-export trait and implementation
pub use traits::{MyTrait, TraitImpl};
