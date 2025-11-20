use rustdoc_types::Item;

/// Wraps an [`Item`] and allows us to override its name.
#[derive(Clone, Debug)]
pub struct NameableItem<'c> {
    /// The item we are effectively wrapping.
    pub item: &'c Item,

    /// If `Some`, this overrides [Item::name], which happens in the case of
    /// renamed imports (`pub use other::Item as Foo;`).
    ///
    /// We can't calculate this on-demand, because we can't know the final name
    /// until we have checked if we need to break import recursion.
    pub overridden_name: Option<String>,

    /// See [`crate::doc::item_processor::sorting_prefix()`] docs for an explanation why we have this.
    pub sorting_prefix: u8,
}

impl NameableItem<'_> {
    /// The regular name of the item. Shown to users.
    pub fn name(&self) -> Option<&str> {
        self.overridden_name
            .as_deref()
            .or(self.item.name.as_deref())
    }
}
