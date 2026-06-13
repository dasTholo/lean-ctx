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
| `intensive_benchmarks.rs` ⚠ | **main's `total < 12000`** in `bench_total_input_overhead` nehmen (Branch's `11600` verwerfen); übrige main-Hunks auto-merge | Beide ändern dieselbe assert-Zeile (Branch 11000→11600, main 11000→12000). main ist höher + dietisiert (#576) → Headroom. Übrige main-Änderungen (`instruction_decoder_block(false)`, #579-Cues, `essential_instructions`-Keywords) liegen disjunkt → von main. Siehe §3a. |
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

### Automatische Adoption — keine Branch-Anpassung nötig (`ctx_read`, `graph_index`)

Geprüft: Beide profitieren ohne Code-Änderung am Branch.

- **`ctx_read`** (`tools/registered/ctx_read.rs`, +69/−51 in main): **nur main**
  hat es geändert (byte-faithful / terse-skip / reference-results-Fix); der
  Branch fasst weder `ctx_read.rs` noch `read_modes`/`terse.rs` an → kommt 1:1
  von main, der Branch ist reiner Konsument und profitiert automatisch.
- **`graph_index`**: asymmetrisch. Branch fügt nur `get_forward_deps` hinzu
  (unabhängig, nutzt nur `self.edges`); main bringt C#-Namespace- + Godot-`.tscn`-
  Edges (#316), content-aware Staleness (#324), bessere scan-root-Erkennung
  (GL#438), `require_git(false)` (#400), `purge_index`, content-cache-gestützte
  Import-Resolution. Diese **verbessern die Graph-Qualität, die `get_forward_deps`
  liest** — reiner Gewinn, keine Branch-Änderung.

### Achtungspunkt — semantische auto-merge-Brüche (API-Umbauten in main)

main hat zwei APIs umgebaut, die textuell sauber auto-mergen, aber semantisch
brechen könnten: `import_resolver::ResolverContext::new` (+`content_cache`-Param)
und `cloud_files::keep_entry` → `walk_filter::keep_entry` (move). **Verifiziert
(2026-06-13):** kein Branch-geänderter Quellcode ruft die alten Formen auf — alle
Treffer liegen in Dateien, die der Branch nicht angefasst hat (`edges.rs`,
`ctx_impact.rs`, `bm25_index`, `search_index`, `ctx_search`, `ctx_tree`) und die
main beim Merge sauber gewinnt. `graph_index/mod.rs` ist unkritisch (Branch ändert
dort nur `get_forward_deps`, main's `scan_inner`-Umbau liegt im disjunkten
Bereich). Definitiver Fang für diese Risikoklasse bleibt der `cargo nextest` +
`clippy`-Gate (§4).

### §3a — Token-Efficiency-Epic #571 (lean surface #575 + schema diet #576) ↔ Branch

main's EPIC #571 (`351af7b9`, v3.8.3) senkt den Fixkosten-Overhead 13.7K→6.0K
tok/Session. Zwei Teile berühren den Branch:

**#575 — Lean default tool surface (Verhaltensänderung, kein Code-Konflikt).**
Default ist jetzt die Lean-Core-Surface: `setup` pinnt **kein** `tool_profile`
mehr, advertised werden nur die 13 `CORE_TOOL_NAMES` (`ctx_read`, `ctx_search`,
`ctx_shell`, `shell`, `ctx_tree`, `ctx_edit`, `ctx_session`, `ctx_knowledge`,
`ctx_overview`, `ctx_graph`, `ctx_call`, `ctx_provider`, `ctx_expand`); alles
andere bleibt über den force-advertised `ctx_call` (INVOKER) erreichbar.
`lean-ctx tools lean/reset` verwalten es. Die Maschinerie (`tool_visibility.rs`
neu, `dynamic_tools.rs`, `tool_profiles.rs`, `profile_cmd.rs`) ist **rein main** —
der Branch fasst nichts davon an → **kein Code-Konflikt**.
- **`ctx_refactor` ist NICHT Core** → standardmäßig nicht in `tools/list`, nur via
  `ctx_call`. Das Projekt pinnt `tool_profile = power` → `explicit_profile=true`
  → `ProfileAuthoritative` → `ctx_refactor` (mit v2c-Actions) voll sichtbar.
  Kein Problem für Projekt-/Subagent-Nutzung; **Doku-Punkt** nur für
  Plugin-Endnutzer ohne gepinntes Profil (dort: `ctx_call` oder `tools reset`).

**#576 — Schema diet ↔ `ctx_refactor`-Bloat (echter Anpassungsbedarf).**
#576 trimmt Core-Tool-Descriptions/Schemas −36 % und faltet große Action-Enums in
**pipe-delimited Descriptions**; ein Budget-Regression-Test sichert das ab.
- Der **harte** Per-Tool-Budget-Test (`tool_visibility::core_tool_surface_stays_within_budget`,
  300 tok/Tool, 2000 total) prüft **nur die Core-13** → `ctx_refactor` ist nicht
  betroffen, sprengt ihn **nicht**.
- Aber der Branch bläht `ctx_refactor` entgegen dem #576-Idiom auf (18-Action-
  **JSON-enum-Array** + lange Description + 14 Properties). Das drückt gegen die
  Full-Surface-Budgets `bench_total_input_overhead (<12000)` und
  `bench_tool_descriptions (<3000)` in `intensive_benchmarks.rs`.
- **Aktion (bedingt, nach dem Merge gemessen):** Gate-Lauf misst den realen
  Full-Surface-Overhead. Bleibt er mit Headroom < 12000 → Schema-diet-Angleichung
  ist **empfohlener Follow-up** (Stil + Headroom). Liegt er nahe/über 12000 →
  **Pflicht**: `ctx_refactor`-Schema an #576 angleichen (Action-Enum in
  pipe-delimited Description falten, Property-Descriptions kürzen). Das erledigt
  zugleich den vom Branch-Autor selbst notierten Code-TODO *„v2c FOLLOW-UP:
  analyze the real overhead drivers instead of raising this ceiling further"*.

### §3b — Invariante: Funktionserhalt unter Schema-diet + Lean-Surface (HARTE PLAN-ANFORDERUNG)

Schema-diet + Lean-Default dürfen **nie** dazu führen, dass Agent oder MCP-Server
die Übersicht verlieren, was `ctx_refactor` (oder ein anderes Tool) kann. Konkret:

- **Schema-diet = Format-Umstellung, kein Informationsverlust.** Die Action-Liste
  wandert vom `"enum": [...]`-Array in die `"description"` als **pipe-delimited
  Text** — Vorbild `registered/ctx_graph.rs` (main):
  `"action": { "type": "string", "description": "build|related|symbol|…" }`.
  **Keine** Action und **kein** Param wird gestrichen; alle 18 v2c-Actions
  (`rename|rename_preview|rename_apply|move_preview|move_apply|safe_delete_preview|
  safe_delete_apply|references|definition|implementations|declaration|
  type_hierarchy|symbols_overview|inspections|replace_symbol_body|
  insert_before_symbol|insert_after_symbol`) bleiben in der Description.
- **Verfügbarkeit ≠ Sichtbarkeit.** `ctx_refactor` bleibt in der Registry
  registriert (`build_registry()` → `registry.rs:139`). Im Lean-Default (nicht
  Core) ist es zwar nicht in `tools/list`, aber über den force-advertised
  `ctx_call` (INVOKER) und `discover_tools` / `ctx_tools find` voll erreichbar.
  Unter `tool_profile = power` (Projekt) ohnehin voll advertised.

**Kanonische Quelle MUSS neuinstallations-sicher sein (im Binary, nicht im Repo).**
Die Übersichts-Quellen zerfallen in zwei Klassen — entscheidend, weil eine reine
Binary-Installation (`cargo install` / binstall / Release-Binary) **nur das
Binary** mitbringt, **keine Repo-Dateien**:

| # | Quelle | Ort | Bei Neuinstallation |
|---|---|---|---|
| 1 | Tool-Description (pipe-delimited Actions) | **Binary** (`tool_def`, `&'static str`) | ✅ immer da |
| 2 | `discover_tools` / `ctx_call` | **Binary** (liest `build_registry().tool_defs()`) | ✅ immer da |
| 3 | `appendix-mcp-tools.md` + `generated/mcp-tools.md` | **Repo-Dateien** | ❌ fehlen |
| 4 | `LEAN-CTX.md` (#579) | von `init`/`setup` generiert | ✅ da, trägt Regeln/Modi, **nicht** die Action-Liste |

Daraus folgt: Die **einkompilierte `tool_def()`-Description** (Quellen 1+2) ist die
einzig neuinstallations-sichere Übersicht — genau deshalb ist das ctx_graph-Idiom
(Actions in die Description falten) die robuste Lösung und **nicht** das Risiko:
die Info wird ins Binary verlagert, nicht in eine Repo-Doc ausgelagert. Die
Repo-Docs (#3) sind nur ein **Entwickler-/CI-Spiegel** (Drift-Test) und dürfen
**nie** die einzige Action-Quelle sein.

- **Plan-Anforderung (verschärft):**
  1. Die **vollständige** v2c-Action-Liste steht in der einkompilierten
     `ctx_refactor`-`tool_def()`-Description (ctx_graph-Idiom) — nie nur in
     Repo-Docs.
  2. `appendix-mcp-tools.md` + `generated/mcp-tools.md` bleiben über `gen_docs` +
     Drift-Test (Gate §4) synchron — als Spiegel, nicht als Quelle.
  3. Smoke-Check bestätigt, dass `discover_tools("refactor")` /
     `ctx_call name=ctx_refactor` (beide aus dem Binary) die volle Action-Liste
     liefern — der Test, der den Neuinstallations-Fall abdeckt.

### §3c — Filesystem-Boundary / PathJail (#392, GH #392) ↔ v2c-move

Der Branch nutzt PathJail intensiv (v2c move: 3-Stage-PathJail → `INVALID_TARGET`).
Geprüft: **alle PathJail-Kern-APIs sind signatur-stabil**, der Branch fasst
`pathjail.rs`/`path_resolve.rs` nicht an → sie kommen 1:1 von main.

- **Stabile APIs (keine Code-Anpassung nötig):**
  `resolve_tool_path(project_root, shell_cwd, raw) -> Result<String,String>` —
  identisch (genau die Fn, die v2c-move/rename aufruft, `ctx_refactor.rs:540/1058`);
  `jail_path(path, root)`, `allow_paths_from_env_and_config()` — unverändert.
- **main-Änderungen = additiv/Verhalten, keine API-Brüche:**
  - **#392** expandiert `~`/`$VAR`/`${VAR}` in `allow_paths`/`extra_roots`
    (vorher literal gematcht, nie getroffen) — neue `expand_user_path`.
    **Gewinn** für den Branch: move-Ziel-Validierung über `extra_roots`
    respektiert nun Tilde/Env.
  - **#422** IDE-Config-Dirs (`~/.cursor`, `~/.claude`, …) sind jetzt **opt-in**
    (`allow_ide_config_dirs = true` / `LEAN_CTX_ALLOW_IDE_DIRS=1`); nur
    `~/.lean-ctx` immer erlaubt. Marginal für JetBrains-Refactoring.
  - **#415** relative Pfade werden **nie** gegen den Prozess-CWD aufgelöst
    (nur `is_absolute`, kein `exists()`-Kurzschluss) → deterministisch über
    MCP/daemon/CLI. **Gewinn**: Branch nutzt `Some(project_root)` +
    projekt-relative Pfade.
  - **GL#442 / #397** Windows-Reparse/Junction-Reject + Unix-Drive-Translation-
    Fix — Korrektheit, marginal.
- **`tool_trait.rs` (auto-merge):** main +`ShellOutcome`/`shell_outcome`/
  `get_usize`; Branch +`impl Default for ToolContext`. Disjunkt → sauber. main
  berührt das `ToolContext`-Struct **nicht** → der Branch's `impl Default` bleibt
  vollständig (kein `missing field`). Folge-Effekt `shell_outcome: None` im
  ctx_refactor-`ToolOutput` ist bereits in §2 erfasst.
- **Anpassungsbedarf:** am PathJail-Aufruf-Code **keiner**. Einzige Wachsamkeit:
  Gate-Lauf bestätigt, dass die v2c-move/rename-**Tests** mit #415 (kein
  CWD-Kurzschluss) grün bleiben. `appendix-paths-and-config.md` = Repo-Doc, von
  main, kein Konflikt.

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
   für den Commit. **Fängt insbesondere die semantischen auto-merge-Brüche aus
   §3 ab** (API-Umbauten `ResolverContext::new` / `walk_filter::keep_entry`):
   ein textuell sauberer Merge, der eine umgebaute API in alter Form aufruft,
   schlägt hier als Compile-/Clippy-Fehler fehl.
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
- **Schema-diet + Lean-Surface (#576/#575) eingehalten — ohne Funktionsverlust
  (§3b):** `ctx_refactor` folgt dem ctx_graph-Idiom (Action-Enum als
  pipe-delimited Description, keine Action/Param gestrichen); `bench_total_input_overhead`
  < 12000 grün; `discover_tools("refactor")` + `ctx_call name=ctx_refactor`
  liefern die volle Action-Liste; `appendix-mcp-tools.md` + `generated/mcp-tools.md`
  führen alle v2c-Actions (Drift-Test grün). Agent + MCP-Server behalten die volle
  Refactoring-Übersicht.
- Build-Befehl für den manuellen Release-Build des Nutzers dokumentiert geliefert.

## 6. Nicht-Ziele

- Kein Rebase, kein selektives Cherry-pick (Ansätze B/C verworfen).
- Keine Ausführung des finalen Release-Builds durch den Agenten.
- Keine Änderung an additiven main-Subsystemen (Context OS / Commercial Plane).
- Keine Refactorings außerhalb der Konfliktauflösung.
