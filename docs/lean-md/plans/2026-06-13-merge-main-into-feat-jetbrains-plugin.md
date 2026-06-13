# Merge `main` → `feat-jetbrains-plugin` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `main` (v3.8.3, +250 Commits) kontrolliert datei-für-datei in `feat-jetbrains-plugin` mergen, Konflikte gemäß Auflösungsmatrix lösen, alle Gates grün, Ergebnis-Version `3.8.3-jb`.

**Architecture:** Ansatz A — ein einziger Merge-Commit. `git merge main` materialisiert die Konflikte; jede Konfliktdatei wird einzeln nach Strategie aufgelöst (⚠-Dateien mit Diff + Freigabe vor `git add`), Rust-Edits ausschließlich via Serena-Tools. Gates (`cargo nextest`/`clippy`/`fmt`/Drift) fangen die semantischen Auto-Merge-Brüche ab. Der finale Release-Build bleibt manuell beim Nutzer.

**Tech Stack:** Rust (lean-ctx Core, `cargo nextest`/`clippy`/`fmt`), Kotlin (JetBrains-Plugin, kein Konflikt), Git-Merge, lean-ctx `ctx_*`-Tooling + Serena für `*.rs`.

---

## Vorbemerkung — Konfliktmarker statt Voll-Code

Dies ist ein Merge-Plan, kein Feature-Plan. Die exakten Hunks entstehen erst beim
`git merge main` (Task 2). Jeder Konflikt-Task nennt deshalb **Datei, Strategie,
konkrete Anker-Stelle und Verifikation** statt vollständigem Vorab-Code. Die
Konfliktmarker (`<<<<<<<`/`=======`/`>>>>>>>`) liest der Ausführende live mit
`ctx_read(path)`; aufgelöst wird die Datei so, dass **kein Marker** zurückbleibt.

**Harte Projektregeln (gelten in JEDEM Task):**
- `*.rs`-Edits **nur** via Serena (`mcp__serena__jet_brains_find_symbol`,
  `replace_symbol_body`, `replace_content`, …) — **nie** native `Edit`/`ctx_edit`.
- Vor jedem `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
- `ctx_shell`: bare command + `cwd=` (nie `cd … &&`), kein `2>&1`,
  Test-Runner ohne `| tail`/`| grep`.
- Tests immer `cargo nextest run`, nie `cargo test`.
- Keine Worktrees — direkt auf `feat-jetbrains-plugin`.

---

### Task 0: Pre-Merge-Hygiene — sauberer Arbeitsbaum

**Files:**
- Inspect: working tree status (`.idea/`, `markdownai/`, `.serena/`, neues Spec-Doc, `.claude/scheduled_tasks.lock`, `rust/.config/`)

- [ ] **Step 1: Branch + merge-base bestätigen**

```text
ctx_shell(command="git status --short --branch", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git merge-base HEAD main", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: Branch `feat-jetbrains-plugin`; merge-base `eed03440…`. Stimmt die
merge-base nicht → STOP, mit Nutzer klären (Divergenz hat sich verschoben).

- [ ] **Step 2: Untracked/uncommittete Dateien klären**

Die untracked Einträge (`.idea/`, `markdownai/`, `.serena/project.yml`,
`.claude/scheduled_tasks.lock`, `rust/.config/`, das neue Spec-Doc) dürfen den
Merge **nicht** blockieren. Entscheidung pro Gruppe:
- Tooling-/IDE-Artefakte (`.idea/`, `.serena/`, `*.lock`, `rust/.config/`,
  `markdownai/`) → **nicht** committen; vor dem Merge stashen **oder** über
  `.gitignore` ausschließen (siehe Task 7, dort wird `.gitignore` ohnehin
  angefasst). Bis dahin: `git stash --include-untracked` ist die sichere Variante.
- Spec-/Plan-Docs unter `docs/lean-md/` (diese Spec + dieser Plan) → **committen**,
  damit sie den Merge überleben.

```text
ctx_shell(command="git add docs/lean-md/specs/2026-06-13-merge-main-into-feat-jetbrains-plugin-design.md docs/lean-md/plans/2026-06-13-merge-main-into-feat-jetbrains-plugin.md", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git commit -m 'docs(merge): Spec + Plan für main→feat-jetbrains-plugin Merge'", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git stash push --include-untracked -m pre-merge-hygiene", cwd="/home/tholo/Scripts/lean-ctx")
```

- [ ] **Step 3: Sauberen Baum verifizieren**

```text
ctx_shell(command="git status --short", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: **leere** Ausgabe (working tree clean). Erst dann weiter.

---

### Task 1: main aktualisieren + Konflikt-Inventar verifizieren (Trockenlauf)

**Files:** keine — reiner Vorab-Check, dass das Inventar aus §1 der Spec noch stimmt.

- [ ] **Step 1: main-Ref aktuell holen**

```text
ctx_shell(command="git fetch origin main", cwd="/home/tholo/Scripts/lean-ctx")
```
(Falls kein Remote/offline: lokalen `main` verwenden — dann diesen Schritt überspringen.)

- [ ] **Step 2: Konflikt-Inventar trocken prüfen**

```text
ctx_shell(command="git merge-tree --write-tree HEAD main", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet (echte Inhalts-Konflikte): `rust/Cargo.toml`, `rust/Cargo.lock`,
`rust/src/shell/exec.rs`, `rust/src/tools/registered/ctx_refactor.rs`,
`rust/tests/intensive_benchmarks.rs`, sowie `.cursorrules`, `.gitignore`,
`AGENTS.md`, `docs/reference/README.md` und modify/delete für
`.claude/rules/lean-ctx.md`, `.claude/settings.local.json`,
`rust/.claude/settings.local.json`.

**Abgleich:** Weicht die Liste vom Spec-Inventar (§1) ab (neue Konfliktdatei oder
eine erwartete fehlt) → STOP, dem Nutzer den Delta melden, bevor gemergt wird.
`packages/jetbrains-lean-ctx/**` darf **nirgends** auftauchen (main fasst es nicht an).

---

### Task 2: `git merge main` — Konflikte materialisieren

**Files:** alle Konfliktdateien aus §1 (werden mit Markern befüllt).

- [ ] **Step 1: Merge starten (kein Auto-Commit)**

```text
ctx_shell(command="git merge --no-commit --no-ff main", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: `Automatic merge failed; fix conflicts and then commit the result.`
Der Merge bleibt offen (`MERGE_HEAD` gesetzt) — **nicht** abbrechen.

- [ ] **Step 2: Konflikt-Zustand erfassen**

```text
ctx_shell(command="git status --short", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: `UU` für Inhalts-Konflikte (Cargo.toml, Cargo.lock, exec.rs,
ctx_refactor.rs, intensive_benchmarks.rs, .cursorrules, .gitignore, AGENTS.md,
docs/reference/README.md), `DU` für die drei modify/delete-Dateien
(`.claude/rules/lean-ctx.md`, `.claude/settings.local.json`,
`rust/.claude/settings.local.json`).

- [ ] **Step 3: Inventar gegen die Matrix abhaken**

Liste die `UU`/`DU`-Dateien gegen §2 der Spec. Jede aufgelistete Datei bekommt
unten einen eigenen Task. Eine **unerwartete** Konfliktdatei → STOP + Nutzer
fragen, bevor sie angefasst wird.

**Kein Commit in diesem Task** — der Merge-Commit entsteht erst in Task 13.

---

### Task 3: `rust/Cargo.toml` — main übernehmen, Version `3.8.3-jb`

**Files:**
- Resolve: `rust/Cargo.toml` (aktuell `version = "3.7.5-jb"` auf Zeile 3)

**Strategie (§2):** main komplett übernehmen (bringt `base64`/`hkdf`/
`chacha20poly1305`/`gethostname` + dev-dep `jsonschema` + example `locomo_bench`);
Branch hat **keine** eigenen dep-Adds. Einziger Branch-Override: das `-jb`-Suffix
in der Version.

- [ ] **Step 1: Konflikt-Hunks lesen**

```text
ctx_read("rust/Cargo.toml")
```
Erwartung: Konfliktmarker stehen um `version` und um den `[dependencies]`-Block.

- [ ] **Step 2: main-Seite checkout-en, dann Version patchen**

`Cargo.toml` ist kein `*.rs` → native Tools erlaubt. main-Version komplett nehmen:

```text
ctx_shell(command="git checkout --theirs rust/Cargo.toml", cwd="/home/tholo/Scripts/lean-ctx")
```
Dann die main-Version (`version = "3.8.3"`) auf `3.8.3-jb` setzen:

```text
ctx_edit("rust/Cargo.toml", "version = \"3.8.3\"", "version = \"3.8.3-jb\"")
```

- [ ] **Step 3: Verifizieren — keine Marker, korrekte Version**

```text
ctx_read("rust/Cargo.toml", mode="lines:1-20")
ctx_search("<<<<<<<|=======|>>>>>>>", "rust/Cargo.toml")
```
Erwartet: `version = "3.8.3-jb"` auf Zeile 3; main's neue deps (`base64`,
`hkdf`, `chacha20poly1305`, `gethostname`, dev-dep `jsonschema`) vorhanden;
**null** Konfliktmarker.

- [ ] **Step 4: Reformat + stage**

```text
mcp__jetbrains__reformat_file (rust/Cargo.toml)
ctx_shell(command="git add rust/Cargo.toml", cwd="/home/tholo/Scripts/lean-ctx")
```

---

### Task 4: `rust/Cargo.lock` — main's Lock als Basis

**Files:**
- Resolve: `rust/Cargo.lock` (Derivat, wird in Task 11 regeneriert)

**Strategie (§2):** Lockfile ist Derivat → main's Version nehmen; die echte
Regeneration auf `3.8.3-jb` macht der erste `cargo`-Lauf (Task 11).

- [ ] **Step 1: main-Seite übernehmen**

```text
ctx_shell(command="git checkout --theirs rust/Cargo.lock", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git add rust/Cargo.lock", cwd="/home/tholo/Scripts/lean-ctx")
```

- [ ] **Step 2: Keine Marker bestätigen**

```text
ctx_search("<<<<<<<", "rust/Cargo.lock")
```
Erwartet: keine Treffer. (Version-Mismatch `3.8.3` vs `3.8.3-jb` im Lock ist ok —
Task 11 regeneriert ihn.)

---

### Task 5: ⚠ `rust/src/tools/registered/ctx_refactor.rs` — Branch-Logik + `shell_outcome: None`

**Files:**
- Resolve: `rust/src/tools/registered/ctx_refactor.rs` (`ToolOutput`-Konstruktor ab Zeile 95)

**Strategie (§2):** Branch-Logik vollständig behalten (v2c Two-Phase-Stack:
rename/move/safe_delete preview+apply, name_path-Edits, 18 Actions, Schema-Felder)
**+** main's neues `ToolOutput`-Feld `shell_outcome: None` im Konstruktor ergänzen.
Grund: das auto-gemergte `tool_trait.rs` (Task 9) gibt `ToolOutput` ein neues Feld
`shell_outcome` (#499) — **jeder** `ToolOutput { … }`-Konstruktor muss es setzen,
sonst `missing field`-Compile-Fehler.

⚠ **Diese Datei einzeln mit Diff + Freigabe vor `git add`.**

- [ ] **Step 1: Konflikt lesen + Diff vorlegen**

```text
ctx_read("rust/src/tools/registered/ctx_refactor.rs")
```
Dem Nutzer die Konflikt-Hunks zeigen; die Branch-Seite ist die zu behaltende
(v2c-Actions). main's Seite trägt nur das additive `shell_outcome`-Feld (#499).

- [ ] **Step 2: Branch-Seite als Basis wählen (Serena)**

`*.rs` → **nur Serena**. Konfliktmarker so auflösen, dass die **Branch-Seite**
des Konstruktors steht (Felder `text/original_tokens/saved_tokens/mode/path/changed`
wie Zeile 95-110). Via `mcp__serena__replace_content` den markierten Bereich durch
die reine Branch-Variante ersetzen.

- [ ] **Step 3: `shell_outcome: None` ergänzen (Serena)**

Im `ToolOutput { … }`-Literal (nach `changed: …,`, vor dem schließenden `}` auf
Zeile 109-110) das Feld einfügen:

```rust
            shell_outcome: None,
```
Via `mcp__serena__replace_symbol_body` (Methode, die den Konstruktor enthält) oder
`replace_content` auf den exakten Block. Ergebnis:

```rust
        Ok(ToolOutput {
            text: result,
            original_tokens: 0,
            saved_tokens: 0,
            mode: Some(action.clone()),
            path: get_str(args, "path"),
            changed: matches!(
                action.as_str(),
                "replace_symbol_body"
                    | "insert_before_symbol"
                    | "insert_after_symbol"
                    | "rename_apply"
                    | "move_apply"
                    | "safe_delete_apply"
            ),
            shell_outcome: None,
        })
```

- [ ] **Step 4: Marker weg, alle 18 Actions intakt**

```text
ctx_read("rust/src/tools/registered/ctx_refactor.rs", mode="diff")
ctx_search("<<<<<<<|=======|>>>>>>>", "rust/src/tools/registered/ctx_refactor.rs")
```
Erwartet: keine Marker; `changed: matches!(…)` enthält weiterhin alle apply-Actions;
`shell_outcome: None` gesetzt. (Compile-Verifikation folgt im Gate Task 10.)

- [ ] **Step 5: Freigabe einholen, dann reformat + stage**

Diff dem Nutzer vorlegen → nach Freigabe:

```text
mcp__jetbrains__reformat_file (rust/src/tools/registered/ctx_refactor.rs)
ctx_shell(command="git add rust/src/tools/registered/ctx_refactor.rs", cwd="/home/tholo/Scripts/lean-ctx")
```

---

### Task 6: ⚠ `rust/src/shell/exec.rs` — beide Seiten vereinen

**Files:**
- Resolve: `rust/src/shell/exec.rs` (heavy commands ~Zeile 122/168, `exec()`-Einstieg, `mod exec_tests` ~Zeile 600+)

**Strategie (§2 + §3):** **Disjunkte** Änderungen beider Seiten behalten:
- **main:** `allowlist_must_enforce` / `allowlist_must_enforce_inner` (#413 — CLI-`-c`-
  Pfad erzwingt Allowlist für Agents: exit 126 in hook-child mode / non-TTY-stderr,
  warn-only nur interaktiv oder bei `LEAN_CTX_ALLOWLIST_WARN_ONLY=1`) **+**
  `diagnostics_store::record_from_shell` (#499).
- **Branch:** `cargo nextest` als heavy command (Zeile 122) + `heavy_timeout()`
  (Zeile 168) inkl. dessen Tests (`heavy_timeout_some_for_heavy_none_otherwise`,
  Zeile 631).

Beides liegt disjunkt → beide Seiten zusammenführen, **keine** verwerfen.
`exec()`-Einstieg und `mod exec_tests` sauber vereinen.

⚠ **Diese Datei einzeln mit Diff + Freigabe vor `git add`.**

- [ ] **Step 1: Konflikt lesen, Hunks klassifizieren**

```text
ctx_read("rust/src/shell/exec.rs")
```
Jeden Konflikt-Hunk einer Seite zuordnen: heavy-command/timeout-Hunks = Branch
behalten; allowlist-enforce + record_from_shell-Hunks = main behalten. Überlappen
sie in derselben Region (z. B. `exec()`-Body), beide Logiken **verschränken**.

- [ ] **Step 2: Vereinigen (Serena)**

`*.rs` → **nur Serena**. Pro Hunk via `replace_content` den Marker-Block durch die
vereinigte Variante ersetzen. Sicherstellen, dass:
- `heavy_timeout()` (Branch) **und** `allowlist_must_enforce*` (main) beide als
  Funktionen existieren,
- `cargo nextest` (Zeile 122) in der heavy-command-Liste bleibt,
- main's `record_from_shell`-Aufruf im erfolg-/fehlschlag-Pfad erhalten ist,
- `mod exec_tests` **beide** Test-Sätze enthält (Branch: heavy_timeout-Tests;
  main: allowlist-enforce-Tests).

- [ ] **Step 3: Marker weg, beide Feature-Sätze präsent**

```text
ctx_read("rust/src/shell/exec.rs", mode="diff")
ctx_search("<<<<<<<|=======|>>>>>>>", "rust/src/shell/exec.rs")
ctx_search("heavy_timeout|allowlist_must_enforce|record_from_shell|cargo nextest", "rust/src/shell/exec.rs")
```
Erwartet: keine Marker; alle vier Symbole vorhanden. Compile/Test-Verifikation im Gate (Task 10).

- [ ] **Step 4: Freigabe, reformat, stage**

```text
mcp__jetbrains__reformat_file (rust/src/shell/exec.rs)
ctx_shell(command="git add rust/src/shell/exec.rs", cwd="/home/tholo/Scripts/lean-ctx")
```

---

### Task 7: ⚠ `rust/tests/intensive_benchmarks.rs` — main's `< 12000` nehmen

**Files:**
- Resolve: `rust/tests/intensive_benchmarks.rs` (`bench_total_input_overhead`, assert ~Zeile 190)

**Strategie (§2):** Beide Seiten ändern dieselbe assert-Zeile in
`bench_total_input_overhead` (Branch: `11000 → 11600`, main: `11000 → 12000`).
**main's `total < 12000` gewinnt** (höher + #576-dietisiert → Headroom), Branch's
`11600` verwerfen. Übrige main-Hunks (`instruction_decoder_block(false)`,
#579-Cues, `essential_instructions`-Keywords) liegen disjunkt → von main auto-mergen.

⚠ **Diese Datei einzeln mit Diff + Freigabe vor `git add`.**

- [ ] **Step 1: Konflikt lesen**

```text
ctx_read("rust/tests/intensive_benchmarks.rs", mode="lines:180-195")
ctx_search("<<<<<<<", "rust/tests/intensive_benchmarks.rs")
```

- [ ] **Step 2: assert auf main's Wert (Serena)**

`*.rs` → **nur Serena**. Den Konflikt-Hunk so auflösen, dass die assert lautet:

```rust
    assert!(
        total < 12000,
        "Total input overhead should be <12000 tokens, got {total}"
    );
```
Den Branch-spezifischen Kommentar (`Threshold raised 11000 → 11600 …`) durch main's
Kommentar ersetzen bzw. an `12000` angleichen. Übrige Konflikt-Hunks (falls
vorhanden) auf die **main**-Seite auflösen.

- [ ] **Step 3: Marker weg, Wert korrekt**

```text
ctx_search("<<<<<<<|11600|12000", "rust/tests/intensive_benchmarks.rs")
```
Erwartet: kein Marker, kein `11600`; genau ein `< 12000`.

- [ ] **Step 4: Freigabe, reformat, stage**

```text
mcp__jetbrains__reformat_file (rust/tests/intensive_benchmarks.rs)
ctx_shell(command="git add rust/tests/intensive_benchmarks.rs", cwd="/home/tholo/Scripts/lean-ctx")
```

> Hinweis: Ob die assert real grün ist (`ctx_refactor`-Bloat ↔ #576), misst
> Task 12. Hier nur die Konfliktzeile auflösen.

---

### Task 8: ⚠ Doku/Config-Konflikte — `.cursorrules`, `.gitignore`, `AGENTS.md`, `docs/reference/README.md`

**Files:**
- Resolve: `.cursorrules`, `.gitignore`, `AGENTS.md`, `docs/reference/README.md`

**Strategie (§2):** main als Basis, branch-spezifische Zeilen (JetBrains / v2c)
**wieder einfügen**. Keine `*.rs` → native Tools / `ctx_edit` erlaubt.

⚠ **Jede Datei einzeln mit Diff + Freigabe vor `git add`.**

- [ ] **Step 1: Pro Datei Konflikt lesen + Branch-Zeilen identifizieren**

```text
ctx_read(".cursorrules")
ctx_read(".gitignore")
ctx_read("AGENTS.md")
ctx_read("docs/reference/README.md")
```
Branch-eigene Zeilen erkennen: JetBrains-Plugin-Erwähnungen, v2c-Refactor-Hinweise,
lean-ctx-Tool-Discipline-Bezüge. `.gitignore`: hier auch die Task-0-Artefakte
(`.idea/`, `.serena/`, `markdownai/`, `*.lock`, `rust/.config/`) ergänzen, falls
nicht schon von main abgedeckt.

- [ ] **Step 2: main-Basis + Branch-Zeilen re-add**

Pro Datei: main-Seite als Grundgerüst, dann die identifizierten Branch-Zeilen
wieder einsetzen (via `ctx_edit` / native Edit). Keine Konfliktmarker zurücklassen.

- [ ] **Step 3: Marker-Scan über alle vier**

```text
ctx_search("<<<<<<<", ".cursorrules")
ctx_search("<<<<<<<", ".gitignore")
ctx_search("<<<<<<<", "AGENTS.md")
ctx_search("<<<<<<<", "docs/reference/README.md")
```
Erwartet: jeweils keine Treffer.

- [ ] **Step 4: Freigabe, reformat, stage (Datei für Datei)**

```text
mcp__jetbrains__reformat_file (je Datei)
ctx_shell(command="git add .cursorrules .gitignore AGENTS.md docs/reference/README.md", cwd="/home/tholo/Scripts/lean-ctx")
```

---

### Task 9: ⚠ modify/delete re-add — `.claude/rules/lean-ctx.md`, `.claude/settings.local.json`, `rust/.claude/settings.local.json`

**Files:**
- Resolve (DU): `.claude/rules/lean-ctx.md`, `.claude/settings.local.json`, `rust/.claude/settings.local.json`

**Strategie (§2, festgeschrieben):** main hat diese Dateien via rules_injection
„dedicated" (#343) **gelöscht**; der Branch ändert sie und nutzt sie aktiv für die
lean-ctx Tool-Discipline → **Branch-Versionen behalten (re-add)**.

⚠ **Einzeln mit Diff + Freigabe.**

- [ ] **Step 1: Branch-Version je Datei behalten**

Bei modify/delete heißt „Branch behalten" = `--ours` (unsere Seite ist der Branch):

```text
ctx_shell(command="git checkout --ours .claude/rules/lean-ctx.md .claude/settings.local.json rust/.claude/settings.local.json", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git add .claude/rules/lean-ctx.md .claude/settings.local.json rust/.claude/settings.local.json", cwd="/home/tholo/Scripts/lean-ctx")
```

- [ ] **Step 2: Re-add bestätigen**

```text
ctx_shell(command="git status --short", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: die drei Dateien nicht mehr `DU`, sondern als `M`/`A` gestaged; Inhalt =
Branch-Version (lean-ctx-Regeln vorhanden).

---

### Task 10: Auto-merge Smoke-Review (keine Freigabe, nur Sichtprüfung)

**Files (kein Konflikt, auto-gemergt):**
- `rust/src/cli/dispatch/mod.rs`, `rust/src/cli/mod.rs`,
  `rust/src/core/config/{mod,tests}.rs`, `rust/src/core/graph_index/{mod,tests}.rs`,
  `rust/src/server/tool_trait.rs`, `docs/reference/appendix-mcp-tools.md`,
  `docs/reference/generated/mcp-tools.md`, `rust/LOCK_ORDERING.md`

**Ziel:** Bestätigen, dass die in §3 markierten semantischen Auto-Merge-Risiken
sauber sind. Der harte Fang bleibt das Gate (Task 11) — hier nur Sichtprüfung.

- [ ] **Step 1: `tool_trait.rs` — `shell_outcome`-Feld + `impl Default` koexistieren**

```text
ctx_read("rust/src/server/tool_trait.rs", mode="signatures")
ctx_search("shell_outcome|impl Default for ToolContext|ShellOutcome|get_usize", "rust/src/server/tool_trait.rs")
```
Erwartet: main's `ShellOutcome`/`shell_outcome`/`get_usize` **und** Branch's
`impl Default for ToolContext` beide vorhanden, kein `missing field`. Bestätigt das
neue Feld, das Task 5 in `ctx_refactor.rs` setzen musste.

- [ ] **Step 2: API-Umbau-Risiken aus §3 prüfen (Treffer dürfen nur in main-Dateien liegen)**

```text
ctx_search("ResolverContext::new|keep_entry", "rust/src")
```
Erwartet: alte Aufruf-Form (`ResolverContext::new` ohne `content_cache`,
`cloud_files::keep_entry`) taucht **nicht** in branch-geänderten Dateien auf.
`graph_index/mod.rs`: nur `get_forward_deps` ist Branch-Zutat — main's
`scan_inner`-Umbau liegt disjunkt.

- [ ] **Step 3: Sichtprüfung der restlichen Auto-Merges**

```text
ctx_read("rust/src/core/graph_index/mod.rs", mode="diff")
ctx_read("rust/src/cli/dispatch/mod.rs", mode="diff")
```
Erwartet: plausible Vereinigung, keine Konfliktmarker. Auffälligkeiten → notieren,
im Gate (Task 11) fällt ein echter Bruch als Compile-Fehler auf.

**Kein `git add` nötig** — diese Dateien sind bereits sauber im Merge-Index.

---

### Task 11: Gate-Lauf 1 — Cargo.lock regenerieren, Compile + Tests + clippy + fmt

**Files:** keine Edits (außer Cargo.lock-Regeneration als Nebeneffekt) — reine Verifikation.

**Ziel:** Der harte Fang für die semantischen Auto-Merge-Brüche aus §3. Ein
textuell sauberer Merge, der eine umgebaute API in alter Form aufruft, schlägt hier
als Compile-/Clippy-Fehler fehl.

- [ ] **Step 1: Cargo.lock auf `3.8.3-jb` regenerieren**

```text
ctx_shell(command="cargo update -p lean-ctx --precise 3.8.3-jb", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Falls das fehlschlägt (precise greift nur bei Registry-Versionen): stattdessen ein
einfacher Build löst den Lock auf — weiter mit Step 2; danach Lock prüfen.

- [ ] **Step 2: Compile**

```text
ctx_shell(command="cargo build", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: `Finished`. Compile-Fehler hier = ein §3-Auto-Merge-Bruch (fehlendes
`shell_outcome`-Feld, alte API-Form, …) → zur betroffenen Datei zurück, fixen
(Serena), erneut bauen. **Nicht** weiter, solange rot.

- [ ] **Step 3: Tests**

```text
ctx_shell(command="cargo nextest run", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: alle grün (Referenz: ~5294 Rust-Tests aus Session-Historie). Rote Tests
→ systematisch debuggen (superpowers:systematic-debugging), Ursache der
Konfliktauflösung zuordnen, fixen, erneut laufen.

- [ ] **Step 4: clippy**

```text
ctx_shell(command="cargo clippy --all-targets", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: keine Warnungen/Fehler.

- [ ] **Step 5: fmt-Check**

```text
ctx_shell(command="cargo fmt --check", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: keine Ausgabe (alles formatiert). Bei Abweichung: `cargo fmt`, dann die
betroffene Datei erneut stagen.

- [ ] **Step 6: Cargo.lock-Änderung stagen**

```text
ctx_shell(command="git add rust/Cargo.lock", cwd="/home/tholo/Scripts/lean-ctx")
```

---

### Task 12: Gate-Lauf 2 — Schema-diet-Messung (§3a/§3b) + Drift-Tests

**Files (bedingt, nur falls Budget gesprengt):**
- `rust/src/tools/registered/ctx_refactor.rs` (`tool_def()`-Description)
- `docs/reference/appendix-mcp-tools.md`, `docs/reference/generated/mcp-tools.md`

**Ziel:** §3a/§3b-Invariante absichern — `ctx_refactor` bleibt funktional voll, die
Full-Surface-Budgets halten, Repo-Doku-Spiegel ist driftfrei.

- [ ] **Step 1: Full-Surface-Overhead messen**

```text
ctx_shell(command="cargo nextest run -E 'test(bench_total_input_overhead) + test(bench_tool_descriptions_token_count)'", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: `bench_total_input_overhead` grün (`total < 12000`),
`bench_tool_descriptions_token_count` grün (`< 2600` total, jedes Nicht-`ctx_call`-
Tool `< 160`). Den real gemessenen `total`-Wert aus dem Test-Output notieren.

- [ ] **Step 2: Entscheidung Schema-diet-Angleichung (§3a)**

- `total` **mit Headroom < 12000** (z. B. ≲ 11000) → Angleichung ist **empfohlener
  Follow-up**, nicht blockierend. Step 3 überspringen, weiter zu Step 4.
- `total` **nahe/über 12000** → **Pflicht**: `ctx_refactor`-Schema an #576 angleichen
  (Step 3).

- [ ] **Step 3 (bedingt): `ctx_refactor`-Schema an ctx_graph-Idiom angleichen (Serena)**

**Invariante (§3b — Funktionserhalt, kein Informationsverlust):** Die 18-Action-
Liste wandert vom `"enum": [...]`-Array in die `"description"` als **pipe-delimited
Text** (Vorbild `rust/src/tools/registered/ctx_graph.rs`). **Keine** Action, **kein**
Param gestrichen. Vollständige Action-Liste, die in der Description stehen MUSS:

```text
rename|rename_preview|rename_apply|move_preview|move_apply|safe_delete_preview|safe_delete_apply|references|definition|implementations|declaration|type_hierarchy|symbols_overview|inspections|replace_symbol_body|insert_before_symbol|insert_after_symbol
```
`*.rs` → **nur Serena**. In `ctx_refactor`s `tool_def()`:
- `action`-Property: `enum`-Array entfernen, Actions als pipe-delimited String in
  `description` (`"type": "string", "description": "rename|rename_preview|…"`).
- Lange Tool-Description + 14 Property-Descriptions kürzen (am #576-Idiom).
- `ctx_refactor` bleibt in `build_registry()` registriert
  (`rust/src/tools/registry.rs:139`) — Sichtbarkeit ≠ Verfügbarkeit.

Danach Step 1 erneut → `total < 12000` mit Headroom bestätigen.

- [ ] **Step 4: Drift-/Conformance-Tests + Repo-Doku-Spiegel (§3b Punkt 2)**

```text
ctx_shell(command="cargo nextest run -E 'test(drift) + test(conformance) + test(mcp_tools) + test(appendix)'", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Schlägt ein Drift-Test fehl (appendix/generated nicht synchron) → `gen_docs`
regenerieren:

```text
ctx_shell(command="cargo run --bin gen_docs", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
(Exakten Bin-Namen vorher prüfen: `ctx_search("gen_docs|gen_mcp_manifest", "rust/src/bin")`
bzw. `ctx_tree("rust/src/bin", 1)`.) Danach Drift-Test erneut → grün.

- [ ] **Step 5: Funktionserhalt-Smoke (§3b Punkt 3) — Binary-Quelle liefert volle Action-Liste**

```text
cargo run -- (oder gebautes Binary) discover_tools "refactor"
ctx_call name=ctx_refactor   # bzw. der projektspezifische Discovery-Pfad
```
Erwartet: beide listen die **volle** v2c-Action-Liste (alle 18). Bestätigt den
Neuinstallations-Fall (Info steckt im Binary, nicht nur in Repo-Docs). Praktikabel
auch als Test, falls vorhanden:
```text
ctx_shell(command="cargo nextest run -E 'test(refactor) and test(action)'", cwd="/home/tholo/Scripts/lean-ctx/rust")
```

- [ ] **Step 6: Geänderte Doku/Schema-Dateien stagen**

```text
mcp__jetbrains__reformat_file (jede in Step 3/4 geänderte Datei)
ctx_shell(command="git add rust/src/tools/registered/ctx_refactor.rs docs/reference/appendix-mcp-tools.md docs/reference/generated/mcp-tools.md", cwd="/home/tholo/Scripts/lean-ctx")
```
(Nur die tatsächlich geänderten Pfade adden.)

---

### Task 13: Merge-Commit + Endverifikation + Build-Befehl-Doku

**Files:** keine — Abschluss.

**Ziel:** Den offenen Merge mit allen aufgelösten Konflikten committen, Endzustand
verifizieren, dem Nutzer den manuellen Release-Build-Befehl liefern (§4 Schritt 9).

- [ ] **Step 1: Vollständigkeit prüfen — kein Konflikt offen, kein Marker im Baum**

```text
ctx_shell(command="git status --short", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git diff --cached --check", cwd="/home/tholo/Scripts/lean-ctx")
```
Erwartet: keine `UU`/`DU`-Einträge mehr; `--check` meldet keine Konfliktmarker.
Alle Konfliktdateien gestaged.

- [ ] **Step 2: Merge-Commit anlegen**

```text
ctx_shell(command="git commit -m 'merge(main): main v3.8.3 → feat-jetbrains-plugin (3.8.3-jb)\n\n- Konflikte gemäß Auflösungsmatrix §2 gelöst (ctx_refactor/exec/benchmarks/Cargo + Doku/Config)\n- main-Verbesserungen #413 (allowlist enforce) + #499 (record_from_shell/shell_outcome) übernommen\n- v2c Two-Phase-Stack (JetBrains-Plugin) unverändert erhalten\n- Schema-diet/Lean-Surface (#575/#576) eingehalten, ctx_refactor funktional voll\n- version = 3.8.3-jb'", cwd="/home/tholo/Scripts/lean-ctx")
```

- [ ] **Step 3: Endverifikation — Gates auf dem Merge-Commit**

```text
ctx_shell(command="cargo nextest run", cwd="/home/tholo/Scripts/lean-ctx/rust")
ctx_shell(command="cargo clippy --all-targets", cwd="/home/tholo/Scripts/lean-ctx/rust")
ctx_shell(command="cargo fmt --check", cwd="/home/tholo/Scripts/lean-ctx/rust")
```
Erwartet: alle grün. Erfolgskriterien (§5) abhaken: Merge-Commit da, `3.8.3-jb`
gesetzt, Gates grün, #413/#499 drin, v2c erhalten, Schema-diet-Invariante gehalten.

- [ ] **Step 4: gestashte Hygiene-Artefakte zurückholen (optional)**

```text
ctx_shell(command="git stash list", cwd="/home/tholo/Scripts/lean-ctx")
ctx_shell(command="git stash pop", cwd="/home/tholo/Scripts/lean-ctx")
```
(Nur falls Task 0 Step 2 gestasht hat und die Artefakte zurück sollen. Konflikte
beim Pop = IDE-Artefakte → unkritisch, ggf. verwerfen.)

- [ ] **Step 5: Manuellen Release-Build-Befehl an den Nutzer liefern (NICHT ausführen)**

Dem Nutzer exakt diese Befehle nennen — der Agent führt sie **nicht** aus (§6):

```bash
# Release-Binary 3.8.3-jb bauen:
cargo build --release --manifest-path rust/Cargo.toml

# oder systemweit installieren:
cargo install --path rust
```

- [ ] **Step 6 (optional): Post-Build-Smoke-Check ansagen (§4 Schritt 10)**

Dem Nutzer als optionalen Check nach seinem Build nennen (kein Merge-Gate):

```bash
lean-ctx doctor --migrate-check
```
Erwartet (laut §4): Exit 0 — der Branch führt keine neuen `config.toml`-Keys ein.

---

## Self-Review

**Spec-Coverage:**
- §1 Pre-Merge-Hygiene / merge-base → Task 0, 1
- §2 Echte Code-Konflikte (ctx_refactor, exec, benchmarks, Cargo.toml/lock) → Task 3–7
- §2 Doku/Config + modify/delete → Task 8, 9
- §2 Auto-merge Smoke-Review → Task 10
- §3 Verbesserungs-Adoption (#413/#499) → Task 6 (exec.rs) + Task 10 (tool_trait)
- §3 semantische Auto-Merge-Brüche (ResolverContext/keep_entry) → Task 10 Step 2 + Task 11 (Compile-Fang)
- §3a/§3b Schema-diet + Funktionserhalt → Task 7 (assert) + Task 12 (Messung/Angleichung/Smoke)
- §3c PathJail → keine Code-Aktion (signatur-stabil); abgedeckt durch Task 11 (v2c-move/rename-Tests grün)
- §4 Gate-/Build-Sequenz → Task 11, 12, 13
- §5 Erfolgskriterien → Task 13 Step 3
- §6 Nicht-Ziele (kein Build durch Agent, kein v1.0-Launch) → Task 13 Step 5 (nur Befehl liefern)

**Typkonsistenz:** `shell_outcome: None` (Task 5) ↔ `ToolOutput`-Feld aus
`tool_trait.rs` (Task 10 Step 1) — Feldname konsistent. `total < 12000`
(Task 7 + Task 12) durchgängig. Action-Liste (18 Actions) in Task 5 (`changed`)
und Task 12 (Description) identisch.

**Platzhalter-Scan:** Keine TBD/TODO; bedingte Schritte (Task 11 Step 1, Task 12
Step 2/3) haben explizite Wenn-dann-Kriterien statt offener „handle edge cases".
