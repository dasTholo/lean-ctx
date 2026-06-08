# Merge `main` → `feat-jetbrains-plugin` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 54 main-Commits in den Branch `feat-jetbrains-plugin` integrieren, sodass die zwei roten Tests (bereits auf main gefixt) grün werden — ohne dass die JetBrains-/Phase-5-Arbeit verloren geht.

**Architecture:** Ein einzelner `git merge main` mit kontrollierter Auflösung der genau zwei vorhergesagten Konflikte (`rust/Cargo.toml`, `rust/tests/hardening_ir_traits.rs`). Vorab wird ein Backup-Branch gesetzt und der Working-Tree sauber gemacht. Akzeptanz = grünes `cargo nextest run` (5079 Tests) + sauberer `cargo clippy`.

**Tech Stack:** git, cargo (nextest, clippy), Rust.

---

## Hintergrund (warum dieser Merge)

- Die zwei roten Tests in `rust/tests/hn_hardening_scenarios.rs` (`scenario_skip_terse_when_already_compressed`, `scenario_shell_compression_with_saved_tokens_skips_terse`) sind **Struktur-Guard-Tests**. Sie brachen durch den main-Refactor `5369cea1` (*refactor guarded dispatch*, 06-05), der `skip_terse` aufteilte (`tool_saved_tokens > 0` lebt jetzt in `finalize_token_count_and_adjust`, `post_process.rs:183`).
- main hat sie bereits repariert: `d7e87825` (*fix(tests): align hn_hardening structural assertions*, 06-08 09:02). `git branch --contains d7e87825` → **nur `main`**.
- Unser Branch zweigt bei merge-base `9c8aa798` (06-07 21:53) ab — **vor** dem Fix. Daher lokal rot, CI auf main grün.
- **Fazit:** Kein eigener Test-Fix nötig. Der Merge bringt `d7e87825` + die refaktorierte `post_process.rs` sauber herein.

## Vorhergesagte Konflikte (`git merge-tree --write-tree main HEAD`)

Genau **2** Konflikte; alles andere merged automatisch (u. a. `Cargo.lock`, `LOCK_ORDERING.md`, `config/mod.rs`, `server/execute.rs`, `shell/exec.rs`).

| Datei | Strategie | Begründung |
|-------|-----------|------------|
| `rust/Cargo.toml` | **main-Basis + nur `-jb`-Suffix** | main fügt deps hinzu; von uns bleibt nur die `-jb`-Version. `rushdown` (ungenutzter Spike-Dep aus `dc30d42c`, 0 Code-Referenzen) wird **entfernt**; unsere kosmetische Reformatierung wird verworfen |
| `rust/tests/hardening_ir_traits.rs` | **main gewinnt** (`--theirs`) | main ersetzt hardcoded `"69 trait-based tools"` durch dynamisches `tool_count()`; unsere `"69"→"72"`-Änderung wird dadurch obsolet |

---

## Task 0: Vorbereitung & Backup

**Files:** keine Code-Änderung. Working-Tree + Branch-Sicherung.

- [ ] **Step 1: Lokales `main` gegen Remote abgleichen**

Run (via `ctx_shell`, `cwd=/home/tholo/Scripts/lean-ctx/rust`):
```
git fetch origin
git log --oneline main..origin/main
```
Expected: **kein Output** (lokales `main` ist auf Stand). Bei Output → `git checkout main && git merge --ff-only origin/main && git checkout feat-jetbrains-plugin` zuerst ausführen, dann hier weiter.

- [ ] **Step 2: Aktuell roten Baseline-Zustand bestätigen**

Run (`cwd=.../rust`): `cargo nextest run --no-fail-fast --status-level fail`
Expected: `5079 tests run: 5077 passed, 2 failed` — die zwei `hn_hardening_scenarios`-Tests. Das ist der dokumentierte Ausgangszustand.

- [ ] **Step 3: Sicherungs-Branch setzen (Rückfahrkarte)**

Run (`cwd=.../rust`): `git branch backup/pre-main-merge-2026-06-08`
Expected: kein Output. Bei Bedarf später Rückkehr via `git reset --hard backup/pre-main-merge-2026-06-08`.

- [ ] **Step 4: Working-Tree sauber machen — modifizierte getrackte Docs committen**

Vor einem Merge muss der Working-Tree clean sein. Es sind 4 getrackte, modifizierte Dateien offen (alle unter `docs/lean-md/`). Untracked Dateien (`.idea/`, `.serena/`, `markdownai/`, `rust/.config/`, `*.lock`) blockieren den Merge **nicht** (kein main-Commit berührt diese Pfade) — sie bleiben liegen.

Run (`cwd=/home/tholo/Scripts/lean-ctx`):
```
git add docs/lean-md/plans/2026-06-08-jetbrains-phase5a-hardening.md docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md docs/lean-md/specs/2026-06-08-jetbrains-phase5a-hardening-design.md docs/lean-md/specs/2026-06-08-jetbrains-phase5b-inspections-design.md docs/lean-md/plans/2026-06-08-merge-main-into-jetbrains-branch.md
git commit -m "docs(jetbrains): Phase-5 spec/plan updates + main-merge plan"
```
Expected: ein Commit mit 5 Dateien.

- [ ] **Step 5: Working-Tree-Sauberkeit verifizieren**

Run (`cwd=.../`): `git status --porcelain`
Expected: nur noch `??`-Zeilen (untracked), keine `M`/`A` mehr. Wenn doch → vor dem Merge committen oder stashen.

---

## Task 1: Merge starten

**Files:** Merge bricht mit 2 Konflikten ab (erwartet).

- [ ] **Step 1: Merge auslösen**

Run (`cwd=/home/tholo/Scripts/lean-ctx/rust` — gleicher Repo-Root genügt, `git` findet die `.git`):
```
git merge main --no-edit
```
Expected: Ausgabe endet mit `Automatic merge failed; fix conflicts and then commit the result.` und nennt Konflikte in `rust/Cargo.toml` und `rust/tests/hardening_ir_traits.rs`.

- [ ] **Step 2: Konfliktliste bestätigen**

Run (`cwd=.../`): `git diff --name-only --diff-filter=U`
Expected: **genau** diese zwei Zeilen:
```
rust/Cargo.toml
rust/tests/hardening_ir_traits.rs
```
Bei mehr/anderen Konflikten: STOPP, Plan-Annahme verletzt → Konflikte manuell prüfen, bevor fortgefahren wird.

---

## Task 2: Konflikt `rust/tests/hardening_ir_traits.rs` auflösen (main gewinnt)

**Files:**
- Resolve: `rust/tests/hardening_ir_traits.rs`

main ersetzt die hardcoded Tool-Count-Assertion durch eine dynamische Ableitung aus der Registry-SSOT (`lean_ctx::server::registry::tool_count()`). Unsere einzige Änderung an dieser Datei war `"69"→"72"` in genau dieser Assertion — sie wird durch main's Variante vollständig ersetzt. Daher: main's Version komplett übernehmen.

- [ ] **Step 1: main-Version der Datei übernehmen**

Run (`cwd=.../rust`): `git checkout --theirs tests/hardening_ir_traits.rs`
Expected: kein Output.

- [ ] **Step 2: Konfliktmarker-Freiheit verifizieren**

Run (`cwd=.../rust`): `git diff --check tests/hardening_ir_traits.rs`
Expected: kein Output (keine `<<<<<<<`/`=======`/`>>>>>>>`-Marker).

Inhaltlich muss der Block jetzt so aussehen (main-Variante, dynamisch):
```rust
let expected = format!(
    "{} trait-based tools",
    lean_ctx::server::registry::tool_count()
);
assert!(
    content.contains(&expected),
    "ARCHITECTURE.md should reference the current registry count ({expected})"
);
```

- [ ] **Step 3: Auflösung stagen**

Run (`cwd=.../rust`): `git add tests/hardening_ir_traits.rs`
Expected: kein Output.

---

## Task 3: Konflikt `rust/Cargo.toml` auflösen (main-Basis + nur `-jb`)

**Files:**
- Resolve: `rust/Cargo.toml`

Strategie: main-Version als Basis nehmen (enthält die neuen deps **und** die ursprüngliche 2-Space-Formatierung der Feature-Listen — unsere kosmetische 4-Space-Reformatierung wird bewusst verworfen, um künftige Diffs zu minimieren). Anschließend nur **ein** echtes Delta wieder aufsetzen: das `-jb`-Versionssuffix. `rushdown` wird **nicht** wieder eingefügt (ungenutzter Spike-Dep aus `dc30d42c`, 0 Code-Referenzen in `src/` und `tests/`) — damit fällt es sauber aus dem Branch heraus.

main bringt hinzu: `dep:tree-sitter-gdscript` (in `tree-sitter`-Feature), `wasm = ["dep:wasmi"]`, dep `tree-sitter-gdscript = "6.1"`, dep `wasmi = "1.0.9"`, dev-dep `wat = "1.251.0"`. Diese sind nach `--theirs` automatisch enthalten.

- [ ] **Step 1: main-Version der Cargo.toml übernehmen**

Run (`cwd=.../rust`): `git checkout --theirs Cargo.toml`
Expected: kein Output. (Da main `rushdown` nie hatte, ist es damit automatisch weg.)

- [ ] **Step 2: Versionssuffix `-jb` wieder setzen**

Bearbeite `rust/Cargo.toml` (non-Rust → `ctx_edit`):
```
old: version = "3.7.5"
new: version = "3.7.5-jb"
```
(Es gibt nur ein Vorkommen im `[package]`-Block, Zeile ~3.)

- [ ] **Step 3: Konfliktmarker- & Inhalts-Check**

Run (`cwd=.../rust`): `git diff --check Cargo.toml`
Expected: kein Output.

Run (`cwd=.../rust`): `ctx_search` nach `version = "3.7.5-jb"`, `tree-sitter-gdscript`, `wasmi`, `wat` in `rust/Cargo.toml`.
Expected: alle vier vorhanden.

Run (`cwd=.../rust`): `ctx_search` nach `rushdown` in `rust/Cargo.toml`.
Expected: **kein Treffer** (bewusst entfernt).

- [ ] **Step 4: Auflösung stagen**

Run (`cwd=.../rust`): `git add Cargo.toml`
Expected: kein Output.

---

## Task 4: Cargo.lock konsistent machen & Merge abschließen

**Files:**
- Auto-aktualisiert: `rust/Cargo.lock`

`Cargo.lock` wurde von git automatisch gemerged. Da `Cargo.toml` per `--theirs`+Re-Apply final feststeht, validiert ein Build-Lauf die Lock-Konsistenz und passt sie bei Bedarf an (alle neuen deps — `rushdown`, `tree-sitter-gdscript`, `wasmi`, `wat` — waren bereits in je einer der beiden Lock-Seiten vorhanden).

- [ ] **Step 1: Build zur Lock-Validierung**

Run (`cwd=.../rust`): `cargo build`
Expected: `Finished` ohne Fehler. Falls `Cargo.lock` dabei aktualisiert wird, ist das erwartet.

- [ ] **Step 2: Cargo.lock stagen (falls geändert)**

Run (`cwd=.../rust`): `git add Cargo.lock`
Expected: kein Output.

- [ ] **Step 3: Keine offenen Konflikte mehr**

Run (`cwd=.../`): `git diff --name-only --diff-filter=U`
Expected: **kein Output** (alle Konflikte aufgelöst).

- [ ] **Step 4: Merge-Commit erstellen**

Run (`cwd=.../`):
```
git commit --no-edit
```
Expected: Merge-Commit `Merge branch 'main' into feat-jetbrains-plugin` wird erstellt.

---

## Task 5: Verifikation — Tests grün

**Files:** keine Änderung; reine Verifikation.

- [ ] **Step 1: Volle Test-Suite**

Run (`cwd=.../rust`): `cargo nextest run --no-fail-fast --status-level fail`
Expected: `5079 tests run: 5079 passed ... 0 failed`. Insbesondere laufen `scenario_skip_terse_when_already_compressed` und `scenario_shell_compression_with_saved_tokens_skips_terse` jetzt grün (kamen via `d7e87825` von main).

- [ ] **Step 2: Bei Restfehlern — systematisch debuggen**

Falls wider Erwarten Tests fehlschlagen: NICHT raten. `superpowers:systematic-debugging` anwenden. Häufigste Ursache wäre ein neuer/anderer Test aus den 54 main-Commits, der lokale Voraussetzungen hat — Einzeltest mit `cargo nextest run -E 'test(<name>)' --no-capture` isolieren.

---

## Task 6: clippy — Status prüfen (optional, kosmetisch)

**Files:**
- Optional Modify: `rust/src/core/config/tests.rs`, `rust/src/server/execute.rs`

Vor dem Merge gab es 3 `pedantic`-Warnungen (2× `needless_raw_string_hashes` in `config/tests.rs:572,593`, 1× `duration_suboptimal_units` in `execute.rs:211`). Diese liegen in Dateien, die wir nicht geändert haben → existieren auch auf main; da main-CI grün ist, sind sie **nicht** CI-blockend. Cleanup ist optional.

- [ ] **Step 1: clippy nach Merge laufen lassen**

Run (`cwd=.../rust`): `cargo clippy --all-targets --all-features`
Expected: `Finished` — evtl. dieselben ≤3 pedantic-Warnungen, keine Errors.

- [ ] **Step 2 (nur falls Cleanup gewünscht): Warnungen beheben**

`config/tests.rs:572` & `:593` (`ctx_edit`): `r#"…"#` → `r"…"` (Hashes entfernen).
`execute.rs:211` — **Rust-Datei → Serena** (`mcp__serena__replace_content` o. symbolic edit), nicht `ctx_edit`: `std::time::Duration::from_millis(5000)` → `std::time::Duration::from_secs(5)`.

- [ ] **Step 3 (falls Step 2 ausgeführt): formatieren, verifizieren, committen**

`mcp__jetbrains__reformat_file` auf jede geänderte Datei, dann:
Run (`cwd=.../rust`): `cargo clippy --all-targets --all-features` → 0 Warnungen.
Run (`cwd=.../rust`): `cargo nextest run --status-level fail` → grün.
```
git add src/core/config/tests.rs src/server/execute.rs
git commit -m "style(clippy): silence pedantic lints (raw-string hashes, duration units)"
```

---

## Abschluss-Checkliste

- [ ] `git log --oneline -1` zeigt den Merge-Commit
- [ ] `git branch --contains d7e87825` enthält jetzt `feat-jetbrains-plugin`
- [ ] `cargo nextest run` → 5079/5079 grün
- [ ] JetBrains-Arbeit intakt: `ctx_tree packages/jetbrains-lean-ctx/src` unverändert vorhanden; `rust/Cargo.toml` zeigt `version = "3.7.5-jb"`, kein `rushdown` mehr; `cargo build` grün
- [ ] Backup-Branch `backup/pre-main-merge-2026-06-08` existiert noch (kann nach erfolgreicher Verifikation gelöscht werden: `git branch -D backup/pre-main-merge-2026-06-08`)
