# Typed Show Filters — Implementation Tasks

> Ref: [design.md](design.md)

## Tasks

- [ ] 1. Add filter enums and `ShowTypeFilter` wrapper
  - [ ] 1.1 **[RED]** Write unit tests for `IndexFilter`, `ConstraintFilter`, `CallableFilter` `Display` output (e.g. `IndexFilter::Range` renders `"RANGE"`, `ConstraintFilter::NotNull` renders `"NOT NULL"`)
  - [ ] 1.2 **[RED]** Write unit tests for `ShowTypeFilter` wrapper `Display` delegation
  - [ ] 1.3 **[GREEN]** Implement `IndexFilter`, `ConstraintFilter`, `CallableFilter` enums with `Display` in `src/admin/mod.rs`
  - [ ] 1.4 **[GREEN]** Implement `ShowTypeFilter` wrapper enum with `Display` and `From` impls
  - [ ] 1.5 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 2. Update `ShowCommand` to store `Option<ShowTypeFilter>`
  - [ ] 2.1 **[RED]** Update `ShowCommand` unit tests in `src/admin/show.rs` to use enum variants instead of strings
  - [ ] 2.2 **[GREEN]** Change `type_filter` field from `Option<Cow<'static, str>>` to `Option<ShowTypeFilter>`
  - [ ] 2.3 **[GREEN]** Update `with_type_filter()` to accept `ShowTypeFilter`
  - [ ] 2.4 **[GREEN]** Update `type_filter()` accessor to return `Option<&ShowTypeFilter>`
  - [ ] 2.5 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 3. Replace `ShowBuilder` with five typed builders
  - [ ] 3.1 **[RED]** Write builder tests: each typed builder accepts its own filter enum, produces correct `Statement`
  - [ ] 3.2 **[GREEN]** Create `show_builder!` macro generating common methods (`yield_all`, `yield_fields`, `where_`, `build`)
  - [ ] 3.3 **[GREEN]** Implement `ShowIndexesBuilder` with `.filter(IndexFilter)`
  - [ ] 3.4 **[GREEN]** Implement `ShowConstraintsBuilder` with `.filter(ConstraintFilter)`
  - [ ] 3.5 **[GREEN]** Implement `ShowFunctionsBuilder` with `.filter(CallableFilter)` + `.executable_by*()` methods
  - [ ] 3.6 **[GREEN]** Implement `ShowProceduresBuilder` with `.filter(CallableFilter)` + `.executable_by*()` methods
  - [ ] 3.7 **[GREEN]** Implement `ShowTransactionsBuilder` with `.ids()` (no filter)
  - [ ] 3.8 **[GREEN]** Remove old `ShowBuilder` and `ShowKind`
  - [ ] 3.9 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 4. Update `Cypher` entry points
  - [ ] 4.1 **[GREEN]** Change return types of `show_indexes()`, `show_constraints()`, `show_functions()`, `show_procedures()`, `show_transactions()` in `src/cypher.rs`
  - [ ] 4.2 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 5. Update renderers
  - [ ] 5.1 **[GREEN]** Update `src/renderer/default.rs` to use `Display` on `ShowTypeFilter` instead of raw `&str`
  - [ ] 5.2 **[GREEN]** Update `src/renderer/pretty.rs` similarly
  - [ ] 5.3 **[GREEN]** Migrate renderer tests to use enum variants
  - [ ] 5.4 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 6. Update parser
  - [ ] 6.1 **[RED]** Update parser round-trip tests to assert `ShowTypeFilter` enum values
  - [ ] 6.2 **[GREEN]** Map parsed filter strings to `ShowTypeFilter` variants in `src/parser/admin.rs`
  - [ ] 6.3 **[VERIFY]** `cargo test` + `cargo clippy`

- [ ] 7. Update prelude, examples, and integration tests
  - [ ] 7.1 **[GREEN]** Re-export `IndexFilter`, `ConstraintFilter`, `CallableFilter`, `ShowTypeFilter` from `src/prelude.rs`
  - [ ] 7.2 **[GREEN]** Update `examples/admin.rs` to use typed filters
  - [ ] 7.3 **[GREEN]** Update `src/examples.rs` doc examples if they reference show filters
  - [ ] 7.4 **[GREEN]** Update integration tests in `tests/` to use typed filters
  - [ ] 7.5 **[VERIFY]** `cargo test` + `cargo clippy` + `cargo doc --no-deps`
