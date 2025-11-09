// Decoded search index items

use crate::search_index::{CrateData, ItemType};
use crate::vlq::VlqHexDecoder;

/// A fully decoded search index item with all metadata resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    /// The crate this item belongs to
    pub crate_name: String,

    /// The item type (struct, function, trait, etc.)
    pub item_type: ItemType,

    /// The item's name
    pub name: String,

    /// Lowercase version for case-insensitive search
    pub normalized_name: String,

    /// The module path
    pub path: String,

    /// Exact path for re-exports (may differ from path)
    pub exact_path: String,

    /// Unique ID for this item
    pub id: usize,

    /// Type parameters for functions/methods
    pub param_types: Vec<String>,

    /// Implementation disambiguator (for trait impls)
    pub impl_disambiguator: Option<String>,

    /// Bit index for deprecated/description bitmaps
    pub bit_index: usize,

    /// Index into the parent_items array (0-based), if this item has a parent
    pub parent_index: Option<usize>,
}

/// Decode a crate's compact data into a vector of search items.
pub fn decode_crate(crate_name: &str, crate_data: &CrateData) -> Vec<SearchItem> {
    let mut items = Vec::new();
    let mut last_name = String::new();
    let mut last_path = String::new();

    // Build lookup maps for sparse arrays
    let paths_map: std::collections::HashMap<usize, &str> = crate_data
        .paths
        .iter()
        .map(|qp| (qp.index, qp.path.as_str()))
        .collect();

    // Create VLQ decoder for parent indices
    let mut parent_decoder = VlqHexDecoder::new(&crate_data.i);

    let reexports_map: std::collections::HashMap<usize, usize> = crate_data
        .reexports
        .iter()
        .map(|r| (r.item_index, r.path_index))
        .collect();

    let param_types_map: std::collections::HashMap<usize, &[String]> = crate_data
        .param_types
        .iter()
        .map(|pt| (pt.item_index, pt.types.as_slice()))
        .collect();

    let impl_disamb_map: std::collections::HashMap<usize, &str> = crate_data
        .impl_disambiguators
        .iter()
        .map(|id| (id.item_index, id.disambiguator.as_str()))
        .collect();

    // Iterate through all items (parallel arrays types and names)
    for i in 0..crate_data.types.len() {
        let bit_index = i + 1;

        // Decode type from types string: char - 'A' (65)
        let type_char = crate_data.types.as_bytes()[i];
        let type_id = type_char - b'A';
        let item_type = decode_item_type(type_id);

        // Get name with compression: empty string means "reuse last name"
        let name = if crate_data.names[i].is_empty() {
            last_name.clone()
        } else {
            crate_data.names[i].clone()
        };

        // Create normalized name: lowercase and remove underscores
        let normalized_name = name.to_lowercase().replace('_', "");

        // Get path with compression: if not in paths_map, reuse last path
        let path = paths_map
            .get(&i)
            .map(|s| s.to_string())
            .unwrap_or_else(|| last_path.clone());

        // Get exact_path: check reexports, otherwise use path
        let exact_path = if let Some(&path_index) = reexports_map.get(&i) {
            paths_map
                .get(&path_index)
                .map(|s| s.to_string())
                .unwrap_or_else(|| path.clone())
        } else {
            path.clone()
        };

        // Get param_types from sparse array
        let param_types = param_types_map
            .get(&i)
            .map(|types| types.to_vec())
            .unwrap_or_default();

        // Get impl_disambiguator from sparse array
        let impl_disambiguator = impl_disamb_map.get(&i).map(|s| s.to_string());

        // Decode parent index (1-based, 0 means no parent)
        let parent_index = parent_decoder.next().and_then(|parent_idx| {
            if parent_idx > 0 {
                Some((parent_idx - 1) as usize)
            } else {
                None
            }
        });

        items.push(SearchItem {
            crate_name: crate_name.to_string(),
            item_type,
            name: name.clone(),
            normalized_name,
            path: path.clone(),
            exact_path,
            id: i,
            param_types,
            impl_disambiguator,
            bit_index,
            parent_index,
        });

        // Update "last" values for next iteration
        last_name = name;
        last_path = path;
    }

    items
}

/// Decode a type ID to ItemType
fn decode_item_type(type_id: u8) -> ItemType {
    match type_id {
        0 => ItemType::MutRef,
        1 => ItemType::PrimitiveOrBuiltin,
        2 => ItemType::Module,
        3 => ItemType::ExternCrate,
        4 => ItemType::Import,
        5 => ItemType::Struct,
        6 => ItemType::Enum,
        7 => ItemType::Function,
        8 => ItemType::Typedef,
        9 => ItemType::Static,
        10 => ItemType::Trait,
        11 => ItemType::Impl,
        12 => ItemType::TyMethod,
        13 => ItemType::Method,
        14 => ItemType::StructField,
        15 => ItemType::Variant,
        16 => ItemType::Macro,
        17 => ItemType::Primitive,
        18 => ItemType::AssocConst,
        19 => ItemType::AssocType,
        20 => ItemType::Constant,
        21 => ItemType::Union,
        22 => ItemType::ForeignType,
        23 => ItemType::Keyword,
        24 => ItemType::OpaqueTy,
        25 => ItemType::ProcAttribute,
        26 => ItemType::ProcDerive,
        27 => ItemType::TraitAlias,
        _ => ItemType::Module, // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_index::CrateData;

    #[test]
    fn test_decode_basic_fields() {
        // Create a simple CrateData with 2 items
        let crate_data = CrateData {
            types: "AB".to_string(), // 2 items
            names: vec!["foo".to_string(), "bar".to_string()],
            paths: vec![],
            parent_items: vec![],
            reexports: vec![],
            i: String::new(),
            f: String::new(),
            desc: String::new(),
            param_types: vec![],
            impl_disambiguators: vec![],
            c: String::new(),
            e: String::new(),
            aliases: None,
        };

        let items = decode_crate("test_crate", &crate_data);

        // Should have 2 items
        assert_eq!(items.len(), 2);

        // Check first item basic fields
        assert_eq!(items[0].crate_name, "test_crate");
        assert_eq!(items[0].id, 0);
        assert_eq!(items[0].bit_index, 1); // bit_index is i + 1

        // Check second item basic fields
        assert_eq!(items[1].crate_name, "test_crate");
        assert_eq!(items[1].id, 1);
        assert_eq!(items[1].bit_index, 2);
    }

    #[test]
    fn test_decode_name_with_compression() {
        // Test name compression: empty string means "reuse last name"
        let crate_data = CrateData {
            types: "ABCD".to_string(), // 4 items
            names: vec![
                "foo".to_string(),
                "".to_string(), // Reuse "foo"
                "bar".to_string(),
                "".to_string(), // Reuse "bar"
            ],
            paths: vec![],
            parent_items: vec![],
            reexports: vec![],
            i: String::new(),
            f: String::new(),
            desc: String::new(),
            param_types: vec![],
            impl_disambiguators: vec![],
            c: String::new(),
            e: String::new(),
            aliases: None,
        };

        let items = decode_crate("test_crate", &crate_data);

        // Check names are decoded correctly with compression
        assert_eq!(items[0].name, "foo");
        assert_eq!(items[1].name, "foo"); // Reused from previous
        assert_eq!(items[2].name, "bar");
        assert_eq!(items[3].name, "bar"); // Reused from previous
    }

    #[test]
    fn test_decode_comprehensive() {
        use crate::search_index::{ImplDisambiguator, ParamTypes, QualifiedPath, Reexport};

        // Create comprehensive test data
        let crate_data = CrateData {
            // 'C'=67-65=2 (Module), 'F'=70-65=5 (Struct), 'K'=75-65=10 (Trait)
            types: "CFK".to_string(),
            names: vec![
                "foo".to_string(),
                "Bar".to_string(),
                "Baz_Trait".to_string(),
            ],
            paths: vec![
                QualifiedPath {
                    index: 0,
                    path: "mylib".to_string(),
                },
                QualifiedPath {
                    index: 1,
                    path: "mylib::structs".to_string(),
                },
            ],
            parent_items: vec![],
            reexports: vec![
                Reexport {
                    item_index: 1,
                    path_index: 0,
                }, // Bar is reexported at "mylib"
            ],
            i: String::new(),
            f: String::new(),
            desc: String::new(),
            param_types: vec![ParamTypes {
                item_index: 2,
                types: vec!["T".to_string(), "U".to_string()],
            }],
            impl_disambiguators: vec![ImplDisambiguator {
                item_index: 1,
                disambiguator: "impl-Debug-for-Bar".to_string(),
            }],
            c: String::new(),
            e: String::new(),
            aliases: None,
        };

        let items = decode_crate("mylib", &crate_data);

        // Check item types
        assert_eq!(items[0].item_type, ItemType::Module);
        assert_eq!(items[1].item_type, ItemType::Struct);
        assert_eq!(items[2].item_type, ItemType::Trait);

        // Check normalized names
        assert_eq!(items[0].normalized_name, "foo");
        assert_eq!(items[1].normalized_name, "bar"); // lowercase
        assert_eq!(items[2].normalized_name, "baztrait"); // lowercase + no underscores

        // Check paths (with compression)
        assert_eq!(items[0].path, "mylib");
        assert_eq!(items[1].path, "mylib::structs");
        assert_eq!(items[2].path, "mylib::structs"); // Reused from previous

        // Check exact_path (reexports)
        assert_eq!(items[0].exact_path, "mylib");
        assert_eq!(items[1].exact_path, "mylib"); // Reexported, different from path
        assert_eq!(items[2].exact_path, "mylib::structs"); // Not reexported, same as path

        // Check param_types
        assert_eq!(items[0].param_types, Vec::<String>::new());
        assert_eq!(items[1].param_types, Vec::<String>::new());
        assert_eq!(items[2].param_types, vec!["T", "U"]);

        // Check impl_disambiguator
        assert_eq!(items[0].impl_disambiguator, None);
        assert_eq!(
            items[1].impl_disambiguator,
            Some("impl-Debug-for-Bar".to_string())
        );
        assert_eq!(items[2].impl_disambiguator, None);
    }

    #[test]
    fn test_decode_parent_info() {
        use crate::search_index::{PathItem, QualifiedPath};

        // Create test data with parent items and parent indices
        // We'll have:
        // - parent_items[0]: Module named "foo" at path "mylib"
        // - parent_items[1]: Struct named "Bar" at path "mylib::structs"
        // - item 0: no parent (parent_idx = 0)
        // - item 1: parent is parent_items[0] (parent_idx = 1)
        // - item 2: parent is parent_items[1] (parent_idx = 2)

        let crate_data = CrateData {
            types: "ABC".to_string(), // 3 items
            names: vec![
                "top".to_string(),
                "child1".to_string(),
                "child2".to_string(),
            ],
            paths: vec![
                QualifiedPath {
                    index: 0,
                    path: "mylib".to_string(),
                },
                QualifiedPath {
                    index: 1,
                    path: "mylib::structs".to_string(),
                },
            ],
            parent_items: vec![
                PathItem {
                    ty: ItemType::Module,
                    name: "foo".to_string(),
                    path_index: Some(0), // points to "mylib"
                    exact_path_index: None,
                    unbox_flag: None,
                },
                PathItem {
                    ty: ItemType::Struct,
                    name: "Bar".to_string(),
                    path_index: Some(1), // points to "mylib::structs"
                    exact_path_index: None,
                    unbox_flag: None,
                },
            ],
            reexports: vec![],
            // VLQ encoded parent indices: 0, 1, 2
            // To encode value V: encoded = (V << 1) | 0 (for positive)
            // 0: (0 << 1) | 0 = 0, as single char: 0&15=0, need terminal byte >= 96, so '`' (96)
            // But 'a' (97) gives: 97&15=1, 1&1=1 (neg), 1>>1=0, result=0 ✓
            // 1: (1 << 1) | 0 = 2, as single char: need &15=2, so 'b' (98)
            // 2: (2 << 1) | 0 = 4, as single char: need &15=4, so 'd' (100)
            i: "abd".to_string(), // 0, 1, 2
            f: String::new(),
            desc: String::new(),
            param_types: vec![],
            impl_disambiguators: vec![],
            c: String::new(),
            e: String::new(),
            aliases: None,
        };

        let items = decode_crate("mylib", &crate_data);

        // Item 0 should have no parent
        assert_eq!(items[0].parent_index, None);

        // Item 1 should have parent_items[0] as parent (index 0)
        assert_eq!(items[1].parent_index, Some(0));

        // Item 2 should have parent_items[1] as parent (index 1)
        assert_eq!(items[2].parent_index, Some(1));
    }
}
