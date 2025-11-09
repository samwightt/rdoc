# Rustdoc Search Index Format

## Overview

The search index has two parts:
1. **Raw compact data** in `search-index.js` - compressed/encoded format
2. **Expanded data** after `initSearch()` processes it - full array of items

## Raw Format (search-index.js)

```javascript
var searchIndex = new Map(JSON.parse('[
  ["crate_name", {compact_data}],
  ...
]'));
```

### Compact Data Fields

Each crate has compact data with these fields. The `t` and `n` arrays are parallel - each character at position `i` in `t` corresponds to the item at position `i` in `n`.

**Core fields:**
- `t` - Type string (single character per item, e.g., "KCCCMNNMQQCQFKFK...")
  - Each character encodes a type ID via: `charCodeAt(i) - 65`
  - Examples: 'A'=0, 'B'=1, 'C'=2, 'K'=10 (trait), 'N'=13, etc.
  - Length always equals `n.length`
- `n` - Names array (item names like ["SliceExt", "alloc", "boxed", ...])
  - Empty string "" means "reuse lastName" for compression
- `q` - Qualified paths array (used to build `path` and `exactPath`)
- `p` - Path/parent data array
  - Each entry is `[ty, name, pathIdx?, exactPathIdx?, unboxFlag?]`
  - Used to build the `paths` array for parent lookups

**Encoded/compressed fields:**
- `i` - Parent indices (VLQ hex encoded)
- `f` - Function type signatures (VLQ hex encoded)
- `D` - Description shard lengths (VLQ hex encoded)
- `P` - Parameter names (comma-separated strings in a Map)
- `b` - Impl disambiguators (Map)
- `c` - Deprecated items bitmap (RoaringBitmap)
- `e` - Empty description bitmap (RoaringBitmap)
- `r` - Re-exports map (for `exactPath`)
- `a` - Aliases (optional field, only present in some crates)

## Expanded Format (after initSearch)

After the search engine processes the raw data via `buildIndex()`, each item becomes:

```javascript
{
  crate: "allocator_api2",          // From Map key
  ty: 10,                            // From t.charCodeAt(i) - 65
  name: "SliceExt",                  // From n[i] (or lastName if "")
  path: "allocator_api2",            // From q (itemPaths.get(i)) or lastPath
  exactPath: "allocator_api2",       // From r (reexports) or falls back to path
  normalizedName: "sliceext",        // name with underscores removed
  id: 1,                             // Auto-incrementing counter
  word: "sliceext",                  // Lowercase name
  descShard: {...},                  // Tracks description location (from D field)
  descIndex: 0,                      // Index within description shard
  parent: undefined,                 // From i field (itemParentIdxDecoder)
  type: null,                        // From f field (itemFunctionDecoder)
  paramNames: [],                    // From P field (split by comma)
  bitIndex: 1,                       // i + 1
  implDisambiguator: null            // From b field
}
```

### Field Mapping Summary

| Raw Field | Expanded Field(s) | Decoding Method |
|-----------|------------------|-----------------|
| `t[i]` | `ty` | `t.charCodeAt(i) - 65` |
| `n[i]` | `name`, `word`, `normalizedName` | Direct array lookup |
| `q` | `path`, `exactPath` | Map lookup via `itemPaths` |
| `i` | `parent` | VLQ hex decoder → `paths[idx-1]` |
| `f` | `type` | VLQ hex decoder with function type parser |
| `P` | `paramNames` | Map lookup, split by comma |
| `b` | `implDisambiguator` | Map lookup |
| `D` | `descShard` | VLQ hex decoder for shard lengths |
| `c` | - | RoaringBitmap for deprecated tracking |
| `e` | - | RoaringBitmap for empty desc tracking |
| `r` | `exactPath` | Map lookup for re-exports |
| `p` | - | Builds `paths` array for parent lookups |
| `a` | - | Processed separately into ALIASES map |

## Decoding Process

The `buildIndex()` method in `search-*.js` (around line 1295):

1. Iterates over each `[crate, crateCorpus]` in the `rawSearchIndex` Map
2. Destructures: `const { t, n, p, a } = crateCorpus;`
3. Sets up decoders for VLQ hex fields (`i`, `f`, `D`)
4. Loops through each item: `for (let i = 0; i < t.length; ++i)`
5. For each position `i`:
   - Gets `name` from `n[i]`
   - Decodes `ty` from `t.charCodeAt(i) - 65`
   - Looks up other fields via decoders/maps
   - Creates row object and pushes to `searchIndex` array

## Tools

**Parse the search index:**
```bash
bun run ./scripts/bun/parse-search-index.js
```

**Get specific crate data:**
```bash
bun run ./scripts/bun/get-itertools.js
```
