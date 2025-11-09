# rdoc

A Rust CLI tool for searching rustdoc generated documentation via the command line. Lets you search for symbols, `cat` documentation for them, and other things.

## Overview

Rustdoc generates a `search-index.js` file containing a compressed index of all documented items. This project provides tools to parse that index format and make it searchable.

## Features

- Parse rustdoc's search index format
- Type-safe representation of search items
- VLQ (Variable-Length Quantity) hex decoder
- Parent relationship tracking

## Usage

```bash
# Scan for a symbol
cargo run -- scan Result
```

## Project Structure

- `src/search_index.rs` - Parses the raw search index format
- `src/search_items.rs` - Decodes items into searchable structures
- `src/vlq.rs` - VLQ hex decoder for compressed data
- `src/commands/` - CLI commands
- `docs/` - Additional documentation

## Development Status

This is an early-stage project. Currently implemented:
- Search index parsing
- Item type decoding
- Parent index decoding

Still to implement:
- Function type signatures
- Bitmap fields (deprecated, empty descriptions)
- Full-text search functionality

## Documentation

See [docs/FIELD_DECODING.md](docs/FIELD_DECODING.md) for details on the rustdoc search index format.
