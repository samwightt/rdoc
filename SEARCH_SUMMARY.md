# Rustdoc Search - Summary

## What we discovered:

1. **Rustdoc has two main files:**
   - `search-HASH.js` - The search engine (~95KB, heavily minified)
   - `search-index.js` - The search data (contains all items from all crates)

2. **How it works:**
   - `search-index.js` creates a `Map` object with crate data
   - `initSearch(searchIndex)` parses this and builds an internal search index
   - The parsed index is stored as `docSearch.searchIndex` - an array of ~98k items
   - Each item has: `name`, `crate`, `path`, `ty` (type), `normalizedName`, etc.

3. **The official search (`execQuery`) is broken in Node/Bun because:**
   - It expects a full browser environment with DOM
   - It needs `searchState` with many UI-related properties
   - It's designed for interactive browser use, not CLI use

4. **Solution: Manual search works perfectly!**
   - We can directly filter `docSearch.searchIndex` array
   - Much simpler and faster than trying to mock the browser environment
   - See `rustdoc-search.js` for a working implementation

## Usage:

```bash
# Search for anything containing "Commit"
bun rustdoc-search.js Commit

# Filter by crate
bun rustdoc-search.js scan --crate git_history

# Filter by type (5=struct, 7=fn, 10=trait, etc.)
bun rustdoc-search.js Repository --type 5

# Exact match only
bun rustdoc-search.js scan --exact
```

## Item Types:

- 2 = module
- 3 = crate
- 5 = struct  
- 6 = enum
- 7 = function
- 10 = trait
- 12 = trait method
- 13 = method
- 16 = macro

