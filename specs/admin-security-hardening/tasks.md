# Admin DSL Security Hardening -- Task List

## Tasks

- [ ] 1. Add `src/admin/validate.rs` with validation functions and unit tests
  - [ ] 1.1 Create `is_valid_identifier()`, `is_valid_transaction_id()`, `assert_valid_identifier()`, `assert_valid_transaction_id()`
  - [ ] 1.2 Add unit tests for both accept and reject cases
  - [ ] 1.3 Register `mod validate` in `src/admin/mod.rs`

- [ ] 2. Add reserved-keyword escaping to `needs_escaping()`
  - [ ] 2.1 Add sorted `RESERVED_KEYWORDS` slice and `is_reserved_keyword()` in `src/renderer/default.rs`
  - [ ] 2.2 Extend `needs_escaping()` to call `is_reserved_keyword()`
  - [ ] 2.3 Add unit tests for reserved-word escaping (e.g., `MATCH`, `RETURN`, `SET`)

- [ ] 3. Harden admin renderer: index commands
  - [ ] 3.1 `write_create_index()`: escape index name via `write_safe_identifier()`
  - [ ] 3.2 `write_index_target()`: escape variables via `write_safe_identifier()`, labels/rel types via `write_safe_identifier()`, properties via `write_safe_identifier()`
  - [ ] 3.3 `write_drop_index()`: escape name via `write_safe_identifier()`
  - [ ] 3.4 Remove `#[allow(clippy::unused_self)]` from methods that now use `self`

- [ ] 4. Harden admin renderer: constraint commands
  - [ ] 4.1 `write_create_constraint()`: escape constraint name, variable, label, rel type, properties, type name via safe primitives
  - [ ] 4.2 `write_drop_constraint()`: escape name via `write_safe_identifier()`

- [ ] 5. Harden admin renderer: SHOW and TERMINATE commands
  - [ ] 5.1 `write_show_command()`: escape transaction IDs via `write_string_literal_static()`, username via `write_safe_identifier()`
  - [ ] 5.2 `write_terminate_transactions()`: escape transaction IDs via `write_string_literal_static()`

- [ ] 6. Add builder validation
  - [ ] 6.1 `IndexBuilder`: validate name, variable, properties in all `for_*()` methods
  - [ ] 6.2 `ConstraintBuilder` / `ConstraintRequire`: validate name, variable, properties
  - [ ] 6.3 `ShowFunctionsBuilder` / `ShowProceduresBuilder`: validate username in `executable_by()`
  - [ ] 6.4 `ShowTransactionsBuilder`: validate transaction IDs in `ids()`
  - [ ] 6.5 `TerminateBuilder` / `Cypher::terminate_transactions()`: validate transaction IDs
  - [ ] 6.6 `Cypher::drop_index()` / `drop_index_if_exists()` / `drop_constraint()` / `drop_constraint_if_exists()`: validate name

- [ ] 7. Add admin security tests to `tests/security_it.rs`
  - [ ] 7.1 Index name, variable, label, property injection tests
  - [ ] 7.2 Constraint name, variable, label, property injection tests
  - [ ] 7.3 Drop index/constraint name smuggling tests
  - [ ] 7.4 Transaction ID quote-breakout tests (SHOW and TERMINATE)
  - [ ] 7.5 Username injection tests
  - [ ] 7.6 Property type name injection test
  - [ ] 7.7 Reserved keyword as admin identifier test

- [ ] 8. Verify: `cargo test` + `cargo clippy` + `cargo doc --no-deps`
