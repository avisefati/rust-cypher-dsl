# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building and Running
- Build the project: `cargo build --release`
- Run the CLI: `cargo run --release -- [COMMAND]`
- Run with debug logs: `RUST_LOG=debug cargo run --release -- [COMMAND]`

### Testing
- Run all tests: `cargo test`
- Run integration tests (requires OpenAI API key): `cargo test --test integration_workflow`
- Run specific test module: `cargo test [MODULE_NAME]`


### Configuration
- Environment variables via `.env` file (see `env.example`)
- OpenAI API settings (model, temperature, batch size, timeouts)
- JWT token expiry times (access and refresh token minutes)
- Default languages and output directories
- Tracing/logging configuration

## Development Guidelines
## Git Branching Workflow

Before starting any new task:

1. **Switch to `developer`** and pull the latest changes: `git checkout developer && git pull`
2. **Create a dedicated branch** for the task: `git checkout -b task/<task-id>-<short-description>`
3. Implement the task on that branch.
4. When approved, commit, push, and open a PR targeting `developer`.

Never work directly on the `developer` or `main` branch. Every task gets its own branch.

**Important**: The working directory is already set to the repo root. Always run all commands directly without path prefixes — never use `git -C <path>`, always use plain `git status`, `git add`, `git commit`, etc. This applies to all tools (cargo, git, etc.).

---

## Development Workflow: Spec-Driven Development

This project follows a strict specification-driven workflow defined in `.cursor/rules/spec*.mdc`. All feature work must go through these phases in order:

### Phase 1: Requirements (`spec-requirements.mdc`)
- Create `specs/{feature_name}/requirements.md` with user stories and EARS-format acceptance criteria.
- Iterate with user until explicitly approved. Do not proceed without approval.

### Phase 2: Design (`spec-design-doc.mdc`)
- Create `specs/{feature_name}/design.md` with: Overview, Architecture, Components and Interfaces, Data Models, Error Handling, Testing Strategy.
- Research and incorporate findings. Use Mermaid for diagrams where helpful.
- Iterate with user until explicitly approved. Do not proceed without approval.

### Phase 3: Implementation Plan (`spec-implementation-plan.mdc`)
- Create `specs/{feature_name}/tasks.md` as a numbered checkbox list (max 2 levels).
- Each task must be a concrete coding action with references to requirements.
- Tasks must be incremental, test-driven, and build on each other.
- Only coding tasks -- no deployment, user testing, or non-code activities.
- Iterate with user until explicitly approved.

### Phase 4: Task Execution (`spec-task-execution.mdc`)
- Always read requirements.md, design.md, and tasks.md before executing.
- Execute **one task at a time**. Stop after each for user review.
- Update checkboxes in tasks.md after completing each task.
- Sub-tasks may proceed in sequence, pausing for review after each.
- **TDD is mandatory** for every coding task. Follow the red-green-refactor cycle:
    1. **Red**: Write failing tests first (stubs with `todo!()` if needed to compile).
    2. **Green**: Write minimal production code to make tests pass.
    3. **Refactor**: Clean up while keeping tests green.
- Communicate the current TDD phase: `[RED]`, `[GREEN]`, `[REFACTOR]`, `[VERIFY]`.

### Task Completion Criteria
- A task **cannot** be marked complete if any modified function body contains `bail!("...not yet implemented...")`, `todo!()`, or `unimplemented!()`. These are only acceptable as intermediate scaffolding during the RED phase.
- Before marking a task complete, verify no stubs remain by searching modified files for `todo!()`, `bail!`, `unimplemented!()`, and `"not yet implemented"`.
- Integration tests must exercise real production types, not only mocks. If a production struct exists, at least one test must instantiate it. If it fails due to missing runtime resources (e.g., model files), that failure mode itself should be tested.

---
## Rust Development Guidelines

The following guidelines are defined in `.cursor/rules/rust*.mdc` and must be followed:

### Error Handling (`rust-error-handling.mdc`)
- Use `Result` for recoverable errors; panic only for programming bugs/invariant violations.
- For this application: `anyhow`/`eyre` are acceptable for convenience.
- Use `?` operator with `.context("what was being attempted")` for propagation.
- Keep error types informative with line numbers and descriptive messages.

### Ownership and Borrowing (`rust-ownership.mdc`)
- Prefer references over `.clone()`. Transfer ownership when possible.
- Use `&str` for string parameters, `String` for owned data.
- Add explicit lifetime annotations when the compiler can't infer them.

### Type System (`rust-type-system.mdc`, `rust-ai-design.mdc`)
- Use newtypes for domain concepts (avoid primitive obsession).
- Use `PathBuf`/`Path` instead of `String` for file paths.
- Leverage enums for opcodes, addressing modes, directives, etc.
- Provide mockable interfaces for I/O operations.

### API Design (`rust-api-design.mdc`)
- Avoid vague names (Service, Manager, Handler, Helper). Use specific names.
- Follow Rust naming conventions: `as_`/`to_`/`into_` for conversions, no `get_` prefix on getters.
- Implement `Debug`, `Clone`, `Display`, `PartialEq`/`Eq` where appropriate.
- Use Builder pattern for types with many initialization options.

### CLI (`rust-clap.mdc`)
- Use `clap` with derive macros for argument parsing.
- Document all options with helpful messages.

### Testing (`rust-testing.mdc`)
- Place unit tests in the same file as the code under test.
- Use integration tests for public API behavior.
- Follow AAA (Arrange-Act-Assert) pattern.
- Test both success and error cases, including edge cases.

### Documentation (`rust-documentation.mdc`)
- Keep doc summary sentences under 15 words.
- Document error conditions and panic conditions explicitly.
- Document magic constants with rationale.

### Performance (`rust-performance.mdc`)
- Pre-allocate with `.with_capacity()` when size is known or estimable.
- Avoid unnecessary heap allocations.
- Prefer static dispatch over `dyn Trait` in hot paths.

### Static Analysis (`rust-static-analysis.mdc`)
- Use `#[expect(lint, reason = "...")]` instead of `#[allow]`.
- Enable clippy pedantic lints.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

### Safety (`rust-safety.mdc`)
- Unsound code is never acceptable.
- Document any `unsafe` blocks with `// SAFETY:` comments.

### Cargo (`rust-cargo.mdc`)
- Use specific version constraints for dependencies.
- All features must be additive.

### Observability (`rust-observability.mdc`)
- Use structured logging if logging is added.
- Never log sensitive data.

### Async (`rust-async.mdc`)
- This project is not expected to use async, but if needed: avoid blocking in async contexts, use timeouts for I/O.

### PostgreSQL Database Standards
Follow the detailed guidelines in `.cursor/rules/design-postgress-tables.mdc`:
- **Primary Keys**: Prefer `BIGINT GENERATED ALWAYS AS IDENTITY` for internal tables; use `UUID` for API-exposed entities
- **Data Types**: Use `TIMESTAMPTZ` for timestamps, `NUMERIC` for money, `TEXT` for strings; avoid `VARCHAR(n)`, `CHAR(n)`, `TIMESTAMP` without timezone
- **Indexing**: Always index FK columns manually; use `CREATE INDEX CONCURRENTLY` for production migrations
- **Constraints**: Add `NOT NULL` where semantically required; use `CHECK` constraints for validation
- **Migration Safety**: Use `NOT VALID` then `VALIDATE CONSTRAINT` for FKs; avoid `ACCESS EXCLUSIVE` locks on large tables
- **JSONB**: Index with GIN for containment queries; keep core relations in tables, use JSONB for optional/variable attributes
- **RLS**: Enable Row-Level Security with `ALTER TABLE ENABLE ROW LEVEL SECURITY` and create appropriate policies
- verification step for each task will be: cargo build + cargo test + cargo clippy --all-targets --all-features -- -D warnings