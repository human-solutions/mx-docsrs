//! JSON documentation processor for rustdoc output.
//!
//! This crate processes `rustdoc_types::Crate` data and provides
//! structured access to public API items with their full paths.

mod crate_wrapper;
mod impl_kind;
mod item_ext;
mod jsondoc;
mod jsondoc_item;
mod nameable_item;
mod path_component;
mod unprocessed_item;

pub use impl_kind::ImplKind;
pub use jsondoc::JsonDoc;
pub use jsondoc_item::JsonDocItem;
pub use nameable_item::NameableItem;
pub use path_component::PathComponent;
