// No longer need to filter cargo noise since we're not running cargo

fn sort_search_results(stdout: &str) -> String {
    if !stdout.contains("Multiple items found") {
        return stdout.to_string();
    }

    let lines: Vec<&str> = stdout.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_results = false;
    let mut result_lines: Vec<String> = Vec::new();
    let mut first_item = None;

    for line in lines {
        if line.contains("────────────────────────────────────────")
        {
            if in_results {
                // End of results section - sort and add them
                result_lines.sort();
                // Capture the first item for the example line
                if let Some(first) = result_lines.first() {
                    let trimmed = first.trim();
                    if let Some(item_name) = trimmed.split_whitespace().last() {
                        first_item = Some(item_name.to_string());
                    }
                }
                result.extend(result_lines.drain(..));
                result.push(line.to_string());
                in_results = false;
            } else {
                // Start of results section
                result.push(line.to_string());
                in_results = true;
            }
        } else if in_results {
            result_lines.push(line.to_string());
        } else if line.trim().starts_with("Example:") {
            // Replace the example line with the first sorted item
            if let Some(item) = &first_item {
                // Example line format: "Example: docsrs <crate> <item>"
                // Keep everything except the last item
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let prefix: Vec<&str> = parts[..parts.len() - 1].to_vec();
                    result.push(format!("{} {}", prefix.join(" "), item));
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

fn run_cli(args: &[&str]) -> (String, String, bool) {
    // Call the library function directly
    match mx_docsrs::run_cli(args) {
        Ok(stdout) => {
            let stdout = sort_search_results(&stdout);
            (stdout, String::new(), true)
        }
        Err(stderr) => (String::new(), stderr, false),
    }
}

#[test]
fn test_cli_with_explicit_version() {
    let (stdout, stderr, success) = run_cli(&["tokio@latest", "spawn"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("Multiple items found") || stdout.contains("fn:"),
        "Should show results"
    );

    insta::assert_snapshot!(stdout, @r"
    Multiple items found. Use the fully qualified name to view a specific item:
    ────────────────────────────────────────────────────────────────────────────────
      fn           tokio::on_task_spawn
      fn           tokio::spawn
      fn           tokio::spawn_blocking
      fn           tokio::spawn_blocking_on
      fn           tokio::spawn_local
      fn           tokio::spawn_local_on
      fn           tokio::spawn_on
      fn           tokio::spawn_with
      fn           tokio::spawned_at
      fn           tokio::spawned_tasks_count
      fn           tokio::task::blocking::spawn_blocking
      fn           tokio::task::local::spawn_local
      fn           tokio::task::spawn::spawn
    ────────────────────────────────────────────────────────────────────────────────

    Example: docsrs tokio tokio::on_task_spawn
    ");
    insta::assert_snapshot!(stderr, @"");
}

#[test]
fn test_cli_with_crate_in_dependencies() {
    let (stdout, stderr, success) = run_cli(&["anyhow", "Error"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("Multiple items found")
            || stdout.contains("struct:")
            || stdout.contains("Error"),
        "Should show results for Error"
    );

    insta::assert_snapshot!(stdout, @r"
    Multiple items found. Use the fully qualified name to view a specific item:
    ────────────────────────────────────────────────────────────────────────────────
      associatedtype anyhow::Error
      fn           anyhow::into_boxed_dyn_error
      fn           anyhow::reallocate_into_boxed_dyn_error_without_backtrace
      trait        anyhow::context::ext::StdError
    ────────────────────────────────────────────────────────────────────────────────

    Example: docsrs tokio anyhow::Error
    ");
    insta::assert_snapshot!(stderr, @"");
}

#[test]
fn test_cli_with_unknown_crate() {
    let (stdout, stderr, success) = run_cli(&["some_unknown_crate", "symbol"]);
    assert!(!success, "CLI should fail for unknown crate");
    assert!(
        stderr.contains("Failed to fetch") || stderr.contains("404") || stderr.contains("error"),
        "Should show error for unknown crate"
    );

    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @"http status: 404");
}

#[test]
fn test_cli_missing_arguments() {
    let (stdout, stderr, success) = run_cli(&["clap"]);
    assert!(!success, "CLI should fail with missing arguments");
    let output = format!("{}{}", stdout, stderr);
    assert!(
        output.contains("required arguments were not provided") || output.contains("SYMBOL"),
        "Should show error about missing symbol argument"
    );

    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @"Missing required argument: SYMBOL");
}

#[test]
fn test_cli_empty_crate_name() {
    let (stdout, stderr, success) = run_cli(&["", "symbol"]);
    assert!(!success, "CLI should fail with empty crate name");
    let output = format!("{}{}", stdout, stderr);
    assert!(
        output.contains("empty"),
        "Should show error about empty name"
    );

    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r"
    error: invalid value '' for '[CRATE_SPEC]': Crate name cannot be empty

    For more information, try '--help'.
    ");
}

#[test]
fn test_cli_empty_version() {
    let (stdout, stderr, success) = run_cli(&["crate@", "symbol"]);
    assert!(!success, "CLI should fail with empty version");
    let output = format!("{}{}", stdout, stderr);
    assert!(
        output.contains("empty"),
        "Should show error about empty version"
    );

    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r"
    error: invalid value 'crate@' for '[CRATE_SPEC]': Version cannot be empty after '@'

    For more information, try '--help'.
    ");
}

#[test]
fn test_cli_help() {
    let (stdout, stderr, success) = run_cli(&["--help"]);
    let output = format!("{}{}", stdout, stderr);
    assert!(success, "Help should succeed");
    assert!(output.contains("docsrs"), "Should mention binary name");
    assert!(
        output.contains("CRATE_SPEC"),
        "Should mention crate spec argument"
    );
    assert!(output.contains("SYMBOL"), "Should mention symbol argument");
    assert!(
        output.contains("@version") || output.contains("optionally"),
        "Should mention optional version syntax"
    );

    insta::assert_snapshot!(stdout, @r#"
    Search for documentation of a symbol in a crate

    Usage: docsrs [OPTIONS] [CRATE_SPEC] [SYMBOL]

    Arguments:
      [CRATE_SPEC]  The crate name to search in, optionally with version (e.g., "serde" or "serde@1.0")
      [SYMBOL]      The symbol to search for

    Options:
          --no-cache     Skip cache and download fresh rustdoc JSON
          --clear-cache  Clear the entire cache directory
      -h, --help         Print help
    "#);
    insta::assert_snapshot!(stderr, @"");
}

#[test]
fn test_cli_resolves_cargo_metadata_dependency() {
    let (stdout, stderr, success) = run_cli(&["cargo_metadata", "MetadataCommand"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("struct:") || stdout.contains("MetadataCommand"),
        "Should show MetadataCommand"
    );

    insta::assert_snapshot!(stdout, @r"
    ────────────────────────────────────────────────────────────
    struct: MetadataCommand
    ────────────────────────────────────────────────────────────
    pub struct MetadataCommand { ... }

    A builder for configuring cargo metadata invocation.


    ────────────────────────────────────────────────────────────
    ");
    insta::assert_snapshot!(stderr, @"");
}

#[test]
fn test_cli_resolves_anyhow_dependency() {
    let (stdout, stderr, success) = run_cli(&["anyhow", "Error"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("struct:")
            || stdout.contains("Error")
            || stdout.contains("Multiple items found"),
        "Should show Error results"
    );

    insta::assert_snapshot!(stdout, @r"
    Multiple items found. Use the fully qualified name to view a specific item:
    ────────────────────────────────────────────────────────────────────────────────
      associatedtype anyhow::Error
      fn           anyhow::into_boxed_dyn_error
      fn           anyhow::reallocate_into_boxed_dyn_error_without_backtrace
      trait        anyhow::context::ext::StdError
    ────────────────────────────────────────────────────────────────────────────────

    Example: docsrs tokio anyhow::Error
    ");
    insta::assert_snapshot!(stderr, @"");
}

#[test]
fn test_cli_complex_version_requirement() {
    let (stdout, stderr, success) = run_cli(&["serde@latest", "Serialize"]);
    assert!(success, "CLI should succeed");
    assert!(
        stdout.contains("trait:")
            || stdout.contains("Serialize")
            || stdout.contains("Multiple items found"),
        "Should show Serialize results"
    );

    insta::assert_snapshot!(stdout, @r"
    Multiple items found. Use the fully qualified name to view a specific item:
    ────────────────────────────────────────────────────────────────────────────────
      associatedtype serde::Deserializer
      associatedtype serde::SerializeMap
      associatedtype serde::SerializeSeq
      associatedtype serde::SerializeStruct
      associatedtype serde::SerializeStructVariant
      associatedtype serde::SerializeTuple
      associatedtype serde::SerializeTupleStruct
      associatedtype serde::SerializeTupleVariant
      fn           serde::deserialize
      fn           serde::deserialize_any
      fn           serde::deserialize_bool
      fn           serde::deserialize_byte_buf
      fn           serde::deserialize_bytes
      fn           serde::deserialize_char
      fn           serde::deserialize_enum
      fn           serde::deserialize_f32
      fn           serde::deserialize_f64
      fn           serde::deserialize_i128
      fn           serde::deserialize_i16
      fn           serde::deserialize_i32
      fn           serde::deserialize_i64
      fn           serde::deserialize_i8
      fn           serde::deserialize_identifier
      fn           serde::deserialize_ignored_any
      fn           serde::deserialize_map
      fn           serde::deserialize_newtype_struct
      fn           serde::deserialize_option
      fn           serde::deserialize_seq
      fn           serde::deserialize_str
      fn           serde::deserialize_string
      fn           serde::deserialize_struct
      fn           serde::deserialize_tuple
      fn           serde::deserialize_tuple_struct
      fn           serde::deserialize_u128
      fn           serde::deserialize_u16
      fn           serde::deserialize_u32
      fn           serde::deserialize_u64
      fn           serde::deserialize_u8
      fn           serde::deserialize_unit
      fn           serde::deserialize_unit_struct
      fn           serde::into_deserializer
      fn           serde::serialize
      fn           serde::serialize_bool
      fn           serde::serialize_bytes
      fn           serde::serialize_char
      fn           serde::serialize_element
      fn           serde::serialize_entry
      fn           serde::serialize_f32
      fn           serde::serialize_f64
      fn           serde::serialize_field
      fn           serde::serialize_i128
      fn           serde::serialize_i16
      fn           serde::serialize_i32
      fn           serde::serialize_i64
      fn           serde::serialize_i8
      fn           serde::serialize_key
      fn           serde::serialize_map
      fn           serde::serialize_newtype_struct
      fn           serde::serialize_newtype_variant
      fn           serde::serialize_none
      fn           serde::serialize_seq
      fn           serde::serialize_some
      fn           serde::serialize_str
      fn           serde::serialize_struct
      fn           serde::serialize_struct_variant
      fn           serde::serialize_tuple
      fn           serde::serialize_tuple_struct
      fn           serde::serialize_tuple_variant
      fn           serde::serialize_u128
      fn           serde::serialize_u16
      fn           serde::serialize_u32
      fn           serde::serialize_u64
      fn           serde::serialize_u8
      fn           serde::serialize_unit
      fn           serde::serialize_unit_struct
      fn           serde::serialize_unit_variant
      fn           serde::serialize_value
      macro        serde::forward_to_deserialize_any
      struct       serde::de::value::BoolDeserializer
      struct       serde::de::value::BorrowedBytesDeserializer
      struct       serde::de::value::BorrowedStrDeserializer
      struct       serde::de::value::BytesDeserializer
      struct       serde::de::value::CharDeserializer
      struct       serde::de::value::CowStrDeserializer
      struct       serde::de::value::EnumAccessDeserializer
      struct       serde::de::value::F32Deserializer
      struct       serde::de::value::F64Deserializer
      struct       serde::de::value::I128Deserializer
      struct       serde::de::value::I16Deserializer
      struct       serde::de::value::I32Deserializer
      struct       serde::de::value::I64Deserializer
      struct       serde::de::value::I8Deserializer
      struct       serde::de::value::IsizeDeserializer
      struct       serde::de::value::MapAccessDeserializer
      struct       serde::de::value::MapDeserializer
      struct       serde::de::value::NeverDeserializer
      struct       serde::de::value::SeqAccessDeserializer
      struct       serde::de::value::SeqDeserializer
      struct       serde::de::value::StrDeserializer
      struct       serde::de::value::StringDeserializer
      struct       serde::de::value::U128Deserializer
      struct       serde::de::value::U16Deserializer
      struct       serde::de::value::U32Deserializer
      struct       serde::de::value::U64Deserializer
      struct       serde::de::value::U8Deserializer
      struct       serde::de::value::UnitDeserializer
      struct       serde::de::value::UsizeDeserializer
      trait        serde::de::Deserialize
      trait        serde::de::DeserializeOwned
      trait        serde::de::DeserializeSeed
      trait        serde::de::Deserializer
      trait        serde::de::IntoDeserializer
      trait        serde::ser::Serialize
      trait        serde::ser::SerializeMap
      trait        serde::ser::SerializeSeq
      trait        serde::ser::SerializeStruct
      trait        serde::ser::SerializeStructVariant
      trait        serde::ser::SerializeTuple
      trait        serde::ser::SerializeTupleStruct
      trait        serde::ser::SerializeTupleVariant
      trait        serde::ser::Serializer
    ────────────────────────────────────────────────────────────────────────────────

    Example: docsrs tokio serde::Deserializer
    ");
    insta::assert_snapshot!(stderr, @"");
}
