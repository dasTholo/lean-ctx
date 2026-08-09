# Preamble — Round RXX: [TITLE]

## Project
lean-ctx is a Rust-based context engineering layer for LLM agents.
Repo: `/path/to/lean-ctx/rust/`

## Round Goal
[1-2 sentences: what this round achieves]

## Shared Type Definitions
[PASTE EXACT Rust code for any types/traits/enums that multiple agents reference.
This is the single source of truth — agents copy from here, not invent their own.]

```rust
// Example:
pub struct MyType {
    pub field_a: String,
    pub field_b: u64,
}
```

## Module Map (NO FILE OVERLAP)

| Agent | Files (CREATE/MODIFY) | Scope |
|---|---|---|
| 01 | `src/core/module/types.rs` (NEW) | Types + traits |
| 02 | `src/core/module/impl.rs` (NEW) | Implementation |
| 03 | `src/core/module/bridge.rs` (NEW) | Integration |
| 04 | `src/tools/tool.rs` (MODIFY L42-50) | Wire to tool |

**RULE: Each file has exactly ONE owner. If two agents need the same file, restructure.**

## Build & Verify (MANDATORY before committing)

```bash
cd rust
cargo fmt
cargo clippy --features FEATURE_NAME -- -D warnings
cargo test --features FEATURE_NAME --lib MODULE_NAME
```

If `cargo clippy` fails, fix ALL errors before committing.
If you cannot commit (hook timeout), leave files staged — the orchestrator will commit.

## CRITICAL RULES — violations cause revert

1. **NO wildcard imports** — `use super::types::*` is FORBIDDEN.
   Write: `use super::types::{TypeA, TypeB, TypeC};`

2. **Struct constructor field order = definition order.**
   If struct is `{ a, b, c }`, constructor MUST be `Foo { a: .., b: .., c: .. }`.

3. **Test-only imports inside test module.**
   ```rust
   // WRONG: import at file top, used only in #[cfg(test)]
   use super::types::TestHelper;

   // RIGHT:
   #[cfg(test)]
   mod tests {
       use super::super::types::TestHelper;
   }
   ```

4. **`sort_by` patterns** — prefer `sort_by_key` or `sort_unstable_by_key`:
   ```rust
   // WRONG:
   items.sort_by(|a, b| a.name.cmp(&b.name));
   // RIGHT:
   items.sort_by_key(|item| item.name.clone());
   ```

5. **`map().unwrap_or_else()` → `map_or_else()`:**
   ```rust
   // WRONG:
   opt.map(|v| v.to_string()).unwrap_or_else(|| "default".to_string());
   // RIGHT:
   opt.map_or_else(|| "default".to_string(), |v| v.to_string());
   ```

6. **`impl Default` → `#[derive(Default)]`** when all fields have sensible defaults.

7. **No `unwrap()` outside tests.** Use `?`, `.unwrap_or()`, or `.ok()?`.

8. **Private helpers with >8 parameters** — add `#[allow(clippy::too_many_arguments)]`.

9. **Generic containers need type annotations:**
   ```rust
   // WRONG:
   let mut items = Vec::new();
   // RIGHT:
   let mut items: Vec<MyType> = Vec::new();
   ```

10. **All new files MUST be <500 LOC.** Split into sub-modules if needed.

## Do NOT

- Modify files not listed in your Module Map
- Add dependencies to Cargo.toml
- Create `mod.rs` entries for other agents' modules
- Use `println!` for debugging (use `tracing::debug!`)
- Leave TODO/FIXME comments
