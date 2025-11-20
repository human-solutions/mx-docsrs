use crate::fmt::tokens_to_string;

use super::public_item::PublicItem;

/// Matches items based on a pattern using suffix-first, then substring fallback strategy.
///
/// Algorithm:
/// 1. Find all items where rendered text ends with the pattern (suffix match)
/// 2. If exactly 1 suffix match found -> return it
/// 3. Otherwise, find all items where rendered text contains the pattern (substring match)
/// 4. If exactly 1 substring match found -> return it
/// 5. Otherwise -> return all matches found (for list display)
///
/// Note: Matching is done on the rendered token output (what the user sees),
/// not on the internal sortable_path.
pub fn match_items(items: Vec<PublicItem>, pattern: &str) -> Vec<PublicItem> {
    if pattern.is_empty() {
        return items;
    }

    // Try suffix matching first (most specific)
    let suffix_matches: Vec<PublicItem> = items
        .iter()
        .filter(|item| {
            let rendered = tokens_to_string(&item.tokens);
            rendered.ends_with(pattern)
        })
        .cloned()
        .collect();

    if suffix_matches.len() == 1 {
        return suffix_matches;
    }

    // Fall back to substring matching (more general)
    let substring_matches: Vec<PublicItem> = items
        .into_iter()
        .filter(|item| {
            let rendered = tokens_to_string(&item.tokens);
            rendered.contains(pattern)
        })
        .collect();

    if substring_matches.len() == 1 {
        return substring_matches;
    }

    // Return all matches (could be suffix matches or substring matches)
    // If we had suffix matches but not exactly 1, return those
    // Otherwise return substring matches (which includes suffix matches)
    if suffix_matches.len() > 1 {
        suffix_matches
    } else {
        substring_matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fmt::Token;
    use rustdoc_types::Id;

    fn create_test_item(path: Vec<&str>, rendered: &str) -> PublicItem {
        PublicItem {
            sortable_path: path.iter().map(|s| s.to_string()).collect(),
            // Store the rendered text as a single token for testing purposes
            tokens: vec![Token::Identifier(rendered.to_string())],
            _parent_id: None,
            _id: Id(0),
        }
    }

    #[test]
    fn test_empty_pattern_returns_all() {
        let items = vec![
            create_test_item(vec!["foo", "Bar"], "pub struct foo::Bar"),
            create_test_item(vec!["baz", "Qux"], "pub struct baz::Qux"),
        ];
        let result = match_items(items.clone(), "");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_single_suffix_match() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "HashMap"],
                "pub use test_reexports::HashMap",
            ),
            create_test_item(
                vec!["test_reexports", "InnerStruct"],
                "pub struct test_reexports::InnerStruct",
            ),
        ];
        let result = match_items(items, "HashMap");
        assert_eq!(result.len(), 1);
        assert_eq!(
            tokens_to_string(&result[0].tokens),
            "pub use test_reexports::HashMap"
        );
    }

    #[test]
    fn test_multiple_suffix_matches_returns_all() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "InnerStruct"],
                "pub struct test_reexports::InnerStruct",
            ),
            create_test_item(
                vec!["test_reexports", "reexported", "InnerStruct"],
                "pub struct test_reexports::reexported::InnerStruct",
            ),
        ];
        let result = match_items(items, "InnerStruct");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_suffix_with_path_components() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "InnerStruct"],
                "pub struct test_reexports::InnerStruct",
            ),
            create_test_item(
                vec!["test_reexports", "reexported", "InnerStruct"],
                "pub struct test_reexports::reexported::InnerStruct",
            ),
        ];
        let result = match_items(items, "reexported::InnerStruct");
        assert_eq!(result.len(), 1);
        assert_eq!(
            tokens_to_string(&result[0].tokens),
            "pub struct test_reexports::reexported::InnerStruct"
        );
    }

    #[test]
    fn test_substring_fallback_single_match() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "HashMap"],
                "pub use test_reexports::HashMap",
            ),
            create_test_item(
                vec!["test_reexports", "reexported", "InnerStruct"],
                "pub struct test_reexports::reexported::InnerStruct",
            ),
        ];
        // "reexported" doesn't match as suffix, but matches as substring
        let result = match_items(items, "reexported");
        assert_eq!(result.len(), 1);
        assert_eq!(
            tokens_to_string(&result[0].tokens),
            "pub struct test_reexports::reexported::InnerStruct"
        );
    }

    #[test]
    fn test_substring_fallback_multiple_matches() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "reexported", "InnerStruct"],
                "pub struct test_reexports::reexported::InnerStruct",
            ),
            create_test_item(
                vec!["test_reexports", "reexported", "InnerEnum"],
                "pub enum test_reexports::reexported::InnerEnum",
            ),
            create_test_item(
                vec!["test_reexports", "HashMap"],
                "pub use test_reexports::HashMap",
            ),
        ];
        let result = match_items(items, "reexported");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_no_matches() {
        let items = vec![
            create_test_item(
                vec!["test_reexports", "HashMap"],
                "pub use test_reexports::HashMap",
            ),
            create_test_item(
                vec!["test_reexports", "InnerStruct"],
                "pub struct test_reexports::InnerStruct",
            ),
        ];
        let result = match_items(items, "NonExistent");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_case_sensitive() {
        let items = vec![create_test_item(
            vec!["test_reexports", "InnerStruct"],
            "pub struct test_reexports::InnerStruct",
        )];
        let result = match_items(items, "innerstruct");
        assert_eq!(result.len(), 0);
    }
}
