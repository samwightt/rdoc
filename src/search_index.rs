// Parser for rustdoc search-index.js format

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

/// Compact crate data from search-index.js
/// Based on the format documented in SEARCH_INDEX_FORMAT.md
#[derive(Debug, Deserialize, Serialize)]
struct CrateData {
    /// Type string - each character encodes a type ID
    t: String,
    /// Names array - parallel to t array
    n: Vec<String>,
    /// Qualified paths array
    #[serde(default)]
    q: Vec<Value>,
    /// Path/parent data array
    #[serde(default)]
    p: Vec<Value>,
    /// Re-exports map
    #[serde(default)]
    r: Vec<Value>,
    /// Parent indices (VLQ hex encoded)
    #[serde(default)]
    i: String,
    /// Function type signatures (VLQ hex encoded)
    #[serde(default)]
    f: String,
    /// Description shard lengths (VLQ hex encoded)
    #[serde(default, rename = "D")]
    desc: String,
    /// Parameter names
    #[serde(default, rename = "P")]
    param_names: Vec<Value>,
    /// Impl disambiguators
    #[serde(default)]
    b: Vec<Value>,
    /// Deprecated items bitmap
    #[serde(default)]
    c: String,
    /// Empty description bitmap
    #[serde(default)]
    e: String,
    /// Aliases (optional)
    #[serde(default)]
    a: Option<Value>,
}

/// Extract the JSON string from search-index.js
/// The file format is: var searchIndex = new Map(JSON.parse('[...]'));
fn extract_json_string(content: &str) -> String {
    // Find the pattern JSON.parse(' and ')
    let start_pattern = "JSON.parse('";
    let end_pattern = "')";

    let start = content.find(start_pattern)
        .expect("Could not find JSON.parse('") + start_pattern.len();

    let end = content[start..].find(end_pattern)
        .expect("Could not find closing ')") + start;

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
        assert!(!json_string.is_empty(), "Extracted JSON string should not be empty");

        // Should start with [[ (array of arrays)
        assert!(json_string.starts_with("[["), "JSON should start with [[");

        // Should end with ]]
        assert!(json_string.ends_with("]]"), "JSON should end with ]]");
    }

    #[test]
    fn test_extract_json_string_unescapes_quotes() {
        // Test with a simple example containing escaped quotes
        let content = r#"var searchIndex = new Map(JSON.parse('[["test",{"desc":"It\'s a test"}]]'));"#;

        let json_string = extract_json_string(&content);

        // Should not contain \' - should be unescaped to just '
        assert!(!json_string.contains(r"\'"), "Should not contain escaped quotes");
        assert!(json_string.contains("It's"), "Should contain unescaped quote");
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
            assert!(!crate_entry.name.is_empty(), "Crate name should not be empty");
            // t and n should be parallel arrays
            assert_eq!(crate_entry.data.t.len(), crate_entry.data.n.len(),
                "t and n arrays should have same length for crate {}", crate_entry.name);
            // Should have at least one item
            assert!(!crate_entry.data.t.is_empty(), "Crate {} should have items", crate_entry.name);
        }
    }
}
