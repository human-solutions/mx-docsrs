use rustdoc_types::Id;

use crate::doc::tokens::Token;

/// Each public item (except `impl`s) have a path that is displayed like
/// `first::second::third`. Internally we represent that with a `vec!["first",
/// "second", "third"]`. This is a type alias for that internal representation
/// to make the code easier to read.
pub(crate) type PublicItemPath = Vec<String>;

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate.
#[derive(Clone)]
pub struct ListItem {
    /// Sortable path for grouping
    pub(crate) sortable_path: PublicItemPath,

    /// The rendered item as a stream of [`Token`]s
    pub(crate) tokens: Vec<Token>,

    /// The [`Id`] of this item's logical parent (if any)
    pub(crate) _parent_id: Option<Id>,

    /// The [`Id`] to which this public item corresponds
    pub(crate) _id: Id,
}
