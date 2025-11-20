use rustdoc_types::Crate;
use rustdoc_types::Id;
use rustdoc_types::ItemEnum;
use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::Hash;

use crate::doc::render::RenderingContext;
use crate::doc::tokens::Token;
use crate::doc::tokens::tokens_to_string;
use crate::proc::IntermediatePublicItem;
use crate::proc::ItemProcessor;
use crate::proc::NameableItem;

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
        let sortable_path = public_item
            .path()
            .iter()
            .map(|p| sortable_name(&p.item, context))
            .collect();
        PublicItem {
            sortable_path,
            tokens: context.token_stream(public_item).into_tokens(),
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

/// Extract public API from a crate.
pub(crate) fn public_api_in_crate(
    crate_: &Crate,
    item_processor: &ItemProcessor,
) -> Vec<PublicItem> {
    let context = RenderingContext {
        crate_,
        id_to_items: item_processor.id_to_items(),
    };

    item_processor
        .output
        .iter()
        .map(|item| PublicItem::from_intermediate_public_item(&context, item))
        .collect::<Vec<_>>()
}

/// The name that, when sorted on, will group items nicely. Is never shown
/// to a user.
fn sortable_name(nameable: &NameableItem, context: &RenderingContext) -> String {
    // Note that in order for the prefix to sort properly lexicographically,
    // we need to pad it with leading zeroes.
    let mut sortable_name = format!("{:0>3}-", nameable.sorting_prefix);

    if let Some(name) = nameable.name() {
        sortable_name.push_str(name);
    } else if let ItemEnum::Impl(impl_) = &nameable.item.inner {
        // In order for items of impls to be grouped together with its impl,
        // add the "name" of the impl to the sorting prefix. Ignore `!` when
        // sorting however, because that just messes the expected order up.
        sortable_name.push_str(&crate::doc::tokens::tokens_to_string(
            context
                .render_impl(impl_, &[], true /* disregard_negativity */)
                .tokens(),
        ));

        // If this is an inherent impl, additionally add the concatenated
        // names of all associated items to the "name" of the impl. This makes
        // multiple inherent impls group together, even if they have the
        // same "name".
        if impl_.trait_.is_none() {
            let mut assoc_item_names: Vec<&str> = impl_
                .items
                .iter()
                .filter_map(|id| context.crate_.index.get(id))
                .filter_map(|item| item.name.as_ref())
                .map(String::as_str)
                .collect();
            assoc_item_names.sort_unstable();

            sortable_name.push('-');
            sortable_name.push_str(&assoc_item_names.join("-"));
        }
    }

    sortable_name
}
