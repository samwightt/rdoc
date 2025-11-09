# rdoc Project Context

- Use TDD: write test → confirm fail → implement → confirm pass. One test at a time, not all at once.
- Keep files small. If search_items.rs gets hard to edit, split functionality into separate modules.
- Use `i` for loop indices, not `enumerate()` on ranges that already give indices.
- Store indices instead of duplicating data (like parent_index instead of full ParentInfo).
- Field names should be descriptive: `item_type` not `ty`.
- The `docs/FIELD_DECODING.md` file documents the rustdoc search index format based on JavaScript implementation.
- VLQ hex decoder is in `src/vlq.rs`. Format: chars <96 are continuation, >=96 are terminal, LSB is sign bit.
- Function type signatures (`f` field) are deferred - too complex with recursive generics.
- Run tests with `cargo test`, scan command with `cargo run -- scan <symbol>`.
