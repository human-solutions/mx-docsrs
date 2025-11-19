mod common;

use common::{find_items_by_name, find_reexports, generate_rustdoc_json, load_rustdoc_json};

#[test]
fn test_reexports_are_in_json() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let mut reexports = find_reexports(&krate);

    assert!(
        !reexports.is_empty(),
        "Should find re-export items in the JSON"
    );

    // Sort for deterministic snapshots
    reexports.sort_by(|a, b| a.1.source.cmp(&b.1.source));

    insta::assert_debug_snapshot!(reexports);
}

#[test]
fn test_reexport_simple() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    let inner_struct_reexport = reexports.iter().find(|(_path, use_item)| {
        use_item.source == "inner::InnerStruct" || use_item.source.ends_with("::inner::InnerStruct")
    });

    assert!(
        inner_struct_reexport.is_some(),
        "Should find re-export of InnerStruct"
    );

    insta::assert_snapshot!(format!("{:#?}", inner_struct_reexport), @r#"
    Some(
        (
            "<unknown>",
            Use {
                source: "inner::InnerStruct",
                name: "InnerStruct",
                id: Some(
                    Id(
                        2,
                    ),
                ),
                is_glob: false,
            },
        ),
    )
    "#);
}

#[test]
fn test_reexport_renamed() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let renamed_results = find_items_by_name(&krate, "RenamedStruct");

    insta::assert_snapshot!(format!("{:#?}", renamed_results), @"[]");
}

#[test]
fn test_reexport_multiple_items() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let trait_results = find_items_by_name(&krate, "InnerTrait");
    let const_results = find_items_by_name(&krate, "INNER_CONST");
    let alias_results = find_items_by_name(&krate, "InnerAlias");

    assert!(
        !trait_results.is_empty(),
        "Should find InnerTrait re-export"
    );
    assert!(
        !const_results.is_empty(),
        "Should find INNER_CONST re-export"
    );
    assert!(
        !alias_results.is_empty(),
        "Should find InnerAlias re-export"
    );

    insta::assert_snapshot!(format!("{:#?}", (trait_results, const_results, alias_results)), @r#"
    (
        [
            "test_reexports::inner::InnerTrait::InnerTrait (Trait(Trait { is_auto: false, is_unsafe: false, is_dyn_compatible: true, items: [Id(60)], generics: Generics { params: [], where_predicates: [] }, bounds: [], implementations: [] }), vis: Public)",
        ],
        [
            "test_reexports::inner::INNER_CONST::INNER_CONST (Constant { type_: Primitive(\"i32\"), const_: Constant { expr: \"100\", value: Some(\"100i32\"), is_literal: true } }, vis: Public)",
        ],
        [
            "test_reexports::inner::InnerAlias::InnerAlias (TypeAlias(TypeAlias { type_: ResolvedPath(Path { path: \"InnerStruct\", id: Id(2), args: None }), generics: Generics { params: [], where_predicates: [] } }), vis: Public)",
        ],
    )
    "#);
}

#[test]
fn test_reexport_glob() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    let glob_reexports: Vec<_> = reexports
        .iter()
        .filter(|(path, use_item)| path.contains("reexported") && use_item.source.contains("inner"))
        .collect();

    insta::assert_snapshot!(format!("{:#?}", glob_reexports), @"[]");
}

#[test]
fn test_reexport_nested_module() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let results = find_items_by_name(&krate, "DeeplyNestedItem");

    assert!(
        !results.is_empty(),
        "Should find DeeplyNestedItem re-export"
    );

    insta::assert_snapshot!(format!("{:#?}", results), @r#"
    [
        "test_reexports::deeply::nested::module::DeeplyNestedItem::DeeplyNestedItem (Struct(Struct { kind: Plain { fields: [Id(65)], has_stripped_fields: false }, generics: Generics { params: [], where_predicates: [] }, impls: [Id(67), Id(68), Id(69), Id(70), Id(71), Id(72), Id(73), Id(74), Id(75), Id(76), Id(77), Id(78), Id(79)] }), vis: Public)",
    ]
    "#);
}

#[test]
fn test_reexport_external_crate() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let reexports = find_reexports(&krate);

    let mut std_reexports: Vec<_> = reexports
        .iter()
        .filter(|(_, import)| import.source.starts_with("std::"))
        .collect();

    assert!(
        !std_reexports.is_empty(),
        "Should find re-exports from std library"
    );

    // Sort for deterministic snapshots
    std_reexports.sort_by(|a, b| a.1.source.cmp(&b.1.source));

    insta::assert_snapshot!(format!("{:#?}", std_reexports), @r#"
    [
        (
            "<unknown>",
            Use {
                source: "std::collections::HashMap",
                name: "HashMap",
                id: Some(
                    Id(
                        171,
                    ),
                ),
                is_glob: false,
            },
        ),
        (
            "<unknown>",
            Use {
                source: "std::vec::Vec",
                name: "MyVec",
                id: Some(
                    Id(
                        173,
                    ),
                ),
                is_glob: false,
            },
        ),
    ]
    "#);
}

#[test]
fn test_reexport_chain() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let results = find_items_by_name(&krate, "ChainedReexport");

    insta::assert_snapshot!(format!("{:#?}", results), @"[]");
}

#[test]
fn test_reexport_visibility_change() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let results = find_items_by_name(&krate, "PublicItem");

    insta::assert_snapshot!(format!("{:#?}", results), @"[]");
}

#[test]
fn test_reexport_selective() {
    let json_path = generate_rustdoc_json("test-reexports");
    let krate = load_rustdoc_json(&json_path);

    let foo_results = find_items_by_name(&krate, "Foo");
    let bar_results = find_items_by_name(&krate, "Bar");
    let baz_results = find_items_by_name(&krate, "Baz");

    assert!(!foo_results.is_empty(), "Should find Foo re-export");
    assert!(!bar_results.is_empty(), "Should find Bar re-export");

    insta::assert_snapshot!(format!("{:#?}", (foo_results, bar_results, baz_results)), @r#"
    (
        [
            "test_reexports::selective::internal::Foo::Foo (Struct(Struct { kind: Unit, generics: Generics { params: [], where_predicates: [] }, impls: [Id(92), Id(93), Id(94), Id(95), Id(96), Id(97), Id(98), Id(99), Id(100), Id(101), Id(102), Id(103), Id(104)] }), vis: Public)",
        ],
        [
            "test_reexports::selective::internal::Bar::Bar (Struct(Struct { kind: Unit, generics: Generics { params: [], where_predicates: [] }, impls: [Id(106), Id(107), Id(108), Id(109), Id(110), Id(111), Id(112), Id(113), Id(114), Id(115), Id(116), Id(117), Id(118)] }), vis: Public)",
        ],
        [],
    )
    "#);
}
