use rustdoc_types::Id;
use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::Hash;

use crate::doc::intermediate_public_item::IntermediatePublicItem;
use crate::doc::render::RenderingContext;
use crate::doc::tokens::Token;
use crate::doc::tokens::tokens_to_string;

/// Each public item (except `impl`s) have a path that is displayed like
/// `first::second::third`. Internally we represent that with a `vec!["first",
/// "second", "third"]`. This is a type alias for that internal representation
/// to make the code easier to read.
pub(crate) type PublicItemPath = Vec<String>;

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate.
#[derive(Clone)]
pub struct PublicItem {
    /// Sortable path for grouping
    pub(crate) sortable_path: PublicItemPath,

    /// The rendered item as a stream of [`Token`]s
    pub(crate) tokens: Vec<Token>,

    /// The [`Id`] of this item's logical parent (if any)
    pub(crate) _parent_id: Option<Id>,

    /// The [`Id`] to which this public item corresponds
    pub(crate) _id: Id,
}

impl PublicItem {
    pub(crate) fn from_intermediate_public_item(
        context: &RenderingContext,
        public_item: &IntermediatePublicItem<'_>,
    ) -> PublicItem {
        PublicItem {
            sortable_path: public_item.sortable_path(context),
            tokens: public_item.render_token_stream(context),
            _parent_id: public_item.parent_id(),
            _id: public_item.id(),
        }
    }

    /// Special version of cmp that groups items logically.
    pub fn grouping_cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This will make e.g. struct and struct fields be grouped together.
        if let Some(ordering) = different_or_none(&self.sortable_path, &other.sortable_path) {
            return ordering;
        }

        // Fall back to lexical sorting if the above is not sufficient
        self.to_string().cmp(&other.to_string())
    }
}

impl PartialEq for PublicItem {
    fn eq(&self, other: &Self) -> bool {
        self.tokens == other.tokens
    }
}

impl Eq for PublicItem {}

impl Hash for PublicItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tokens.hash(state);
    }
}

impl std::fmt::Debug for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Display for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tokens_to_string(&self.tokens))
    }
}

/// Returns `None` if two items are equal. Otherwise their ordering is returned.
fn different_or_none<T: Ord>(a: &T, b: &T) -> Option<Ordering> {
    match a.cmp(b) {
        Ordering::Equal => None,
        c => Some(c),
    }
}
