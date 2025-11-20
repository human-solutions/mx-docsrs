use rustdoc_types::{Id, Item};

use crate::doc::nameable_item::NameableItem;
use crate::doc::path_component::PathComponent;
use crate::doc::public_item::PublicItemPath;
use crate::doc::render::RenderingContext;
use crate::doc::tokens::Token;

/// This struct represents one public item of a crate, but in intermediate form.
/// Conceptually it wraps a single [`Item`] even though the path to the item
/// consists of many [`Item`]s. Later, one [`Self`] will be converted to exactly
/// one [`PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'c> {
    path: Vec<PathComponent<'c>>,
    parent_id: Option<Id>,
    id: Id,
}

impl<'c> IntermediatePublicItem<'c> {
    pub fn new(path: Vec<PathComponent<'c>>, parent_id: Option<Id>, id: Id) -> Self {
        Self {
            path,
            parent_id,
            id,
        }
    }

    pub fn item(&self) -> &'c Item {
        self.path()
            .last()
            .expect("path must not be empty")
            .item
            .item
    }

    pub fn path(&self) -> &[PathComponent<'c>] {
        &self.path
    }

    pub fn parent_id(&self) -> Option<Id> {
        self.parent_id
    }

    pub fn id(&self) -> Id {
        self.id
    }

    /// See [`crate::doc::item_processor::sorting_prefix()`] docs for an explanation why we have this.
    pub fn sortable_path(&self, context: &RenderingContext) -> PublicItemPath {
        self.path()
            .iter()
            .map(|p| NameableItem::sortable_name(&p.item, context))
            .collect()
    }

    pub fn path_contains_renamed_item(&self) -> bool {
        self.path().iter().any(|m| m.item.overridden_name.is_some())
    }

    pub fn render_token_stream(&self, context: &RenderingContext) -> Vec<Token> {
        context.token_stream(self).into_tokens()
    }
}
