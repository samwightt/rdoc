## Editing

- Always use TDD. Work in small chunks. Write a small test, run the test for the MODULE to confirm the new test fails, then implement the code that will fix that test.
	- Do NOT write all tests at once, write them one at a time.
- If a function / method definition gets over 75 or 100 lines long, consider pulling it out into a separate method.
	- If you have two or more levels of nested `if let Some()...`, pull that logic out into a separate method and use `?` operator syntax.
- After you've finished a feature, run clippy and fix any issues.

## Code style

- Store indices instead of duplicating data (like parent_index instead of full ParentInfo).
- Avoid duplicating data if possible, unless it leads to performance benefits. Prefer using references like indices, names, etc. to reference other stuff.
- Field names should be descriptive: `item_type` not `ty`.

## Misc
- The `docs` folder contains docs on implementation of rust's js index format for docs.
- Run tests with the `--quiet` flag so it doesn't pollute things with too much info.
