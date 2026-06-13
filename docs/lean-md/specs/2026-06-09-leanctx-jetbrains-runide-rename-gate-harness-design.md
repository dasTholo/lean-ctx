# lean-ctx JetBrains — Reproduzierbares runIde-Rename-Verifikations-Harness (Design)

> **Status:** Design (genehmigt). Folge-Schritt: `writing-plans` → Implementation-Plan.
> **Sprache:** Doku/Spec deutsch; Code + Bezeichner englisch.
> **Bezug:** v2b-Rename (`docs/lean-md/specs/2026-06-09-leanctx-jetbrains-v2b-refactoring-rename-design.md`, §10
> „manuelles runIde-Gate"); v2b-Plan (`docs/lean-md/plans/2026-06-09-leanctx-jetbrains-v2b-rename.md`, Task 14 Step 5).

## 1. Ziel

Das manuelle runIde-Gate für die v2b-Two-Phase-Rename-Operation (`rename_preview`/`rename_apply`) als **skriptbare,
reproduzierbare Befehlssequenz** ausführbar machen. Es soll den **vollen** v2b-Stack live verifizieren — die
Rust-Gate-Logik (`plan_hash`/TOCTOU, Konflikt-Gate, PathJail, Cache-Evict) **und** das JetBrains-Plugin (
`RenameProcessor`-Naht, Multi-File-Transaktion) — gegen ein sauberes Kotlin-Projekt mit korrektem Find-Usages-Scope.

## 2. Problem / Hintergrund

Die automatisierten v2b-Gates sind grün (Rust 5275/5275, Kotlin-Akzeptanztest, clippy/fmt). Der `BasePlatformTestCase`
-Light-Fixture des Kotlin-Akzeptanztests indiziert `project.basePath` aber **nicht** als Source-Root → die
resolve-basierte Cross-File-Usage-Suche findet 0 Treffer. Cross-File-Rename — das Kernfeature — ist daher **nur** live (
runIde) verifizierbar.

Beim Live-Versuch traten zwei strukturelle Hürden auf:

1. **Root-Matching.** lean-ctx löst den `project_root` **einmal pro MCP-Session** aus den Client-Roots auf (
   `call_tool.rs::resolve_roots_once`) und ist nicht mid-session umschaltbar. `live_jetbrains_backend(project_root)`
   findet das Plugin-Port-File über `jetbrains-<sha256(project_root)[..16]>.port`. Öffnet die Sandbox-IDE ein anderes
   Projekt als den Session-Root, schlägt das Matching fehl → korrektes `BACKEND_REQUIRED`.
2. **Find-Usages-Scope.** Das äußere Repo `/home/tholo/Scripts/lean-ctx` ist ein großes, gemischtes Projekt (Rust +
   Kotlin-Subprojekt + verschachtelte git-Repos + Submodule). Geöffnet als Outer-Repo ist `packages/jetbrains-lean-ctx`
   **nicht** als sauberes Gradle/Kotlin-Source-Modul importiert: `definition` (direkte PSI-Resolve) funktioniert, aber
   `findUsages`/`references` liefern 0 — auch die v1-`references`-Action mit `scope=all`. Das ist **upstream von v2b** (
   die IDE liefert eine leere Usage-Liste; v2b reicht sie korrekt durch).

**Kernspannung:** Die IDE muss ein **sauberes Kotlin-Gradle-Projekt** öffnen (für `findUsages`), UND lean-ctx muss die
Rename-Calls mit **genau diesem** Root machen (für Port-File-Matching). Beides gleichzeitig — ohne die in-session-
`ctx_refactor` (am äußeren Repo verankert) — ist der Zweck dieses Harness.

## 3. Architektur / Komponenten

### 3.1 `lean-ctx call` — neuer CLI-Subcommand (Rust)

Ein generischer one-shot Tool-Invocation-Befehl, der einen einzelnen registrierten Tool-Handler mit explizit gesetztem
`project_root` ausführt:

```
lean-ctx call <tool> --project-root <path> --json '<args-json>'
```

- **Verhalten:** Baut einen `ToolContext` mit `project_root = <path>` (umgeht `resolve_roots_once`), parst `<args-json>`
  zu den Tool-Argumenten, ruft den vorhandenen Tool-Handler (z.B. `crate::tools::ctx_refactor::handle`) und schreibt das
  Text-Ergebnis nach stdout. Exit-Code 0 bei Tool-Ausführung (auch bei fachlichem `ERROR:`-Ergebnis — das ist
  Tool-Output, kein Prozess-Fehler), ≠0 nur bei CLI-Fehlern (unbekanntes Tool, ungültiges JSON, fehlender
  Pflicht-Parameter).
- **Reuse, kein neues Tool-Logik:** nutzt das bestehende In-Process-Tool-Registry / die `McpTool`-Dispatch-Schicht. Die
  einzige Neuerung ist der CLI-Einstieg, der `project_root` injiziert statt aus MCP-Roots aufzulösen.
- **Generischer Nutzen:** jedes Tool an beliebigem Root one-shot ausführbar (Debugging/Skripting), nicht nur für dieses
  Gate.
- **`--json` Quelle:** Inline-String; optional `--json-file <path>` als Komfort (YAGNI: nur falls trivial mitnehmbar).

> **Sicherheit:** `--project-root` durchläuft dieselbe Validierung wie sonst (kein broad/unsafe root —
`core::pathutil::is_broad_or_unsafe_root`). Tool-interne PathJail bleibt unverändert wirksam.

### 3.2 Fixture — minimales Kotlin-Gradle-Projekt

Ein **committeter Generator** (Shell-Skript, z.B. `scripts/runide-gate-setup.sh`) materialisiert das Fixture nach
`tmp/runide-rename-gate/` (tmp/ ist gitignored Scratch → die Live-Kopie liegt dort; der Generator selbst ist
versioniert = reproduzierbar). Erzeugte Struktur:

```
tmp/runide-rename-gate/
  settings.gradle.kts        # rootProject.name = "runide-rename-gate"
  build.gradle.kts           # plugins { kotlin("jvm") version "<bundled-kompatibel>" }
  src/main/kotlin/
    Widget.kt                # package p; class Widget  (Rename-Ziel)
    Usage.kt                 # package p; fun use(): Widget = Widget()  (cross-file ref)
    Gadget.kt                # package p; class Gadget  (Namenskollision für Konflikt-Test)
  notes.txt                  # plain text (UNSUPPORTED_LANGUAGE-Fall)
```

Als eigenes Gradle-Projekt geöffnet bekommt es saubere Kotlin-Source-Roots + Projekt-Scope → `findUsages` über
`Widget.kt`/`Usage.kt` funktioniert.

> Das Fixture ist bewusst minimal und vom Haupt-Build entkoppelt (eigenes `settings.gradle.kts`, eigener Root) — es wird
> nie vom lean-ctx-Haupt- oder Plugin-Gradle-Build erfasst.

### 3.3 Runbook — committetes Doc

Ein Markdown-Runbook (`docs/lean-md/runbooks/runide-rename-gate.md` o.ä.) beschreibt die ausführbare Gate-Sequenz:

1. **Setup:** Generator ausführen → Fixture in `tmp/runide-rename-gate/`.
2. **Launch:** `./gradlew runIde --args="<abs tmp/runide-rename-gate>"` (cwd=`packages/jetbrains-lean-ctx`). Sandbox-IDE
   öffnet das Fixture; Indizierung abwarten.
3. **Gate-Checks** je als `lean-ctx call ctx_refactor --project-root <fixture> --json '…'` mit erwartetem Ergebnis:
    - **Preview cross-file:** `rename_preview name_path=Widget new_name=Gadget` → Usages über `Widget.kt` + `Usage.kt`,
      `plan_hash` gesetzt.
    - **Apply + Undo:** `rename_apply … plan_hash=<aus preview>` → alle Stellen umbenannt; in der IDE **ein**
      Undo-Eintrag (Strg+Z revertet komplett). Danach Strg+Z zum Zurücksetzen des Fixtures.
    - **TOCTOU:** Quelle zwischen preview & apply ändern → `rename_apply` mit altem `plan_hash` → `CONFLICT`.
    - **Konflikt ± force:** `new_name=Gadget` (Kollision mit `Gadget.kt`) → preview meldet `conflicts`; apply ohne
      `force` → `CONFLICT`; mit `force=true` → durchgereicht.
    - **INDEXING:** während frischer Indizierung (Projekt neu öffnen) → preview → `INDEXING`, kein Teil-Rename.
    - **UNSUPPORTED_LANGUAGE:** Ziel in `notes.txt` → `UNSUPPORTED_LANGUAGE`, kein Crash.
    - **BACKEND_REQUIRED:** IDE schließen → preview/apply → `BACKEND_REQUIRED` in beiden Phasen.

Jeder Schritt notiert das beobachtete Ergebnis (für die finale v2b-PR-/Merge-Beschreibung).

## 4. Datenfluss

```
runIde --args=<fixture>
  → Sandbox-IDE öffnet <fixture> (basePath = <fixture>)
  → Plugin BackendHttpServer schreibt jetbrains-<hash(<fixture>)>.port (Root = <fixture>)
lean-ctx call ctx_refactor --project-root <fixture> --json '{action:rename_preview,…}'
  → ToolContext{project_root=<fixture>} → ctx_refactor::handle
  → handle_rename_refactor → live_jetbrains_backend(<fixture>)
  → read_port_file(<fixture>) matcht → pid_alive + /health
  → HTTP /renamePreview|/renameApply an Plugin
  → preview/apply-Ergebnis → stdout
```

## 5. Testing

- **`lean-ctx call`:** Rust-Unit-Test(s): (a) Dispatch zu einem read-only Tool an einem temp-Root liefert dessen
  Output; (b) unbekanntes Tool → CLI-Fehler ≠0; (c) ungültiges JSON → klarer CLI-Fehler. Keine IDE nötig (ein
  headless-fähiges Tool als Sonde, z.B. ein Read-/Tree-Tool).
- **Fixture-Generator:** optional ein Smoke-Test, dass das Skript die erwarteten Dateien erzeugt (oder bewusst manuell —
  YAGNI).
- **Gate selbst:** manuell via Runbook (das ist der Zweck — nicht automatisierbar ohne laufende IDE).

## 6. Fehlerbehandlung

- `lean-ctx call`: unbekanntes Tool → `error: unknown tool '<x>'` + Exit≠0; JSON-Parse-Fehler → klare Meldung + Exit≠0;
  unsicherer/breiter `--project-root` → Ablehnung (`is_broad_or_unsafe_root`).
- Tool-fachliche Negativfälle (`BACKEND_REQUIRED`, `CONFLICT`, `INDEXING`, `UNSUPPORTED_LANGUAGE`) sind **regulärer
  stdout-Output** mit Exit 0 — sie sind erwartete Gate-Ergebnisse, keine CLI-Fehler.

## 7. Scope / YAGNI

- `lean-ctx call` minimal: `<tool> --project-root --json` (+ evtl. `--json-file`). Kein voller RPC-CLI, keine
  Tool-Liste/Introspektion (existiert via MCP/`profile` bereits).
- Fixture minimal (4 Quell- + 2 Gradle-Dateien). Kein Multi-Modul, kein Java, keine echten Deps.
- **Kein** automatisiertes CI-Gate (Heavy-Fixture/`HeavyPlatformTestCase`) — bewusst ausgeschlossen (eigener, größerer
  Aufwand; hier nicht das Ziel).
- Kein Auto-Launch/Orchestrierung von runIde aus lean-ctx heraus — der Runbook steuert manuell.

## 8. Offene Punkte (im Plan zu fixieren / verifizieren)

1. **`runIde --args`** öffnet in IntelliJ-Platform-Gradle-Plugin 2.x das übergebene Projekt? Falls nicht zuverlässig:
   Fallback dokumentieren (Projekt einmal via `File → Open` öffnen → Sandbox persistiert es) **oder** eine kleine
   `runIde { argumentProviders … }`-Konfiguration ergänzen.
2. **`lean-ctx call`-Dispatch:** Erreicht der CLI-Einstieg die `ctx_refactor`-Handler mit einem injizierten
   `ToolContext`? Genauer Integrationspunkt im bestehenden CLI-Dispatch (`rust/src/cli/…`) + Tool-Registry zu bestimmen.
3. **Kotlin-`jvm`-Version** im Fixture-`build.gradle.kts` kompatibel zur Sandbox-IDE (2026.1.3, gebündeltes
   Kotlin-Plugin) wählen, sodass der Import ohne Sync-Fehler durchläuft.

## 9. Self-Review (Abdeckung)

| Anforderung | Abgedeckt durch |
| Live-Verifikation des vollen v2b-Stacks | §3.1 (`lean-ctx call` → Rust-Gate + Plugin), §4 |
| Sauberer Find-Usages-Scope | §3.2 (dediziertes Kotlin-Gradle-Fixture) |
| Root-Matching gelöst | §3.1 (`--project-root`-Injektion), §4 |
| Reproduzierbarkeit | §3.2 (committeter Generator), §3.3 (committetes Runbook) |
| Alle 9 Gate-Fälle | §3.3 (Preview/Apply/TOCTOU/Konflikt±force/INDEXING/UNSUPPORTED_LANGUAGE/BACKEND_REQUIRED) |
| Testbarkeit der Neuerung | §5 (`lean-ctx call`-Unit-Tests) |
| Sicherheit | §3.1-Notiz + §6 (Root-Validierung, PathJail intakt) |
| YAGNI | §7 (minimaler CLI, minimales Fixture, kein CI/Auto-Launch) |
