# Design-Spec: JetBrains-Plugin Phase 5a — Härtung (Rust-Stale-Cache + Kanonisierung + Surfacing + CI)

| Feld             | Wert                                                                                               |
|------------------|----------------------------------------------------------------------------------------------------|
| Status           | Genehmigt (Design), bereit für `writing-plans`                                                     |
| Datum            | 2026-06-08                                                                                         |
| Branch           | `feat-jetbrains-plugin`                                                                            |
| Vorgänger        | Phase 4 (Commits `6228d88e`→`e794bd01`) — `type_hierarchy` + `symbols_overview`, abgeschlossen     |
| Eltern-Spec      | `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` (§9 Phase 5, §14.1, §17.6) |
| Schwester-Spec   | Phase 5b — `format` + `inspections` (eigener Spec, **nach** 5a)                                    |
| Nächster Schritt | `superpowers:writing-plans` (Implementierungsplan)                                                 |
| Sprache          | Code/Kommentare Englisch; Spec Deutsch                                                             |

---

## 1. Context — Warum

§9 des Eltern-Specs bündelte „Phase 5" ursprünglich als **format + inspections + Härtung** in
einem Commit. Zwei Befunde rechtfertigen einen **Split** (User-Entscheidung 2026-06-08):

1. **`format`/`inspections` sind PSI-Feature-Endpoints** (wie Phase-3-Nav, Phase-4-Hierarchie):
   neue Handler + DTOs + Wire + Tool-Actions. Die `LspBackend`-Trait-Shapes stehen bereits als
   Default-`Err` (`rust/src/lsp/backend.rs:95-100`) — exakt das Phase-4-Muster. → **Phase 5b**.
2. **Die „Härtung" ist nach der bereits gelandeten Kotlin-Port-Lifecycle-Arbeit fast nur noch
   Rust + CI.** Die Plugin-seitige Port-Datei-Härtung (Stale-Reaping, PID-Liveness, Heartbeat,
   Watcher, atomarer `0600`-Write) ist **schon auf dem Branch** (Commits `0f20444d`→`d2fd93f9`,
   Spec `acc5374e`). Offen bleibt die **Rust-Seite** + Test-Hygiene + CI.

**Phase 5a = Robustheit-only, ein Commit (§12.3 Eltern-Spec), KEINE neuen Wire-Endpoints.**
Sie schließt die in Phase 0/1/3/4 angelegten Härtungs-Follow-ups, die produktiv relevant sind.

### Offener Rest (Härtung)

| #  | Item                                                                | Quelle                        |
|----|---------------------------------------------------------------------|-------------------------------|
| H1 | Rust Stale-Cache-Invalidierung in `with_backend`/`select_backend`   | §14.1 #2 (Kern, Ziel Phase 5) |
| H2 | `project_root`-Kanonisierung im `jetbrains_backend.rs` (§5.5-Trap)  | §14.1 #1 / §17.6 #4           |
| H3 | `truncated`/`total` Rust-seitig im `ctx_refactor`-Output surfacen   | §17.6 #1 / Phase-4 §6.1       |
| H4 | Plugin-CI-Job (Gradle-Test headless)                                | §9                            |
| H5 | Test-Hygiene: `unitTest_*`-Port-Datei-Leak (a) + Same-Root-Doku (b) | Memory-Befunde                |

---

## 2. Fixierte Entscheidungen (User, 2026-06-08)

| # | Entscheidung                                                                                    | Begründung                                                                                                                                                                                                                                                 |
|---|-------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | **Schnitt: Härtung zuerst (5a), Features (`format`/`inspections`) später (5b).**                | Robustheit ist korrektheits-relevant (tote HTTP-Endpoints, falsche Pfade); format/inspections sind additive Komfort-Ops. Sauberer reviewbarer Schnitt, konsistent mit dem Ein-Commit-Rhythmus der Phasen 2–4.                                              |
| 2 | **H1 Re-Validierung: günstiger `pid` + Port-Datei-Check pro Call** (kein health-Ping pro Call). | `kill(pid,0)` + ein fs-Read = sub-ms; deckt den häufigsten Fall (IDE geschlossen) und IDE-Neustart (neuer Port/Token). Voller health-Ping pro Call = ms-HTTP-Latenz auf jedem `ctx_refactor`. Spiegelt den Kotlin-Reaper (pid-basiert).                    |
| 3 | **H5(b) Same-Root-Kollision: dokumentieren, Fix später.**                                       | Betrifft nur den seltenen Fall (gleichzeitig `runIde`-Sandbox + Produktiv-IDE auf demselben Root). Echter Fix (pid-suffigierte Multi-Datei-Discovery) zieht Kotlin-Writer/Reaper + Rust-Discovery nach → bläht den Härtungs-Commit auf. Eigener Follow-up. |
| 4 | **Keine neuen Wire-Endpoints in 5a.**                                                           | Reine Rust-/CI-/Test-Härtung. Bestehende Wire-Shapes (§6, Phase-3/4-Deltas) unverändert.                                                                                                                                                                   |

---

## 3. Architektur — Neue/erweiterte Komponenten

### 3.1 H1 — Stale-Cache-Invalidierung (Rust)

| Datei (~erweitert)                      | Aufgabe                                                                                                                                                                                                                                                                                                                    |
|-----------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `rust/src/lsp/backend.rs` (~)           | Neue **Default**-Trait-Methode `fn is_stale(&self, _project_root: &str) -> bool { false }` (Self-Management-Block, nach den Default-degrading-Methoden). Backing A (`LspClient`) erbt `false` → nie stale.                                                                                                                 |
| `rust/src/lsp/jetbrains_backend.rs` (~) | Struct um `pid: u32` (+ `port: u16`) erweitern; `new(port, token, project_root, pid)` (select_backend hat `pf.pid`). `is_stale` überschreiben: Port-Datei für `project_root` neu lesen — **weg** ∨ `pid` ≠ gespeichert ∨ Port ≠ gespeichert ∨ `!pid_alive(pid)` ⇒ `true`. **Kein** HTTP (`health` bewusst nicht pro Call). |
| `rust/src/lsp/router.rs` (~)            | In `with_backend` (L110-119) **vor** Nutzung des Cache-Eintrags: `if backend.is_stale(project_root) { backends.remove(language); }` → danach der bestehende `!contains_key`-Pfad re-selektiert (`select_backend`). Auto → Fallback A; `b_only` → `Err` (bestehende Logik).                                                 |

**Effekt:** IDE geschlossen nach Cachen eines B-Eintrags → nächster `ctx_refactor` erkennt
`is_stale`, evictet, fällt sauber auf Backing A (auto) bzw. liefert sauberen `Err` (b_only) —
**kein** Hänger gegen einen toten HTTP-Endpoint mehr. IDE-Neustart (neuer Port/Token) wird beim
Re-Select automatisch aufgegriffen.

### 3.2 H2 — `project_root`-Kanonisierung (Rust, §5.5-Trap)

| Datei (~)                               | Aufgabe                                                                                                                                                                                                                                                                                                                                      |
|-----------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `rust/src/lsp/jetbrains_backend.rs` (~) | In `new` `project_root` **einmalig** kanonisieren: `std::fs::canonicalize` (realpath, löst Symlinks) + Trailing-`/`-Trim. Danach arbeiten `position_body` (`strip_prefix`, L147-158) und `rel_to_uri` (`format!("{root}/{rel}")`, L54-57) auf dem kanonischen Root. Fehler-Guard: `canonicalize`-Err → Fallback auf Roh-Root (+`log::warn`). |

**Parität:** Die Kotlin-Seite leitet den Port-Datei-Key bereits aus `sha256(realpath(root))[..16]`
ab (`LeanCtxPaths`, Phase 2) — Rust muss denselben realpath nutzen, damit Pfad-Ableitung und
`project_hash` **byte-identisch** bleiben (deckt die §5.5-Forderung). Bei symlinktem Root oder
Trailing-Slash schlägt `strip_prefix` sonst fehl → der absolute Pfad ginge als vermeintlich
relativer an die IDE; `rel_to_uri` könnte Double-Slash erzeugen.

### 3.3 H3 — `truncated`/`total` surfacen (Rust)

| Datei (~)                               | Aufgabe                                                                                                                                                                                                                                                                     |
|-----------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `rust/src/lsp/jetbrains_backend.rs` (~) | Parser (`parse_locations` L65-84, `parse_type_hierarchy` L86-116, `parse_symbols` L118-133) lesen `truncated`/`total` (heute toleriert, nicht ausgewertet). Rückgabe um diese Felder anreichern (z. B. via Wrapper-Struct oder zusätzlichem Rückgabewert — Detail im Plan). |
| `rust/src/tools/ctx_refactor.rs` (~)    | Output-Suffix anhängen, wenn `truncated`: `… (truncated — N von M)` für Listen, `(truncated, N nodes)` für `type_hierarchy`. Backing A → immer `false` → kein Suffix.                                                                                                       |

**Ziel:** Der Agent sieht die Unvollständigkeit der Treffer-/Knotenliste statt sie für komplett
zu halten (Cap 500 bzw. `maxNodes`/`maxDepth` aus Phase 3/4).

### 3.4 H4 — Plugin-CI-Job (Infra)

| Datei (neu/~)                                     | Aufgabe                                                                                                                                                                                                                                   |
|---------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| CI-Workflow (`.github/workflows/…` o. ä.) (neu/~) | Job: `./gradlew test` in `packages/jetbrains-lean-ctx`, **headless** (BasePlatformTestCase läuft headless), IC `2026.1.3` gepinnt (bestehende `build.gradle.kts`-Pins). Cacht Gradle/IC-Download. Setzt H5(a) voraus (sauberes Data-Dir). |

### 3.5 H5 — Test-Hygiene

| Item                        | Aufgabe                                                                                                                                                                                                                                                                                                                                                   |
|-----------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **(a)** `unitTest_*`-Leak   | BasePlatformTestCase bootet via `LeanCtxStartupActivity` einen echten `BackendHttpServer` → schreibt Port-Dateien ins reale Data-Dir (`~/.lean-ctx` bzw. `LEAN_CTX_DATA_DIR`). Fix: Test-Setup setzt `LEAN_CTX_DATA_DIR` auf ein Temp-Dir (pro Test/Suite) + Teardown-Assertion „keine Port-Dateien übrig". Hält CI (H4) und Entwickler-Maschinen sauber. |
| **(b)** Same-Root-Kollision | **Nur dokumentiert** (Entscheidung #3): bekannte Limitierung in §6 (Follow-up). Kein Code-Fix in 5a.                                                                                                                                                                                                                                                      |

---

## 4. Wire-Protokoll

**Keine Änderung.** Phase 5a fügt keine Endpoints hinzu und ändert keine Request/Response-Shapes.
`truncated`/`total` werden seit Phase 3/4 bereits über die Wire geliefert (toleriert) — H3 wertet
sie nur **Rust-seitig** aus. Degradierung unverändert: auto → Fallback A; `b_only` → `Err`.

---

## 5. Verifikation (End-to-End) — Gate

1. **`cargo nextest run`** grün (niemals `cargo test`):
    - **H1:** Stub-Backend, das `is_stale=true` meldet → `with_backend` evictet + re-selektiert
      (auto → Backing A; `b_only` → `Err`). `is_stale=false` → Cache-Eintrag bleibt. Test über
      `seed_stub_backend`-Seam (`router.rs:155`).
    - **H1 (jetbrains):** `JetBrainsHttpBackend::is_stale` → Port-Datei weg / `pid` geändert /
      Port geändert / pid tot ⇒ `true`; unveränderte lebende Port-Datei ⇒ `false`.
    - **H2:** Kanonisierung — symlinkter Root + Trailing-Slash erzeugen denselben kanonischen
      Root → `strip_prefix` greift; `project_hash`-Parität (identisch zur Kotlin-`sha256`).
    - **H3:** Parser lesen `truncated`/`total`; `ctx_refactor`-Output trägt das Suffix bei
      `truncated=true`, keins bei `false`/Backing A.
    - Backing-A-Regressionsschutz (kein `scope`/`truncated`-Verhalten, kein `is_stale`).
    - `cargo clippy --all-targets` ohne neue Lints.
2. **Kotlin `./gradlew test`** grün — H4-CI-Job führt ihn headless aus; H5(a): Suite hinterlässt
   **keine** Port-Dateien im (Temp-)Data-Dir.
3. **Manuelles `runIde`** (IC/IU-2026.1.x): IDE auf → `ctx_refactor` (B gecacht) → IDE schließen →
   nächster `ctx_refactor` fällt **sauber auf Backing A** (kein Hänger, kein Timeout gegen toten
   Endpoint). Symlinkter Projekt-Root → `references` lösen weiterhin auf.
4. **Fallback ohne IDE** → Backing A unverändert (Regressionsschutz).
5. **Companion-Plugin** (Statusbar/Actions) weiterhin funktional (keine Regression).

---

## 6. Offene Follow-ups (in 5a angelegt / übernommen, später)

1. **`format` + `inspections`** — Phase 5b (eigener Spec). Trait-Shapes stehen bereits
   (`backend.rs:95-100`); Plugin-Handler + Wire + Tool-Actions analog Phase 3/4.
2. **Same-Root-Port-Datei-Kollision (H5b).** `runIde`-Sandbox + Produktiv-IDE auf demselben Root
   schreiben dieselbe `jetbrains-<sha256(realpath)[..16]>.port`, letzter Schreiber gewinnt
   (Memory `phase3_e1_gate_continuation`). **Bekannte Limitierung.** Fix-Skizze: pid-suffigierter
   Dateiname (`jetbrains-<hash>-<pid>.port`) + Rust-Discovery listet alle passenden, wählt die mit
   lebendem `pid` + `health`-ok. Zieht Kotlin-Writer/Reaper + Rust-Discovery nach.
3. **`scope=all` Token-Volumen** — bibliotheksweite Suchen können trotz Cap groß sein; beobachten,
   nicht vorab optimieren (§17.6 #2 / Phase-4 §6.2).
4. **Java-Fixtures** — Plugin-Regression ist Kotlin-only; Java-Abdeckung relevant spätestens beim
   K2-Fallback (§17.6 #3 / Phase-4 §6.3) — gehört zu Phase 5b (format/inspections sind sprach-
   neutraler).
5. **Live-`ctx_refactor`-`.kt`-E2E** (env-gebunden) — setzt voraus, dass die MCP-Server-Projekt-
   wurzel == das in der Sandbox geöffnete Projekt ist (Port-Discovery über `project_hash`); separat
   vom direkten HTTP-Pfad. Carry-over aus Phase-3/4-Gate-Protokoll.

---

## 7. Risiken

- **`is_stale`-Korrektheit bei IDE-Neustart.** Startet die IDE während einer Session neu, ändern
  sich Port **und** Token. `is_stale` muss `pid`-**und**-Port-Mismatch erkennen (nicht nur pid-tot),
  sonst spräche ein gecachter Backend mit altem Token gegen den neuen Server (→ 401). Test deckt
  „Port geändert ⇒ stale" explizit ab (§5.1).
- **`canonicalize`-Verfügbarkeit.** `std::fs::canonicalize` erfordert einen existierenden Pfad.
  `project_root` existiert immer (sonst gäbe es keine Session) — Fehler-Guard fällt dennoch auf
  Roh-Root zurück, statt zu panicken.
- **CI-Headless-Stabilität (H4).** BasePlatformTestCase lädt die IC-Plattform (~1 GB) — CI-Job
  braucht Gradle-/IC-Cache, sonst lange/teure Läufe. Pinning auf `2026.1.3` (bestehend) hält den
  Download deterministisch. Kein `runIde` in CI (bleibt manuelles Gate).
- **H5(a)-Env-Isolation.** Setzt das Test-Setup `LEAN_CTX_DATA_DIR` global statt pro-Suite, können
  parallele Gradle-Worker kollidieren. Pro-Test-Temp-Dir bevorzugen.

---

## 8. Referenz-Artefakte

- Stale-Cache-Pfad: `rust/src/lsp/router.rs:58-122` (`select_backend`/`with_backend`/`BACKENDS`),
  `rust/src/lsp/port_discovery.rs` (`read_port_file`/`pid_alive`/`health_ok`).
- Trait: `rust/src/lsp/backend.rs:53-101` (`LspBackend`; `is_stale` neu im Self-Management-Block).
- HTTP-Backend: `rust/src/lsp/jetbrains_backend.rs:16-30` (Struct/`new`), `:54-57` (`rel_to_uri`),
  `:65-133` (Parser), `:147-158` (`position_body`).
- Tool-Output: `rust/src/tools/ctx_refactor.rs`.
- Kotlin-Port-Lifecycle (bereits fertig, Referenz): `0f20444d`→`d2fd93f9`,
  `BackendHttpServer`/`PortFileWriter`/`StalePortFileReaper`/`PortFileWatcher`/`PortFileHeartbeat`,
  `ProcessLiveness`/`PortFileReader` (`packages/jetbrains-lean-ctx`).
- Memory-Befunde: `jetbrains-BackendHttpServer-self-healing-lifecycle`,
  `jetbrains-port-cleanup-T1-T5-done`, `phase3_e1_gate_continuation`.

---

## 9. Gate-Protokoll Phase 5a

**Status:** Implementierung abgeschlossen (alle Tasks spec- + quality-reviewed), automatisierte
Gates grün. Ausgeführt via `superpowers:subagent-driven-development` auf `feat-jetbrains-plugin`
(kein Worktree), Final-Gate am 2026-06-08.

### 9.1 Härtungs-Commits (H1–H5a)

| #  | Commit     | Inhalt                                                                          | Ergebnis                                                       |
| H1 | `4b60d3fa` | `is_stale`-Trait-Methode + JetBrains pid/port-Liveness-Self-Check + Test-Env-Lock-Fix | grün — Default `false` (Backing A), B liest Port-Datei neu     |
| H1 | `67b487c0` | Router evictet stale Backing-B-Cache-Einträge vor Re-Use (`with_backend`)       | grün — auto → Fallback A, `b_only` → sauberer `Err`           |
| H2 | `da88d525` | `project_root`-Kanonisierung im JetBrains-Backend (realpath-Parität zur Kotlin-`sha256`) | grün — symlinkter Root + Trailing-Slash → `strip_prefix` greift |
| H3 | `8f9b9912` | `truncated`/`total` Rust-seitig im `ctx_refactor`-Output surfacen                | grün — Suffix nur bei `truncated=true`; Backing A → kein Suffix |
| H4 | `bcefc0b3` | Plugin-CI-Workflow (Build + headless Test, JVM 21)                              | YAML geprüft, lokaler `gradle check`-Lauf grün (s. 9.3)        |
| H5a | `a3e6042d` | Port-Datei-Hygiene-Test (kein Leak ins reale Data-Dir, `PortFileHygieneTest`)   | grün — Suite hinterlässt keine Port-Dateien                    |

### 9.2 User-Erweiterung (mid-execution, nicht im ursprünglichen Plan)

| Commit     | Inhalt                                                                                                          |
| `f15bd096` | Release-Asset-Upload bei GitHub-Release (`gh release upload build/distributions/*.zip`, **kein** Marketplace-Publish) + Plugin-Versions-Bump `1.0.0` → `3.7.5` (Spiegelung der lean-ctx-Version). |

### 9.3 Automatisierte Gates (Final-Gate, 2026-06-08)

| Gate                        | Befehl                                              | Ergebnis                                                                                                       |
| **Rust-Tests**              | `cargo nextest run -p lean-ctx --no-fail-fast`      | **5073 von 5075 passed**, 14 skipped. Die **2 Fails** sind pre-existing Baseline (`hn_hardening_scenarios`: `double_compression_guard::scenario_skip_terse_when_already_compressed` + `integration::scenario_shell_compression_with_saved_tokens_skips_terse`) — **nicht** durch Phase 5a eingeführt. Kein weiterer Fail. |
| **Rust-Clippy**             | `cargo clippy -p lean-ctx --all-targets`            | clean — nur die 3 Baseline-Warnungen (`core/config/tests.rs` ×2, `server/execute.rs` ×1). **Keine** neuen Lints in `backend.rs`/`jetbrains_backend.rs`/`router.rs`/`ctx_refactor.rs`. |
| **Kotlin-Gate**             | `./gradlew check` bzw. `test --rerun-tasks`         | **BUILD SUCCESSFUL** — **55 Tests** über 20 Test-Suites, alle grün, inkl. `PortFileHygieneTest` (1 Test, grün). (Frühere „26" stammte aus einem inkrementellen `test`-Lauf ohne `--rerun-tasks`; autoritativ ist die Voll-Wiederholung mit 55.) |
| **Drift-Gate (Doc-Gen)**    | `cargo nextest run`: `reference_docs_drift`, `docs_tool_counts_up_to_date`, `mcp_manifest_up_to_date` | **4 von 4 passed.** H3 änderte nur den **Output-Text** von `ctx_refactor`, nicht Action-Enum/Schema → generiertes Tool-Doc (`docs/reference/generated/mcp-tools.md`) **unverändert**; `git status --porcelain docs/reference/generated/` **clean** (keine Regenerierung nötig). |

### 9.4 Manuelles `runIde`-Gate — **PENDING / User-gated**

Nicht in diesem automatisierten Pass ausgeführt (separates Terminal, ~1 GB IC-Download). Offen:
IDE auf → `ctx_refactor` (Backing B gecacht) → IDE schließen → nächster `ctx_refactor` fällt
**sauber auf Backing A** (kein Hänger/Timeout gegen toten Endpoint); symlinkter Projekt-Root →
`references` lösen weiterhin auf (§5 Punkt 3). Wird vom User in einer Live-IDE-Session verifiziert.

### 9.5 Review-Befunde — bewusst out-of-scope belassen

Zwei CI-Nice-to-haves wurden vom Review markiert, aber außerhalb des Scopes gelassen (plan-
spezifiziertes YAML, geringes Risiko):

- Keine `concurrency`-Group im Plugin-Workflow (parallele Läufe nicht gecancelt).
- Kein `timeout-minutes` am Job; keine Action-SHA-Pinning (Versions-Tags statt Commit-SHA).
