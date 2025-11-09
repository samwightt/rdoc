# Search Index Field Decoding Documentation

This document describes how the remaining fields in rustdoc's `search-index.js` format are decoded, based on analysis of `search-92309212.js`.

## Overview

The search index uses several compression techniques:
- **VLQ (Variable-Length Quantity) Hex Encoding**: For numeric sequences
- **Roaring Bitmaps**: For efficiently storing sets of item indices
- **Sparse Arrays**: Already parsed as JSON structures

## Field Decoding Details

### Field 'i' - Parent Indices (VLQ Hex Encoded)

**Location in buildIndex**: Lines 1379-1382, 1436, 1452

```javascript
const itemParentIdxDecoder = new VlqHexDecoder(
    crateCorpus.i,
    (noop) => noop,
);
// ... later in the loop:
const itemParentIdx = itemParentIdxDecoder.next();
// ... used as:
parent: itemParentIdx > 0 ? paths[itemParentIdx - 1] : undefined,
```

**Decoding Process**:
1. Create a `VlqHexDecoder` with the `i` string
2. For each item, call `.next()` to get the parent index
3. If `itemParentIdx > 0`, it points into the `paths` array (1-based indexing, subtract 1 for 0-based)
4. If `itemParentIdx == 0`, the item has no parent (is top-level)

**Result**: Each item gets a `parent` field referencing an entry in the `paths` array, or `undefined`.

---

### Field 'f' - Function Type Signatures (VLQ Hex Encoded)

**Location in buildIndex**: Lines 1386-1389, 1423, 1424-1437

```javascript
const itemFunctionDecoder = new VlqHexDecoder(
    crateCorpus.f,
    buildFunctionSearchTypeCallback(paths, lowercasePaths),
);
// ... later in the loop:
const type = itemFunctionDecoder.next();
```

**Decoding Process**:
1. Create a `VlqHexDecoder` with the `f` string
2. Use `buildFunctionSearchTypeCallback` as the transformation callback
3. For each item, call `.next()` to get the function type signature
4. The callback transforms the decoded values into a structured object with:
   - `inputs`: Array of input parameter types
   - `output`: Array of output/return types
   - `where_clause`: Array of where clause constraints

**Result**: Each item gets a `type` field containing `null` (for non-functions) or an object with `{ inputs, output, where_clause }`.

---

### Field 'D' - Description Shard Lengths (VLQ Hex Encoded)

**Location in buildIndex**: Lines 1365-1377, 1410-1421

```javascript
const itemDescShardDecoder = new VlqHexDecoder(crateCorpus.D, (noop) => {
    const n = noop;
    return n;
});

let descShard = {
    crate,
    shard: 0,
    start: 0,
    len: itemDescShardDecoder.next(),
    promise: null,
    resolve: null,
};
```

**Decoding Process**:
1. Create a `VlqHexDecoder` with the `D` string
2. Call `.next()` to get the length of the first description shard
3. As items are processed, track `descIndex` (position within current shard)
4. When `descIndex >= descShard.len`, advance to the next shard:
   - Increment `shard` number
   - Update `start` position
   - Call `.next()` again to get the next shard length
   - Reset `descIndex` to 0

**Note**: Items with empty descriptions (marked in field 'e') don't increment `descIndex`.

**Result**: Descriptions are logically partitioned into shards. Each item gets a `descShard` reference and `descIndex` indicating where its description lives.

---

### Field 'c' - Deprecated Items Bitmap

**Location in buildIndex**: Line 1380

```javascript
this.searchIndexDeprecated.set(crate, new RoaringBitmap(crateCorpus.c));
```

**Decoding Process**:
1. Parse the `c` string as a `RoaringBitmap`
2. Store per-crate in `searchIndexDeprecated` map
3. To check if an item is deprecated: `searchIndexDeprecated.get(crate).contains(bitIndex)`

**Result**: A compressed bitmap that can efficiently test whether any item (by its `bitIndex`) is deprecated.

---

### Field 'e' - Empty Description Bitmap

**Location in buildIndex**: Lines 1381, 1401, 1411, 1461

```javascript
this.searchIndexEmptyDesc.set(crate, new RoaringBitmap(crateCorpus.e));

// Usage:
if (!this.searchIndexEmptyDesc.get(crate).contains(bitIndex)) {
    descIndex += 1;
}
```

**Decoding Process**:
1. Parse the `e` string as a `RoaringBitmap`
2. Store per-crate in `searchIndexEmptyDesc` map
3. To check if an item has an empty description: `searchIndexEmptyDesc.get(crate).contains(bitIndex)`

**Result**: A compressed bitmap marking which items have no description. This is used to skip incrementing `descIndex` for items without descriptions.

---

### Field 'a' - Aliases

**Location in buildIndex**: Lines 1374, 1466-1485

```javascript
const { t, n, p, a } = crateCorpus;
// ...
if (a) {
    const currentCrateAliases = new Map();
    this.ALIASES.set(crate, currentCrateAliases);
    for (const alias_name in a) {
        if (!Object.hasOwn(a, alias_name)) {
            continue;
        }
        let currentNameAliases;
        if (currentCrateAliases.has(alias_name)) {
            currentNameAliases = currentCrateAliases.get(alias_name);
        } else {
            currentNameAliases = [];
            currentCrateAliases.set(alias_name, currentNameAliases);
        }
        for (const local_alias of a[alias_name]) {
            currentNameAliases.push(local_alias + currentIndex);
        }
    }
}
```

**Decoding Process**:
1. Field `a` is already a JavaScript object (HashMap) from JSON
2. Structure: `{ "alias_name": [item_index1, item_index2, ...], ... }`
3. For each alias, adjust item indices by adding `currentIndex` (the offset for this crate in the global index)
4. Store in per-crate aliases map

**Result**: A map from alias names to arrays of global item indices, allowing items to be found by alternative names.

---

### Field 'p' - Parent Items

**Location in buildIndex**: Lines 1374, 1390-1415

```javascript
const { t, n, p, a } = crateCorpus;
// ...
let p_length = p.length;
let lastPath = undef2null(itemPaths.get(0));
for (let i = 0; i < p_length; ++i) {
    const p_i = p[i];
    const [ty, name] = p_i;
    const elemPath = (idx, if_null, if_not_found) => {
        if (p_i.length > idx && p_i[idx] !== undefined) {
            const p = itemPaths.get(p_i[idx]);
            if (p !== undefined) {
                return p;
            }
            return if_not_found;
        }
        return if_null;
    };
    const path = elemPath(2, lastPath, null);
    const exactPath = elemPath(3, path, path);
    const unboxFlag = p_i.length > 4 && !!p_i[4];
    lowercasePaths.push({
        ty,
        name: name.toLowerCase(),
        path,
        exactPath,
        unboxFlag,
    });
    paths[i] = { ty, name, path, exactPath, unboxFlag };
}
```

**Decoding Process**:
1. Field `p` is already an array parsed from JSON
2. Each element `p[i]` is an array with:
   - **Index 0**: `ty` (ItemType number)
   - **Index 1**: `name` (string)
   - **Index 2** (optional): path index into `itemPaths` map
   - **Index 3** (optional): exact path index into `itemPaths` map
   - **Index 4** (optional): `unboxFlag` (boolean)
3. Path compression: if index 2 is missing, reuse `lastPath`
4. Exact path: if index 3 is missing, use `path`
5. Build two parallel arrays: `paths` and `lowercasePaths`

**Result**: The `paths` array contains parent item information. Regular items reference this array via their parent index (from field 'i').

---

## Summary Table

| Field | Type | Decoding Method | Purpose |
|-------|------|-----------------|---------|
| `i` | VLQ Hex | `VlqHexDecoder` → numeric sequence | Parent item indices (1-based, 0 = no parent) |
| `f` | VLQ Hex | `VlqHexDecoder` + callback → objects | Function type signatures (inputs/outputs/where) |
| `D` | VLQ Hex | `VlqHexDecoder` → numeric sequence | Description shard lengths |
| `c` | Bitmap String | `RoaringBitmap` | Deprecated items bitmap |
| `e` | Bitmap String | `RoaringBitmap` | Empty description items bitmap |
| `a` | JSON Object | Already parsed | Alias name → item indices mapping |
| `p` | JSON Array | Already parsed | Parent item metadata (type, name, paths) |

## Implementation Notes

### VlqHexDecoder
This class decodes Variable-Length Quantity hex-encoded strings. Each call to `.next()` consumes part of the string and returns the next decoded value. An optional callback can transform the decoded value.

### RoaringBitmap
A compressed bitmap data structure for efficiently storing and querying sets of integers. Supports `.contains(index)` to test membership.

### Index Offsets
When combining multiple crates into a single global search index, item indices are adjusted by `currentIndex` (the running total of items from previous crates).

---

## VlqHexDecoder Implementation Details

**Location**: Lines 740-791 in search-92309212.js

The `VlqHexDecoder` class implements a stateful decoder for Variable-Length Quantity (VLQ) hex-encoded data with backreference support.

### Constructor

```javascript
constructor(string, cons) {
    this.string = string;      // The hex-encoded string to decode
    this.cons = cons;          // Constructor/callback function to transform decoded values
    this.offset = 0;           // Current position in the string
    this.backrefQueue = [];    // Queue of recent results for backreferences (max 16)
}
```

### Core Decoding Methods

#### `decode()` - Decode a single integer

```javascript
decode() {
    let n = 0;
    let c = this.string.charCodeAt(this.offset);
    if (c === 123) {  // '{' character
        this.offset += 1;
        return this.decodeList();
    }
    // Read hex digits while char code < 96 (continuation bytes)
    while (c < 96) {
        n = (n << 4) | (c & 15);  // 0xf - extract low 4 bits
        this.offset += 1;
        c = this.string.charCodeAt(this.offset);
    }
    // Last byte (char code >= 96)
    n = (n << 4) | (c & 15);  // 0xf
    const [sign, value] = [n & 1, n >> 1];  // LSB is sign bit
    this.offset += 1;
    return sign ? -value : value;
}
```

**Encoding Details**:
- Characters with code < 96 are continuation bytes (more data follows)
- Characters with code >= 96 are terminal bytes (last byte of the number)
- Each character contributes 4 bits to the number (low nibble: `c & 15`)
- The LSB of the final number is the sign bit: 0 = positive, 1 = negative
- The actual value is `n >> 1` (shift right to remove sign bit)

**Example**:
- Input: `"A"` (char code 65)
  - 65 >= 96, so it's a terminal byte
  - `n = 0 | (65 & 15) = 1`
  - Sign bit: `1 & 1 = 1` (negative)
  - Value: `1 >> 1 = 0`
  - Result: `-0 = 0`

- Input: `"a"` (char code 97)
  - 97 >= 96, so it's a terminal byte
  - `n = 0 | (97 & 15) = 1`
  - Sign bit: `1 & 1 = 1` (negative)
  - Value: `1 >> 1 = 0`
  - Result: `-0 = 0`

#### `decodeList()` - Decode a list of values

```javascript
decodeList() {
    let c = this.string.charCodeAt(this.offset);
    const ret = [];
    while (c !== 125) {  // '}' character ends the list
        ret.push(this.decode());
        c = this.string.charCodeAt(this.offset);
    }
    this.offset += 1;  // Skip the closing '}'
    return ret;
}
```

**Usage**: Lists are encoded as `{` followed by encoded values, terminated by `}`.

#### `next()` - High-level interface with backreferences

```javascript
next() {
    const c = this.string.charCodeAt(this.offset);
    
    // Backreference: char codes 48-63 ('0'-'9', ':', ';', '<', '=', '>', '?')
    if (c >= 48 && c < 64) {
        this.offset += 1;
        return this.backrefQueue[c - 48];  // Return previously decoded value
    }
    
    // Special case: '`' (char code 96) means cons(0)
    if (c === 96) {
        this.offset += 1;
        return this.cons(0);
    }
    
    // Normal decode path
    const result = this.cons(this.decode());
    
    // Add to backref queue (FIFO, max 16 items)
    this.backrefQueue.unshift(result);
    if (this.backrefQueue.length > 16) {
        this.backrefQueue.pop();
    }
    
    return result;
}
```

**Backreference System**:
- Characters `'0'` through `'?'` (codes 48-63) are backreferences
- They index into `backrefQueue`: `'0'` = index 0, `'1'` = index 1, etc.
- The queue stores the last 16 decoded results (after transformation by `cons`)
- This allows repeated values to be encoded as a single character

**Special Values**:
- `` ` `` (backtick, char code 96): Shorthand for `cons(0)`
- `{`: Start of a list
- Characters 48-63: Backreferences to recent values

### Encoding Format Summary

**Number Encoding**:
1. Numbers are encoded in base-16 (hex), 4 bits per character
2. Multi-byte numbers: continuation bytes (code < 96), then terminal byte (code >= 96)
3. Sign-magnitude representation: LSB of decoded number is sign, rest is magnitude
4. Formula: `value = (n >> 1) * (n & 1 ? -1 : 1)`

**Character Ranges**:
- `0-9, :, ;, <, =, >, ?` (48-63): Backreferences
- `@` through `_` (64-95): Continuation bytes for numbers
- `` ` `` and above (96+): Terminal bytes for numbers
- `{` (123): Start list
- `}` (125): End list

### Usage Patterns

**Simple numeric sequence** (e.g., parent indices):
```javascript
const decoder = new VlqHexDecoder("ABC", (n) => n);
decoder.next();  // Returns a number
decoder.next();  // Returns next number
```

**Complex objects** (e.g., function types):
```javascript
const decoder = new VlqHexDecoder(data, (decoded) => {
    if (decoded === 0) return null;
    // Transform decoded value into structured object
    return { inputs: [...], output: [...] };
});
decoder.next();  // Returns transformed object or null
```

### Compression Techniques

1. **Variable-length encoding**: Small numbers use fewer bytes
2. **Backreferences**: Repeated values encoded as single character (16-item cache)
3. **Special case for zero**: `` ` `` character (1 byte) instead of full encoding
4. **Nested structures**: Lists can contain numbers or other lists

---

## RoaringBitmap Implementation Details

**Location**: Lines 792-890 in search-92309212.js

The `RoaringBitmap` class implements a compressed bitmap data structure for efficiently storing and querying sets of integers. It uses a two-level structure: keys partition the 32-bit integer space into 64KB chunks, and containers store the actual values within each chunk.

### Constructor - Decoding from Base64

```javascript
constructor(str) {
    // Step 1: Decode base64 string to byte array
    const strdecoded = atob(str);
    const u8array = new Uint8Array(strdecoded.length);
    for (let j = 0; j < strdecoded.length; ++j) {
        u8array[j] = strdecoded.charCodeAt(j);
    }
    
    // Step 2: Check for run-length encoding
    const has_runs = u8array[0] === 59;  // 0x3b
    
    // Step 3: Read container count
    const size = has_runs
        ? (u8array[2] | (u8array[3] << 8)) + 1
        : u8array[4] | (u8array[5] << 8) | (u8array[6] << 16) | (u8array[7] << 24);
    
    // Step 4: Read run bitmap if present
    let i = has_runs ? 4 : 8;
    let is_run;
    if (has_runs) {
        const is_run_len = Math.floor((size + 7) / 8);
        is_run = u8array.slice(i, i + is_run_len);
        i += is_run_len;
    } else {
        is_run = new Uint8Array();
    }
    
    // Step 5: Read keys and cardinalities
    this.keys = [];
    this.cardinalities = [];
    for (let j = 0; j < size; ++j) {
        this.keys.push(u8array[i] | (u8array[i + 1] << 8));
        i += 2;
        this.cardinalities.push((u8array[i] | (u8array[i + 1] << 8)) + 1);
        i += 2;
    }
    
    // Step 6: Read container offsets (optional validation)
    this.containers = [];
    let offsets = null;
    if (!has_runs || this.keys.length >= 4) {
        offsets = [];
        for (let j = 0; j < size; ++j) {
            offsets.push(
                u8array[i] | (u8array[i + 1] << 8) |
                (u8array[i + 2] << 16) | (u8array[i + 3] << 24)
            );
            i += 4;
        }
    }
    
    // Step 7: Read containers (Run, Bits, or Array type)
    for (let j = 0; j < size; ++j) {
        if (offsets && offsets[j] !== i) {
            throw new Error(`corrupt bitmap ${j}: ${i} / ${offsets[j]}`);
        }
        if (is_run[j >> 3] & (1 << (j & 7))) {
            // Run container
            const runcount = u8array[i] | (u8array[i + 1] << 8);
            i += 2;
            this.containers.push(
                new RoaringBitmapRun(runcount, u8array.slice(i, i + runcount * 4))
            );
            i += runcount * 4;
        } else if (this.cardinalities[j] >= 4096) {
            // Bits container
            this.containers.push(new RoaringBitmapBits(u8array.slice(i, i + 8192)));
            i += 8192;
        } else {
            // Array container
            const end = this.cardinalities[j] * 2;
            this.containers.push(
                new RoaringBitmapArray(this.cardinalities[j], u8array.slice(i, i + end))
            );
            i += end;
        }
    }
}
```

### Binary Format

**Header** (varies by format):
- **With runs** (`has_runs = true`):
  - Byte 0: `0x3b` (59) - magic number indicating run-length encoding
  - Bytes 2-3: Container count - 1 (little-endian u16)
  - Bytes 4+: Run bitmap (1 bit per container)
  
- **Without runs** (`has_runs = false`):
  - Bytes 0-3: (unused in this code path)
  - Bytes 4-7: Container count (little-endian u32)

**Keys and Cardinalities** (after header + run bitmap):
- For each of `size` containers:
  - 2 bytes: Key (u16 little-endian) - upper 16 bits of integers in this container
  - 2 bytes: Cardinality - 1 (u16 little-endian) - number of values minus 1

**Offsets** (optional, for validation):
- For each container (if `!has_runs || keys.length >= 4`):
  - 4 bytes: Offset (u32 little-endian) - byte position of container data

**Container Data**:
- Three types based on cardinality and run flag:

### Container Types

#### 1. RoaringBitmapRun (Run-Length Encoded)

Used when the container is marked as a run in the `is_run` bitmap.

```javascript
class RoaringBitmapRun {
    constructor(runcount, array) {
        this.runcount = runcount;
        this.array = array;  // runcount * 4 bytes
    }
    
    contains(value) {
        let left = 0;
        let right = this.runcount - 1;
        while (left <= right) {
            const mid = Math.floor((left + right) / 2);
            const i = mid * 4;
            const start = this.array[i] | (this.array[i + 1] << 8);
            const lenm1 = this.array[i + 2] | (this.array[i + 3] << 8);
            if (start + lenm1 < value) {
                left = mid + 1;
            } else if (start > value) {
                right = mid - 1;
            } else {
                return true;  // value in range [start, start + lenm1]
            }
        }
        return false;
    }
}
```

**Format**:
- First 2 bytes: Run count (u16)
- Then for each run (4 bytes):
  - Bytes 0-1: Start value (u16)
  - Bytes 2-3: Length - 1 (u16)
- Each run represents consecutive integers: `[start, start + lenm1]`

**Query**: Binary search to find if value falls within any run range.

#### 2. RoaringBitmapBits (Bitmap)

Used when cardinality >= 4096 (dense container).

```javascript
class RoaringBitmapBits {
    constructor(array) {
        this.array = array;  // 8192 bytes = 65536 bits
    }
    
    contains(value) {
        return !!(this.array[value >> 3] & (1 << (value & 7)));
    }
}
```

**Format**:
- 8192 bytes (65536 bits total)
- Each bit represents whether that value (0-65535) is present
- Bit layout: byte `i` contains bits for values `i*8` through `i*8+7`

**Query**: Direct bit lookup - `O(1)` time.

#### 3. RoaringBitmapArray (Sorted Array)

Used when cardinality < 4096 (sparse container) and not a run.

```javascript
class RoaringBitmapArray {
    constructor(cardinality, array) {
        this.cardinality = cardinality;
        this.array = array;  // cardinality * 2 bytes
    }
    
    contains(value) {
        let left = 0;
        let right = this.cardinality - 1;
        while (left <= right) {
            const mid = Math.floor((left + right) / 2);
            const i = mid * 2;
            const x = this.array[i] | (this.array[i + 1] << 8);
            if (x < value) {
                left = mid + 1;
            } else if (x > value) {
                right = mid - 1;
            } else {
                return true;
            }
        }
        return false;
    }
}
```

**Format**:
- Sorted array of u16 values (2 bytes each)
- `cardinality` values total

**Query**: Binary search - `O(log n)` time.

### Main Query Method

```javascript
contains(keyvalue) {
    const key = keyvalue >> 16;        // Upper 16 bits
    const value = keyvalue & 0xffff;   // Lower 16 bits
    
    // Binary search for the key in keys array
    let left = 0;
    let right = this.keys.length - 1;
    while (left <= right) {
        const mid = Math.floor((left + right) / 2);
        const x = this.keys[mid];
        if (x < key) {
            left = mid + 1;
        } else if (x > key) {
            right = mid - 1;
        } else {
            // Found the right container, delegate to it
            return this.containers[mid].contains(value);
        }
    }
    return false;
}
```

**Two-Level Lookup**:
1. Split 32-bit integer into upper 16 bits (key) and lower 16 bits (value)
2. Binary search `keys` array to find the right container
3. Delegate to the container's `contains()` method

### Data Structure Summary

**Roaring Bitmap** partitions the 32-bit integer space:
- **Keys**: Upper 16 bits (0-65535) - which 64KB chunk
- **Containers**: Lower 16 bits (0-65535) - values within that chunk

**Container Selection**:
- **Run**: Consecutive ranges (marked in `is_run` bitmap)
- **Bits**: Dense sets (cardinality >= 4096) - 8192 bytes
- **Array**: Sparse sets (cardinality < 4096) - sorted array

**Space Efficiency**:
- Array: 2 bytes per value (best for sparse)
- Bits: 8192 bytes fixed (best for dense, ~50%+ full)
- Run: 4 bytes per consecutive range (best for consecutive values)

### Usage in Search Index

The rustdoc search uses RoaringBitmap for two purposes:
1. **Deprecated items** (`c` field): `bitmap.contains(bitIndex)` checks if item is deprecated
2. **Empty descriptions** (`e` field): `bitmap.contains(bitIndex)` checks if item has no description

Both use `bitIndex` (1-based item position) as the integer to test for membership.


