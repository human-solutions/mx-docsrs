mod common;

use common::{find_items_by_name, generate_rustdoc_json, load_rustdoc_json};

#[test]
fn test_visibility_public_items_are_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let public_struct_results = find_items_by_name(&krate, "PublicStruct");
    assert!(
        !public_struct_results.is_empty(),
        "PublicStruct should be found in rustdoc JSON"
    );

    let public_function_results = find_items_by_name(&krate, "public_function");
    assert!(
        !public_function_results.is_empty(),
        "public_function should be found in rustdoc JSON"
    );

    let public_enum_results = find_items_by_name(&krate, "PublicEnum");
    assert!(
        !public_enum_results.is_empty(),
        "PublicEnum should be found in rustdoc JSON"
    );

    insta::assert_snapshot!(format!("{:#?}", (
        public_struct_results,
        public_function_results,
        public_enum_results
    )), @r#"
    (
        [
            "test_visibility::PublicStruct::PublicStruct (Struct(Struct { kind: Plain { fields: [Id(58)], has_stripped_fields: true }, generics: Generics { params: [], where_predicates: [] }, impls: [Id(63), Id(64), Id(65), Id(66), Id(67), Id(68), Id(69), Id(70), Id(71), Id(72), Id(73), Id(74), Id(75), Id(76)] }), vis: Public)",
        ],
        [
            "test_visibility::public_function::public_function (Function(Function { sig: FunctionSignature { inputs: [], output: Some(ResolvedPath(Path { path: \"String\", id: Id(59), args: None })), is_c_variadic: false }, generics: Generics { params: [], where_predicates: [] }, header: FunctionHeader { is_const: false, is_unsafe: false, is_async: false, abi: Rust }, has_body: true }), vis: Public)",
        ],
        [
            "test_visibility::PublicEnum::PublicEnum (Enum(Enum { generics: Generics { params: [], where_predicates: [] }, has_stripped_variants: false, variants: [Id(93), Id(95)], impls: [Id(97), Id(98), Id(99), Id(100), Id(101), Id(102), Id(103), Id(104), Id(105), Id(106), Id(107), Id(108), Id(109)] }), vis: Public)",
        ],
    )
    "#);
}

#[test]
fn test_visibility_crate_visible_items_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let crate_struct_results = find_items_by_name(&krate, "CrateVisibleStruct");
    let crate_function_results = find_items_by_name(&krate, "crate_visible_function");

    insta::assert_snapshot!(format!("{:#?}", (crate_struct_results, crate_function_results)), @r#"
    (
        [],
        [],
    )
    "#);
}

#[test]
fn test_visibility_private_items_not_in_json() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let private_struct_results = find_items_by_name(&krate, "PrivateStruct");
    assert!(
        private_struct_results.is_empty(),
        "PrivateStruct (private) should NOT be in rustdoc JSON"
    );

    let private_function_results = find_items_by_name(&krate, "private_function");
    assert!(
        private_function_results.is_empty(),
        "private_function (private) should NOT be in rustdoc JSON"
    );

    insta::assert_snapshot!(format!("{:#?}", (private_struct_results, private_function_results)), @r#"
    (
        [],
        [],
    )
    "#);
}

#[test]
fn test_visibility_nested_super_visible() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let super_visible_results = find_items_by_name(&krate, "NestedSuperVisible");

    insta::assert_snapshot!(format!("{:#?}", super_visible_results), @"[]");
}

#[test]
fn test_visibility_levels_are_recorded() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let mut visibility_info = vec![];
    for (_id, item) in &krate.index {
        if let Some(name) = &item.name {
            match name.as_str() {
                "PublicStruct" => {
                    assert!(
                        matches!(item.visibility, rustdoc_types::Visibility::Public),
                        "PublicStruct should have Public visibility"
                    );
                    visibility_info.push((name.clone(), format!("{:?}", item.visibility)));
                }
                "CrateVisibleStruct" => {
                    assert!(
                        matches!(item.visibility, rustdoc_types::Visibility::Crate),
                        "CrateVisibleStruct should have Crate visibility"
                    );
                    visibility_info.push((name.clone(), format!("{:?}", item.visibility)));
                }
                _ => {}
            }
        }
    }

    insta::assert_snapshot!(format!("{:#?}", visibility_info), @r#"
    [
        (
            "PublicStruct",
            "Public",
        ),
    ]
    "#);
}

#[test]
fn test_visibility_private_fields_handling() {
    let json_path = generate_rustdoc_json("test-visibility");
    let krate = load_rustdoc_json(&json_path);

    let mut struct_info = None;
    for (_id, item) in &krate.index {
        if let Some(name) = &item.name {
            if name == "PublicStruct" {
                if let rustdoc_types::ItemEnum::Struct(s) = &item.inner {
                    if let rustdoc_types::StructKind::Plain { fields, .. } = &s.kind {
                        assert!(
                            !fields.is_empty(),
                            "PublicStruct should have visible fields"
                        );
                        struct_info = Some(fields.clone());
                    }
                }
            }
        }
    }

    insta::assert_snapshot!(format!("{:#?}", struct_info), @r#"
    Some(
        [
            Id(
                58,
            ),
        ],
    )
    "#);
}
