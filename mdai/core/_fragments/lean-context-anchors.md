@markdownai v1.0

- [ ] `mode="full"` — only allowed for the one spec-source read; flag all others.
- [ ] `raw=true` — every `ctx_shell raw=true` needs `@note visible consumer="human"`.
- [ ] `fresh=true` — only valid immediately after a write/edit to the same path.
- [ ] `Grep` / `rg ` — lean-ctx violation; replace with `@call ctx_search(...)`.
- [ ] `cat ` / `head ` / `tail ` — lean-ctx violation; replace with `@call ctx_read(...)`.
- [ ] `bash ` / `sh ` — lean-ctx violation; replace with `@call ctx_shell(...)`.
