use rustdoc_types::{Crate, Id, Item};

/// The [`Crate`] type represents the deserialized form of the rustdoc JSON
/// input. This wrapper adds some helpers and state on top.
pub struct CrateWrapper<'c> {
    crate_: &'c Crate,

    /// Normally, an item referenced by [`Id`] is present in the rustdoc JSON.
    /// If [`Self::crate_.index`] is missing an [`Id`], then we add it here, to
    /// aid with debugging.
    missing_ids: Vec<Id>,
}

impl<'c> CrateWrapper<'c> {
    pub fn new(crate_: &'c Crate) -> Self {
        Self {
            crate_,
            missing_ids: vec![],
        }
    }

    pub fn get_item(&mut self, id: Id) -> Option<&'c Item> {
        self.crate_.index.get(&id).or_else(|| {
            self.missing_ids.push(id);
            None
        })
    }

    /// Returns the crate root module ID.
    pub fn root(&self) -> Id {
        self.crate_.root
    }
}
