use rustdoc_types::{Id, Item};

use crate::path_component::PathComponent;

/// This struct represents one public item of a crate.
/// Conceptually it wraps a single [`Item`] even though the path to the item
/// consists of many [`Item`]s.
#[derive(Clone, Debug)]
pub struct JsonDocItem<'c> {
    path: Vec<PathComponent<'c>>,
    parent_id: Option<Id>,
    id: Id,
}

impl<'c> JsonDocItem<'c> {
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

    pub fn path_contains_renamed_item(&self) -> bool {
        self.path().iter().any(|m| m.item.overridden_name.is_some())
    }
}
