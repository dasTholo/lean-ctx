# Design: Merge `main` → `feat-jetbrains-plugin`

**Datum:** 2026-06-13
**Branch:** `feat-jetbrains-plugin`
**Ziel-Version nach Merge:** `3.8.3-jb`
**Ansatz:** A — Merge-Commit mit kontrollierter, datei-für-datei freigegebener Konfliktauflösung

---

## 1. Ausgangslage

`main` hat eine Version bekommen und ist stark divergiert. Statt eines blinden
Merges werden die Änderungen gezielt verglichen, pro Konfliktdatei einzeln
abgesprochen und erst danach gemergt. Abschluss: lokaler Release-Build der
Version `3.8.3-jb` (**manuell durch den Nutzer**, nicht durch den Agenten).

### Divergenz (Stand 2026-06-13)

| Kennzahl | Wert |
|---|---|
| merge-base | `eed03440` |
| `main` voraus | +250 Commits |
| `feat-jetbrains-plugin` voraus | +306 Commits |
| Version `main` | `3.8.3` |
| Version Branch | `3.7.5-jb` (geforkt von 3.7.5) |

### Konflikt-Inventar (`git merge-tree main feat-jetbrains-plugin`)

Echte Inhalts-Konflikte (Code + Manifest):
`rust/Cargo.toml`, `rust/Cargo.lock`, `rust/src/shell/exec.rs`,
`rust/src/tools/registered/ctx_refactor.rs`, `rust/tests/intensive_benchmarks.rs`.

Inhalts-Konflikte (Doku/Config):
`.cursorrules`, `.gitignore`, `AGENTS.md`, `docs/reference/README.md`.

modify/delete (in `main` gelöscht, im Branch geändert):
`.claude/rules/lean-ctx.md`, `.claude/settings.local.json`,
`rust/.claude/settings.local.json`.

Auto-merge sauber (kein Konflikt, nur Smoke-Review):
`rust/src/cli/dispatch/mod.rs`, `rust/src/cli/mod.rs`,
`rust/src/core/config/{mod,tests}.rs`, `rust/src/core/graph_index/{mod,tests}.rs`,
`rust/src/server/tool_trait.rs`, `docs/reference/appendix-mcp-tools.md`,
`docs/reference/generated/mcp-tools.md`, `rust/LOCK_ORDERING.md`.

**Wichtig:** `main` hat das JetBrains-Plugin (`packages/jetbrains-lean-ctx`,
Kotlin) **nicht** angefasst → dort null Konflikt.

---

## 2. Konflikt-Auflösungsmatrix

Jede Datei hat eine Default-Strategie. ⚠-Dateien werden bei der Ausführung
**einzeln mit Diff + Begründung vor dem Anwenden** freigegeben.

### Echte Code-Konflikte

| Datei | Strategie | Begründung |
|---|---|---|
| `ctx_refactor.rs` ⚠ | Branch-Logik behalten **+** main's `shell_outcome: None` im `ToolOutput`-Konstruktor ergänzen | Branch = Feature-Kern (v2c Two-Phase-Stack: rename/move/safe_delete preview+apply, name_path-Edits, neue Schema-Felder); main nur additives Feld (#499). `ToolOutput` erhält via auto-gemergtem `tool_trait.rs` das neue Feld → jeder Konstruktor muss es setzen. |
| `exec.rs` ⚠ | **Beide** Seiten vereinen | main: `allowlist_must_enforce`/`allowlist_must_enforce_inner` (#413, CLI-`-c`-Pfad erzwingt Allowlist für Agents statt nur zu warnen) + `diagnostics_store::record_from_shell` (#499). Branch: `cargo nextest` als heavy command + `heavy_timeout()`. Disjunkt → beide behalten; Test-Blöcke (`mod exec_tests`) und `exec()`-Einstieg sauber zusammenführen. |
| `intensive_benchmarks.rs` ⚠ | Inspektion bei Ausführung | Test-Datei; vermutlich beidseitige Append-Konflikte. |
| `Cargo.toml` | main komplett übernehmen, nur `version = "3.8.3-jb"` setzen | Branch hat **keine** eigenen dep-Adds; main bringt `base64`/`hkdf`/`chacha20poly1305`/`gethostname` + dev-dep `jsonschema` + example `locomo_bench`. |
| `Cargo.lock` | main's Lock als Basis; wird beim ersten Gate-`cargo`-Lauf auf `3.8.3-jb` regeneriert | Lockfile ist Derivat. |

### Doku/Config

| Datei | Strategie |
|---|---|
| `.cursorrules`, `.gitignore`, `AGENTS.md`, `docs/reference/README.md` ⚠ | main als Basis, branch-spezifische Zeilen (JetBrains/v2c) wieder einfügen |
| `.claude/rules/lean-ctx.md`, `.claude/settings.local.json`, `rust/.claude/settings.local.json` ⚠ | **Entscheidung (festgeschrieben):** Branch-Versionen **behalten** (re-add). main hat sie via rules_injection „dedicated" (#343) gelöscht; der Branch nutzt sie aktiv für die lean-ctx Tool-Discipline. |

### Auto-merge (nur Smoke-Review, keine Freigabe)

`cli/dispatch/mod.rs`, `cli/mod.rs`, `core/config/{mod,tests}.rs`,
`core/graph_index/{mod,tests}.rs`, `server/tool_trait.rs`,
`docs/reference/{appendix,generated}-mcp-tools.md`, `LOCK_ORDERING.md`.

---

## 3. Verbesserungs-Adoption (main-Änderungen, die den Branch-Code verbessern)

### Direkt branch-relevant — Pflicht-Adoption (fließt in die `exec.rs`-Vereinigung)

- **Allowlist enforce statt warn (#413):** der CLI-`-c`-Pfad (Shell-Hook für
  Agents) erzwingt jetzt dieselbe Allowlist-Grenze wie `ctx_shell` (exit 126
  statt nur `tracing::warn!`). Geblockt wird in hook-child mode
  (`LEAN_CTX_HOOK_CHILD`) und wenn stderr kein TTY ist (Agent/Script/Pipe);
  warn-only bleibt für interaktive Menschen oder bei
  `LEAN_CTX_ALLOWLIST_WARN_ONLY=1`.
- **`diagnostics_store::record_from_shell` (#499):** fehlschlagende
  cargo/tsc/eslint-Läufe markieren ihre Dateien als context-priority,
  erfolgreiche Läufe räumen die Markierung. Neues Subsystem, das der Branch
  danach mitnutzt.
- **byte-faithful redirect + `ctx_read`/terse-skip-Korrektheit:** kommen über
  main automatisch mit; relevant, da der Branch intensiv über
  `ctx_read`/`ctx_shell` arbeitet.

### Additiv, kein Branch-Impact (nur zur Kenntnis, keine Aktion)

Context OS 12.x (Personas, Extension-Registry, WASM-Runtime, Python/TS/Rust
SDKs, generische Ingestion + Extractors), Commercial Plane 13.x (Team-RBAC,
Billing), `/v1/capabilities` + OpenAPI-Discovery.

### Achtungspunkt — Tool-Surface / Drift

`main` wuchs auf Tool-Count 72 und führte Capabilities/OpenAPI-Drift-Tests ein.
Der Branch erweitert `ctx_refactor` massiv (neue Actions, **kein** neues Tool).
Nach dem Merge müssen `docs/reference/appendix-mcp-tools.md` und
`docs/reference/generated/mcp-tools.md` konsistent sein → ggf.
`gen_docs`/`gen_mcp_manifest` neu laufen lassen, sonst schlägt der Drift-Test
fehl. Expliziter Gate-Punkt.

---

## 4. Gate- & Build-Sequenz

1. **Pre-Merge-Hygiene:** Arbeitsbaum-Status klären (untracked: `.idea/`,
   `markdownai/`, `.serena/`, neues Spec-Doc) — commiten oder stashen; sauberer
   Baum vor dem Merge.
2. **`git merge main`** → Konflikte materialisieren.
3. **Konfliktauflösung datei-für-datei** nach Matrix (§2); Rust-Edits
   ausschließlich via Serena-Tools, ⚠-Dateien einzeln mit Diff + Freigabe.
4. **`version = "3.8.3-jb"`** in `rust/Cargo.toml`.
5. **`Cargo.lock`:** main's Lock als Basis; Regeneration auf `3.8.3-jb` beim
   ersten Gate-`cargo`-Lauf.
6. **Reformat** aller geänderten Dateien (`reformat_file`) — Projektregel vor
   `git add`.
7. **Gates (Agent):** `cargo nextest run` · `cargo clippy` · `cargo fmt --check`
   · Drift-/Conformance-Tests · ggf. `gen_docs` neu. Alle grün = Voraussetzung
   für den Commit.
8. **Merge-Commit** anlegen.
9. **Finaler Release-Build (Nutzer, manuell):** der Agent liefert den exakten
   Befehl (`cargo build --release` bzw. `cargo install --path rust`) → `3.8.3-jb`
   Binary. Der Agent führt ihn **nicht** aus.

---

## 5. Erfolgskriterien

- Merge-Commit auf `feat-jetbrains-plugin` mit aufgelösten Konflikten gemäß §2.
- `version = "3.8.3-jb"` in `rust/Cargo.toml`.
- Alle Gates grün: `cargo nextest run`, `cargo clippy`, `cargo fmt --check`,
  Drift-/Conformance-Tests.
- main's branch-relevante Verbesserungen (#413, #499) vollständig übernommen.
- JetBrains-Plugin-Funktionalität (v2c Two-Phase-Stack) unverändert erhalten.
- Build-Befehl für den manuellen Release-Build des Nutzers dokumentiert geliefert.

## 6. Nicht-Ziele

- Kein Rebase, kein selektives Cherry-pick (Ansätze B/C verworfen).
- Keine Ausführung des finalen Release-Builds durch den Agenten.
- Keine Änderung an additiven main-Subsystemen (Context OS / Commercial Plane).
- Keine Refactorings außerhalb der Konfliktauflösung.
