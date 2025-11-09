// Parser for rustdoc search-index.js format

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};
use std::collections::HashMap;

/// Item type ID from rustdoc search index.
///
/// Represents the different kinds of Rust items that can appear in documentation.
/// The numeric values correspond to rustdoc's internal type encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum ItemType {
    MutRef = 0,
    PrimitiveOrBuiltin = 1,
    Module = 2,
    ExternCrate = 3,
    Import = 4,
    Struct = 5,
    Enum = 6,
    Function = 7,
    Typedef = 8,
    Static = 9,
    Trait = 10,
    Impl = 11,
    TyMethod = 12,
    Method = 13,
    StructField = 14,
    Variant = 15,
    Macro = 16,
    Primitive = 17,
    AssocConst = 18,
    AssocType = 19,
    Constant = 20,
    Union = 21,
    ForeignType = 22,
    Keyword = 23,
    OpaqueTy = 24,
    ProcAttribute = 25,
    ProcDerive = 26,
    TraitAlias = 27,
}

/// A crate entry from the search index
#[derive(Debug, Deserialize, Serialize)]
struct CrateEntry {
    /// Name of the crate
    #[serde(rename = "0")]
    name: String,
    /// Compact data for this crate
    #[serde(rename = "1")]
    data: CrateData,
}

/// Qualified path entry - maps an item index to its module path.
#[derive(Debug, Deserialize, Serialize)]
struct QualifiedPath {
    /// Item index this path applies to
    #[serde(rename = "0")]
    index: usize,

    /// Fully qualified module path
    #[serde(rename = "1")]
    path: String,
}

/// Parent item type information.
#[derive(Debug, Deserialize, Serialize)]
struct PathItem {
    /// Item type
    #[serde(rename = "0")]
    ty: ItemType,

    /// Item name
    #[serde(rename = "1")]
    name: String,

    /// Index into the `paths` array for module path
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    path_index: Option<usize>,

    /// Index into the `paths` array for exact path (re-exports)
    #[serde(rename = "3", skip_serializing_if = "Option::is_none", default)]
    exact_path_index: Option<usize>,

    /// Unbox flag for special handling
    #[serde(rename = "4", skip_serializing_if = "Option::is_none", default)]
    unbox_flag: Option<u32>,
}

/// Re-export entry.
#[derive(Debug, Deserialize, Serialize)]
struct Reexport {
    /// Item index
    #[serde(rename = "0")]
    item_index: usize,

    /// Index into the `paths` array for re-export location
    #[serde(rename = "1")]
    path_index: usize,
}

/// Parameter types for a function or method.
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
struct ParamTypes {
    /// Item index
    #[serde(rename = "0")]
    item_index: usize,

    /// Type parameters (parsed from comma-separated string)
    #[serde(rename = "1")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    types: Vec<String>,
}

/// Implementation disambiguator for trait implementations.
#[derive(Debug, Deserialize, Serialize)]
struct ImplDisambiguator {
    /// Item index
    #[serde(rename = "0")]
    item_index: usize,

    /// URL-encoded disambiguator string
    #[serde(rename = "1")]
    disambiguator: String,
}

/// Compact crate data from search-index.js.
///
/// This represents the compressed/encoded search data for a single crate.
/// The core structure uses parallel arrays: `types` and `names` are the same length,
/// where position `i` represents one searchable item.
///
/// For each item at position `i`:
/// - `types[i]` (character) encodes the item's type (struct, fn, trait, etc.)
/// - `names[i]` (string) is the item's name
/// - Additional fields (`paths`, `parent_items`, `reexports`, etc.) provide optional metadata via sparse maps
///
/// Based on the format documented in SEARCH_INDEX_FORMAT.md
#[derive(Debug, Deserialize, Serialize)]
struct CrateData {
    /// Type string where each character encodes a type ID for the corresponding item.
    ///
    /// Each character maps to a type via: `char.to_digit(36) - 10` or similar encoding.
    /// Common types: 'K'=10 (trait), 'N'=13 (method), 'C'=2 (module), etc.
    /// Length always equals `names.length` (parallel arrays).
    #[serde(rename = "t")]
    types: String,

    /// Names array containing the name of each searchable item.
    ///
    /// Parallel to the `types` array - position `i` in this array corresponds to position `i` in `types`.
    /// Empty string "" means "reuse the last name" (compression technique).
    /// Examples: ["SliceExt", "alloc", "boxed", ...]
    #[serde(rename = "n")]
    names: Vec<String>,

    /// Qualified paths array - sparse map of item indices to their module paths.
    ///
    /// Not all items have entries here. When processing item at index `i`, if there's
    /// a QualifiedPath with that index, use its path. Otherwise, reuse the last path. (compression technique).
    ///
    /// For example, if `q` contains `{index: 142, path: "either::iterator"}`,
    /// then the item at position 142 in the `n` array belongs to the module path
    /// "either::iterator".
    #[serde(rename = "q", default)]
    paths: Vec<QualifiedPath>,

    /// Path/parent data array - type information for items that can be parents.
    ///
    /// This array is used to build the `paths` array for parent lookups. Items in the
    /// main arrays can reference entries here via the `parent_indices` field to
    /// indicate their parent type (e.g., a method's parent struct/trait).
    #[serde(rename = "p", default)]
    parent_items: Vec<PathItem>,

    /// Re-exports array - maps items to their re-export locations.
    ///
    /// Sparse array tracking which items are re-exported and where. Each entry maps
    /// an item index to a path index in the `paths` array, indicating the module path
    /// where the item is re-exported.
    #[serde(rename = "r", default)]
    reexports: Vec<Reexport>,
    /// Parent indices (VLQ hex encoded)
    #[serde(default)]
    i: String,
    /// Function type signatures (VLQ hex encoded)
    #[serde(default)]
    f: String,
    /// Description shard lengths (VLQ hex encoded)
    #[serde(default, rename = "D")]
    desc: String,

    /// Parameter types array - maps item indices to their parameter types.
    ///
    /// Sparse array containing type parameter information for functions and methods.
    /// Each entry maps an item index to a vector of type parameters (generics, associated types, etc.).
    #[serde(default, rename = "P")]
    param_types: Vec<ParamTypes>,

    /// Implementation disambiguators - uniquely identify trait implementations.
    ///
    /// Sparse array mapping item indices to URL-encoded disambiguator strings.
    /// Used to distinguish between multiple trait implementations for the same type.
    #[serde(default, rename = "b")]
    impl_disambiguators: Vec<ImplDisambiguator>,
    /// Deprecated items bitmap
    #[serde(default)]
    c: String,
    /// Empty description bitmap
    #[serde(default)]
    e: String,

    /// Aliases - maps alternative names to item indices.
    ///
    /// Optional field containing a map from alias names to arrays of item indices.
    /// This allows items to be found by multiple names during search.
    /// For example, "errno" and "__errno_location" might both map to the same item.
    #[serde(default, rename = "a")]
    aliases: Option<HashMap<String, Vec<usize>>>,
}

/// Extract the JSON string from search-index.js
/// The file format is: var searchIndex = new Map(JSON.parse('[...]'));
fn extract_json_string(content: &str) -> String {
    // Find the pattern JSON.parse(' and ')
    let start_pattern = "JSON.parse('";
    let end_pattern = "')";

    let start = content
        .find(start_pattern)
        .expect("Could not find JSON.parse('")
        + start_pattern.len();

    let end = content[start..]
        .find(end_pattern)
        .expect("Could not find closing ')")
        + start;

    let json_str = &content[start..end];

    // Unescape \' to '
    json_str.replace(r"\'", "'")
}

/// Parse the JSON string into a vector of crate entries
/// The format is an array of [crate_name, crate_data] pairs
fn parse_search_index(json_string: &str) -> Vec<CrateEntry> {
    // Parse directly as a JSON array of CrateEntry structs
    serde_json::from_str(json_string).expect("Failed to parse JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_string() {
        let content = std::fs::read_to_string("tests/fixtures/search-index.js")
            .expect("Failed to read fixture");

        let json_string = extract_json_string(&content);

        // Should extract a non-empty string
        assert!(
            !json_string.is_empty(),
            "Extracted JSON string should not be empty"
        );

        // Should start with [[ (array of arrays)
        assert!(json_string.starts_with("[["), "JSON should start with [[");

        // Should end with ]]
        assert!(json_string.ends_with("]]"), "JSON should end with ]]");
    }

    #[test]
    fn test_extract_json_string_unescapes_quotes() {
        // Test with a simple example containing escaped quotes
        let content =
            r#"var searchIndex = new Map(JSON.parse('[["test",{"desc":"It\'s a test"}]]'));"#;

        let json_string = extract_json_string(&content);

        // Should not contain \' - should be unescaped to just '
        assert!(
            !json_string.contains(r"\'"),
            "Should not contain escaped quotes"
        );
        assert!(
            json_string.contains("It's"),
            "Should contain unescaped quote"
        );
    }

    #[test]
    fn test_parse_search_index() {
        let content = std::fs::read_to_string("tests/fixtures/search-index.js")
            .expect("Failed to read fixture");

        let json_string = extract_json_string(&content);
        let crates = parse_search_index(&json_string);

        // Should have parsed multiple crates
        assert!(!crates.is_empty(), "Should have parsed at least one crate");

        // Each crate should have a name and data
        for crate_entry in &crates {
            assert!(
                !crate_entry.name.is_empty(),
                "Crate name should not be empty"
            );
            // types and names should be parallel arrays
            assert_eq!(
                crate_entry.data.types.len(),
                crate_entry.data.names.len(),
                "types and names arrays should have same length for crate {}",
                crate_entry.name
            );
            // Should have at least one item
            assert!(
                !crate_entry.data.types.is_empty(),
                "Crate {} should have items",
                crate_entry.name
            );
        }
    }

    #[test]
    fn explore_aliases_field() {
        let content = std::fs::read_to_string("tests/fixtures/search-index.js")
            .expect("Failed to read fixture");

        let json_string = extract_json_string(&content);
        let crates = parse_search_index(&json_string);

        // Collect all aliases from all crates that have them
        let all_aliases: Vec<_> = crates
            .iter()
            .filter_map(|crate_entry| {
                crate_entry
                    .data
                    .aliases
                    .as_ref()
                    .map(|aliases| (crate_entry.name.clone(), aliases))
            })
            .collect();

        println!("\n=== Total crates with aliases: {} ===", all_aliases.len());

        // Show all entries
        for (i, (crate_name, aliases)) in all_aliases.iter().enumerate() {
            println!(
                "\n[{}] Crate: {} ({} aliases)",
                i,
                crate_name,
                aliases.len()
            );
            for (alias, item_indices) in aliases.iter().take(5) {
                println!("  \"{}\" -> {:?}", alias, item_indices);
            }
            if aliases.len() > 5 {
                println!("  ... and {} more", aliases.len() - 5);
            }
        }
    }

    #[test]
    fn test_malformed_p_field_errors() {
        // Test with wrong type for ty field (string instead of number)
        let malformed_json =
            r#"[["test", {"t":"A", "n":["foo"], "p":[[{"wrong": "type"}, "name"]]}]]"#;
        let result: Result<Vec<CrateEntry>, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err(), "Should error on malformed ty field");

        // Test with wrong type for path_index (string instead of number)
        let malformed_json2 = r#"[["test", {"t":"A", "n":["foo"], "p":[[5, "name", "wrong"]]}]]"#;
        let result2: Result<Vec<CrateEntry>, _> = serde_json::from_str(malformed_json2);
        assert!(
            result2.is_err(),
            "Should error on malformed path_index field"
        );

        // Test with valid data should succeed
        let valid_json = r#"[["test", {"t":"A", "n":["foo"], "p":[[5, "name", 10, 20]]}]]"#;
        let result3: Result<Vec<CrateEntry>, _> = serde_json::from_str(valid_json);
        assert!(result3.is_ok(), "Should succeed with valid data");
    }
}
