# Design-Spec: lean-ctx JetBrains — Editor-Focus-Signal (#500-Producer-Parität)

| Feld        | Wert                                                                                                               |
|-------------|--------------------------------------------------------------------------------------------------------------------|
| Status      | Draft (Design genehmigt 2026-06-13)                                                                                |
| Branch      | `feat-jetbrains-plugin` (Fortführung, Muster v1-§12.3 — ein Commit pro Phase, kein worktree)                       |
| Scope       | JetBrains-Plugin meldet Editor-Fokus an lean-ctx (Producer-Seite von #500), 1:1-Parität zum VS-Code-Verhalten      |
| Vorhaben    | `EditorFocusReporter` + Verdrahtung in `LeanCtxStartupActivity`; Opt-out via IntelliJ-Registry-Key                 |
| Basis-#500  | `rust/src/core/editor_signal.rs` (Ingress + Speicher), `vscode-extension/src/editor-signal.ts` (Referenz-Producer) |
| Reihenfolge | **Vor** v2d. v2d (`inline` + `reformat`, §10 der v2c-Spec) bleibt unverändert als nächste Phase.                   |

---

## 1. Kontext & Motivation

**#500 (Editor focus)** kam mit dem Merge `main → feat-jetbrains-plugin` (v3.8.3). Es ist das
stärkste verfügbare Relevanz-Signal: die Datei, die der Entwickler *gerade ansieht*, soll im
Context-Ranking hochgewichtet werden.

**Architektur von #500 (bestehend, unverändert):**

- **Ingress (CLI):** `lean-ctx editor-signal --file <abs_path>` → `core::editor_signal::record_focus()`
  (`rust/src/cli/dispatch/mod.rs:519`). Kein Daemon, kein Socket, <10ms.
- **Speicher:** **eine globale** Datei `~/.lean-ctx/editor_signal.json`
  (`EditorSignal { active_file, recent_files[(path, ts)], updated_at }`), atomar geschrieben
  (tmp + rename). `recent_files`-Ring (dedup, active→recent-Promotion, `truncate(10)`),
  Pfad-Normalisierung via `normalize_tool_path`, Freshness-Fenster `FRESHNESS_SECS = 120`.
- **Consumer:** `core::editor_signal::apply_boost()` (`rust/src/tools/ctx_preload.rs:45`) hebt
  passende Kandidaten an (`active_file` +0.30, `recent_files` +0.10; Suffix-Match via
  `paths_match`). Außerdem Dashboard-„Editor focus"-Kachel (`dashboard/routes/signals.rs`,
  `cockpit-commander.js`).
- **Producer:** **nur VS Code** (`vscode-extension/src/editor-signal.ts`): `onDidChangeActiveTextEditor`
  → 2s-Debounce → fire-and-forget `lean-ctx editor-signal --file`. Pfad-only, nur Workspace-Dateien,
  nur `file`-Scheme.

**Die Lücke:** Das JetBrains-Plugin (`com.leanctx.plugin`) meldet **keinen** Editor-Fokus →
JetBrains-Nutzer bekommen das #500-Ranking nicht. Diese Phase schließt die Lücke, indem das Plugin
das VS-Code-Producer-Verhalten 1:1 spiegelt.

**Kein Scope-Konflikt mit v2c-§10:** Dort sind „Editor-UI-Tools des JetBrains-MCP" out-of-scope —
das meint lean-ctx, das *JetBrains-Editor-Tools ausführt* (Consumer von IDE-UI). Editor focus ist
das **Gegenteil**: das Plugin *meldet* Fokus an lean-ctx (Producer, wie `editor-signal.ts`).
Architektonisch unverwandt zu `inline`/`reformat` (kein HTTP-Backend, keine Refactoring-Engine,
kein `plan_hash`).

---

## 2. Getroffene Entscheidungen (User, 2026-06-13)

| # | Thema                | Entscheidung                                                                                                                                                                                                                                  |
|---|----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | Reihenfolge          | **Editor focus zuerst, eigene Phase.** Danach v2d = `inline` + `reformat` (v2c-§10) unverändert.                                                                                                                                              |
| 2 | Ingress-Mechanismus  | **Binary-Shell-out** via `BinaryResolver` — `lean-ctx editor-signal --file <path>` (wie VS Code). `record_focus()` bleibt einzige Wahrheit → **kein** Kotlin-Drift des Signal-Formats.                                                        |
| 3 | Opt-out              | **IntelliJ-Registry-Key** `leanctx.editor.signal.enabled` (default `true`), producer-seitig vor dem Spawn ausgewertet. **Keine** neue Plugin-Config-Schicht (kein `PersistentStateComponent`/`Configurable`), **keine** Rust-Config-Änderung. |
| 4 | Listener-Verdrahtung | **Ansatz A:** programmatisch in `LeanCtxStartupActivity` (Projekt-MessageBus). Ein Ort deckt Tab-Wechsel **und** Initial-Melden ab; reutilisiert den bestehenden Startup-Hook.                                                                |
| 5 | Signal-Scope         | **Global** (`editor_signal.json`, last-write-wins). Reine VS-Code-Parität, **keine** Rust-Änderung. Multi-Fenster-last-write-wins als bekannte #500-Grenze dokumentiert (§5).                                                                 |
| 6 | Akzeptanz            | Kotlin-Unit-Tests (Filter/Dedup/Registry-Gate) + **manuelles runIde-Gate** (Liefergegenstand: Runbook). Muster wie v2b/v2c-Gates.                                                                                                             |

**Begründungen (verdichtet):**

- **Shell-out statt In-Process-Write:** `record_focus()` macht nicht-triviale Zustandslogik
  (`recent_files`-Ring, `normalize_tool_path`, Freshness). In-Process-Write müsste das in Kotlin
  spiegeln + einen Cross-Language-Format-Vertragstest pflegen → dauerhafte Drift-Fläche.
  Shell-out reicht nur den Pfad; alles andere bleibt korrekt-für-frei. Preis: ein kurzlebiger,
  2s-debounced, fire-and-forget Prozess pro Tab-Wechsel (vernachlässigbar; VS Code macht es so).
- **Port-File ist NICHT das Vehikel:** Das Port-File ist **pro-Projekt** gekeyt (`projectHash`,
  `LeanCtxPaths.portFile`) für **Connection-Routing** (Rust-als-HTTP-Client muss das richtige
  Projekt-Backend erreichen — hart). `editor_signal.json` ist **eine globale** Datei für einen
  **weichen Ranking-Hinweis**, bewusst daemon-/socket-frei (#500-Modul-Doc). Verschiedene Artefakte,
  verschiedene Lebenszyklen — das Port-File zu überladen würde Ranking an den HTTP-Handshake koppeln.
- **Registry-Key statt sichtbarem Setting:** Das Plugin hat **keine** Config-Infrastruktur
  (kein `PersistentStateComponent`/`Configurable`/Registry-Nutzung). Eine sichtbare Settings-Seite
  wäre eine ganze neue Config-Schicht für ein Pfad-only-Signal. Der eingebaute IntelliJ-`Registry`
  gibt ein Opt-out für Power-User mit minimalem Code; sichtbare Seite später nachrüstbar.

---

## 3. Architektur & Komponenten

**Neue Datei:** `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`
— eine Klasse, ein Zweck: Fokus-Wechsel → gefilterter, debounced, fire-and-forget Binary-Call.
Trägt die **einzige** neue Logik.

**Geänderte Datei:** `LeanCtxStartupActivity.kt` — abonniert beim Projekt-Start den
`FileEditorManagerListener` (Topic `FileEditorManagerListener.FILE_EDITOR_MANAGER`) auf dem
Projekt-MessageBus und meldet die bereits offene Datei initial.

**Geänderte Datei:** `plugin.xml` — `<registryKey>`-Eintrag für `leanctx.editor.signal.enabled`
(default `true`, Beschreibung).

**Wiederverwendet (unverändert):** `BinaryResolver` (Pfad zum Binary). `LeanCtxPaths` nur
**indirekt** — der Binary schreibt nach `data_dir`, das `LeanCtxPaths.resolveDataDir` bereits spiegelt.

**Rust-Seite: unverändert.** `editor-signal`-CLI-Ingress + `record_focus()` + `apply_boost()` +
Dashboard existieren aus 3.8.3. Diese Phase fügt **nur** einen weiteren Producer hinzu.

### Komponenten-Verantwortung

- **`EditorFocusReporter`** (pro Projekt eine Instanz, an ein Projekt-`Disposable` gebunden):
    - hält den 2s-Debounce-`Alarm`, `lastSent`-Dedup
    - `onFileFocused(file: VirtualFile)`: Registry-Check → Filter → Dedup → Alarm
    - der Spawn liegt hinter einer **injizierbaren** Funktion/Interface (Testbarkeit, §6)
- **`LeanCtxStartupActivity`**: subscribt `selectionChanged → reporter.onFileFocused(event.newFile)`
  **und** ruft initial `reporter.onFileFocused(currentlyOpenFile)`.

---

## 4. Datenfluss

```
Tab-Wechsel (selectionChanged) ─┐
Plugin-Start (offene Datei)  ───┼─→ reporter.onFileFocused(VirtualFile)
                                │      ├─ Registry leanctx.editor.signal.enabled? nein → return
                                │      ├─ echte Projektdatei? (Filter §4.1) nein → return
                                │      ├─ Pfad == lastSent? ja → return
                                │      └─ Alarm: nach 2s → spawn (Background, NIE EDT)
                                └─→ BinaryResolver.resolve()
                                       └─ exec `lean-ctx editor-signal --file <absPath>` (fire-and-forget)
                                            └─ Rust: record_focus() → editor_signal.json (atomar)
                                                 └─ apply_boost() in ctx_preload → Ranking-Boost
```

### 4.1 Filter (genau wie VS Code, nach Kotlin übersetzt)

- `file.isInLocalFileSystem` **und** `!file.isDirectory` (nur echte Dateien — keine Scratch-/
  Decompiled-/Library-/Non-`file`-Scheme-Buffer).
- Datei liegt unter `project.basePath` (Projekt-zugehörig).
- **Pfad-only — nie Inhalt.** (Privacy-Eigenschaft von #500.)

---

## 5. Fehlerbehandlung & Edge-Cases

**Fire-and-forget, nie blockieren (EDT-Sicherheit):**

- Spawn auf Hintergrund-Thread (`AppExecutorUtil`/Pooled), **nie** auf dem EDT.
- `BinaryResolver.resolve() == null` (kein Binary) → still inert, kein Log-Spam.
- Binary zu alt (`editor-signal`-Subcommand fehlt → Exit ≠ 0) → still schlucken (wie VS Codes `.catch()`).
- Spawn-Exception (IO) → geschluckt; ein verlorenes Signal ist harmlos (nächster Tab-Wechsel sendet neu).

**Lifecycle:**

- `Alarm` an das Projekt-`Disposable` gebunden → bei Projekt-Schließung disposed (kein Leak, kein
  Spawn nach Close).
- `lastSent` pro Reporter-Instanz (pro Projekt).

**Edge-Cases:**

- **Mehrere IDE-Fenster/Projekte (bekannte #500-Grenze):** `editor_signal.json` ist **eine globale**
  Datei → last-write-wins. Fokus in Fenster X kann durch Fokus in Fenster Y überschrieben werden;
  X verliert dann seinen Boost (Consumer matcht ein fremdes `active_file` einfach nicht → harmlos,
  aber kein Boost). **Gilt identisch für VS Code** — vorbestehende #500-Eigenschaft, **kein**
  JetBrains-Regress. Eine per-Projekt-Korrektheit (Signal-Datei wie `portFile` gekeyt) wäre ein
  editor-übergreifender #500-Kern-Umbau (CLI-`--project-root`, alle Producer/Consumer/Dashboard)
  und ist **bewusst NICHT** Teil dieser Phase (§8).
- **Schnelles Tab-Hopping:** Debounce kollabiert auf den letzten Tab nach 2s Ruhe.
- **Datei außerhalb Projekt (z.B. Library-Source):** Filter verwirft → kein Signal.
- **Registry zur Laufzeit umgeschaltet:** beim nächsten `onFileFocused` ausgewertet (kein Neustart nötig).

---

## 6. Tests & Akzeptanz-Gate

**Kotlin-Unit-Tests** (`…/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt`, reine
Logik ohne IDE-Plattform-Treiber — Spawn hinter injizierbarer Funktion):

- Filter: lokale Projektdatei → akzeptiert; Verzeichnis / Non-local / außerhalb-`basePath` → verworfen.
- Dedup: gleicher Pfad zweimal → ein Spawn.
- Registry aus → kein Spawn.
- (Debounce-Kollaps ggf. über injizierbaren Alarm/Clock; sonst manuell im Gate.)

**Manuelles runIde-Gate** — Liefergegenstand `docs/lean-md/runbooks/runide-editor-focus-gate.md`
(Muster wie v2b/v2c-Gates):

1. `runIde`, Projekt öffnen, Datei A öffnen → `~/.lean-ctx/editor_signal.json` zeigt `active_file = A`.
2. Datei B öffnen → `active_file = B`, A in `recent_files`.
3. Dashboard-„Editor focus"-Kachel zeigt B (frisch).
4. Registry-Key `leanctx.editor.signal.enabled = false` → Tab-Wechsel ändert die Datei **nicht**.
5. Binary-Pfad ungültig / Binary fehlt → kein Crash, IDE stabil.

**Geerbte Gates:** `gradlew build` grün. **Keine Rust-Änderung** in dieser Phase → Rust-Tests/
`clippy`/`fmt` unberührt (nicht neu zu fahren, nur referenziert). **Kein** Schema-Drift-Gate
(keine MCP-Tool-/`ctx_refactor`-Schema-Änderung).

---

## 7. Betroffene Dateien (Übersicht)

| Datei                                                             | Änderung                                                              |
|-------------------------------------------------------------------|-----------------------------------------------------------------------|
| `…/plugin/EditorFocusReporter.kt`                                 | **NEU** — Filter, Debounce, Dedup, Registry-Gate, Spawn (injizierbar) |
| `…/plugin/LeanCtxStartupActivity.kt`                              | ~ Listener-Subscribe + Initial-Melden                                 |
| `…/src/main/resources/META-INF/plugin.xml`                        | + `<registryKey>` `leanctx.editor.signal.enabled` (default true)      |
| `…/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt` | **NEU** — Unit-Tests                                                  |
| `docs/lean-md/runbooks/runide-editor-focus-gate.md`               | **NEU** — manuelles Gate-Runbook                                      |
| Rust (`editor_signal.rs`, `ctx_preload.rs`, Dashboard, CLI)       | **unverändert** (nur referenziert)                                    |
| `vscode-extension/src/editor-signal.ts`                           | **unverändert** (Referenz-Producer)                                   |

---

## 8. Bewusst NICHT in dieser Phase (YAGNI)

- **Kein** per-Projekt-`editor_signal.json` / #500-Multi-Fenster-Umbau (editor-übergreifender
  Kern-Umbau; bekannte Grenze, §5). Falls je gewünscht: eigene editor-agnostische Phase.
- **Keine** sichtbare Settings-Seite (`PersistentStateComponent`/`Configurable`) — Registry-Key
  genügt; nachrüstbar.
- **Keine** Rust-Config-Änderung (kein zentrales `editor_signal_enabled`-Flag) — Opt-out lebt
  JetBrains-lokal im Registry-Key.
- **Kein** In-Process-Write der Signal-Datei (Drift-Vermeidung, §2).
- **Kein** HTTP-Daemon-Endpoint für das Signal (CLI-Ingress ist bewusst daemon-frei).
- **Kein** Inhalt im Signal — Pfad-only (Privacy, §4.1).

---

## 9. Verhältnis zu v2d

Diese Phase ist **unabhängig** von v2d und blockiert es nicht. v2d (`inline` + `reformat`,
v2c-Spec §10) bleibt als nächste Phase unverändert bestehen. Reihenfolge: **Editor focus → v2d**.
Die v2c-§10-Mitnahmen für v2d (z.B. v1-§13.3-`reformat_file`-Diskrepanz) sind von dieser Phase
nicht berührt.

---

## 10. Branch- & Commit-Strategie

- Fortführung auf `feat-jetbrains-plugin` (Muster v1-§12.3): **ein Commit pro Phase** nach
  erfülltem Gate, kein Squash während der Entwicklung. Direkt auf dem Branch, **kein worktree**
  (Projekt-Rule).
- Vor `git add`: `reformat_file` auf jede geänderte Datei (Projekt-Rule).
- Finaler Merge nach `main` via Squash-Merge-PR (gemeinsam mit dem JetBrains-Plugin-Strang).
