use rustdoc_types::Id;
use std::fmt::Display;
use std::hash::Hash;

use crate::doc::render::RenderingContext;
use crate::fmt::Token;
use crate::fmt::tokens_to_string;
use crate::proc::IntermediatePublicItem;

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate.
#[derive(Clone)]
pub struct PublicItem {
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
            tokens: context.token_stream(public_item).into_tokens(),
            _parent_id: public_item.parent_id(),
            _id: public_item.id(),
        }
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
