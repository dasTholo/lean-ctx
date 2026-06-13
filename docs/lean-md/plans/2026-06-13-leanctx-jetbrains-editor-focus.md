# lean-ctx JetBrains — Editor-Focus-Signal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Das JetBrains-Plugin (`com.leanctx.plugin`) meldet den fokussierten Editor-Pfad an lean-ctx (`lean-ctx editor-signal --file <path>`) — Producer-Parität zum VS-Code-Verhalten von #500, sodass JetBrains-Nutzer das Editor-Focus-Ranking bekommen.

**Architecture:** Eine neue Klasse `EditorFocusReporter` trägt die gesamte neue Logik (Registry-Gate → Filter → Dedup → 2s-Debounce → fire-and-forget Binary-Spawn). Der Kern ist als reine Funktionen über primitive Eingaben faktorisiert (ohne IDE-Plattform-Treiber unit-testbar); Spawn und Debounce-Scheduler sind injizierbar. `LeanCtxStartupActivity` abonniert den `FileEditorManagerListener` auf dem Projekt-MessageBus und meldet die initial offene Datei. `record_focus()` (Rust) bleibt die einzige Signal-Wahrheit → kein Kotlin-Drift. **Keine Rust-Änderung.**

**Tech Stack:** Kotlin, IntelliJ Platform SDK (`Registry`, `Alarm`, `FileEditorManagerListener`, `AppExecutorUtil`), Gradle (`gradlew build`), JUnit 4 (`org.junit.Test`) für reine Logik-Tests.

**Spec:** `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-editor-focus-design.md`

---

## Wichtige Projekt-Constraints (gelten für JEDE Task)

- **Branch:** Arbeit direkt auf `feat-jetbrains-plugin` — **kein worktree** (Projekt-Rule).
- **Commit-Strategie (Spec §10):** **EIN** Commit für die gesamte Phase, **erst nach erfülltem Gate** (Task 8). **Während** der Entwicklung wird **nicht** committet. Die TDD-Schritte unten führen Tests aus, committen aber nicht — der einzige `git commit` ist Task 8.
- **Rust unberührt:** Diese Phase fügt nur einen weiteren Producer hinzu. `editor_signal.rs`, `ctx_preload.rs`, CLI-Ingress, Dashboard bleiben unverändert. Keine `cargo`-Tests, kein `clippy`/`fmt`, kein Schema-Drift-Gate.
- **Kotlin-Editing:** Diese Dateien sind `.kt` — laut Projekt-Rule sind `.rs`-Dateien Serena-pflichtig, `.kt`/`.xml`/`.md` **nicht**. Native `Write`/`Edit` für die Kotlin-/XML-/Markdown-Dateien sind erlaubt.
- **Vor `git add` (Task 8):** `mcp__jetbrains__reformat_file` auf jede geänderte/neue `.kt`- und `.xml`-Datei (Projekt-Rule).

---

## File Structure

| Datei | Verantwortung | Änderung |
|---|---|---|
| `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt` | Die **einzige** neue Logik: Registry-Gate, Filter, Dedup, Debounce-Scheduler, fire-and-forget Spawn. Kern als reine Funktionen über primitive Eingaben (testbar); Plattform-Abhängigkeiten injizierbar. | **NEU** |
| `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt` | Abonniert `FileEditorManagerListener` auf dem Projekt-MessageBus, erzeugt einen `EditorFocusReporter` pro Projekt, meldet die initial offene Datei. | ~ Verdrahtung |
| `packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml` | Registriert den Opt-out-Registry-Key `leanctx.editor.signal.enabled` (default `true`). | + `<registryKey>` |
| `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt` | Unit-Tests für Filter, Dedup, Registry-Gate, basePath-Grenze (reine JUnit4, kein Plattform-Treiber). | **NEU** |
| `docs/lean-md/runbooks/runide-editor-focus-gate.md` | Manuelles runIde-Akzeptanz-Gate (Liefergegenstand). | **NEU** |

**Decomposition-Begründung:** Der Reporter-Kern (`isUnderBasePath`, `maybeReport`) arbeitet auf primitiven Werten (`Boolean`, `String`) statt auf `VirtualFile`/`Registry`, damit er ohne `BasePlatformTestCase` (langsamer IDE-Treiber) unit-testbar ist — genau wie der bestehende `StatsReaderTest`. Der dünne `onFileFocused(VirtualFile)`-Adapter, der Default-Spawn und der Alarm-Scheduler sind Plattform-gebunden und werden im manuellen runIde-Gate (Task 7) abgedeckt, nicht im Unit-Test.

---

## Referenzen (vor dem Coden lesen)

- **Referenz-Producer (VS Code, 1:1 zu spiegeln):** `vscode-extension/src/editor-signal.ts` — `DEBOUNCE_MS = 2000`, `lastSent`-Dedup, `isWorkspaceFile` (= `startsWith(folder)`), Filter `scheme === "file"`, fire-and-forget `.catch()`, initial-report beim Aktivieren.
- **Bestehendes Spawn-Muster:** `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/BinaryResolver.kt` — `resolve(): String?` (gecacht; `null` wenn kein Binary), `runCommand(...)` (blockierend, **nicht** für fire-and-forget nutzen — wir spawnen leichtgewichtig selbst).
- **Bestehender Startup-Hook:** `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt` — `execute(project)` ruft bereits `BinaryResolver.resolve()` + `startBackend(project)`; `project` ist ein `Disposable` (für MessageBus-Connection + Alarm-Parent).
- **Test-Muster (reine JUnit4):** `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/StatsReaderTest.kt` — `@Test`-Methoden, `org.junit.Assert.assertEquals`, **kein** `BasePlatformTestCase`.
- **Runbook-Muster:** `docs/lean-md/runbooks/runide-rename-gate.md`.

---

## Task 1: `EditorFocusReporter` — basePath-Grenz-Funktion (pure, TDD)

Start mit der kleinsten reinen Funktion: gehört ein Pfad unter `basePath`? VS Code nutzt `fsPath.startsWith(folder)`; wir härten die Pfad-Grenze minimal (`/`-Segment), damit `/foo/bar2` **nicht** als unter `/foo/bar` gilt.

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`
- Create (Test): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt`

- [ ] **Step 1: Failing-Test schreiben** — `EditorFocusReporterTest.kt` anlegen:

```kotlin
package com.leanctx.plugin

import com.leanctx.plugin.EditorFocusReporter.Companion.isUnderBasePath
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class EditorFocusReporterTest {

    @Test
    fun fileUnderBasePathIsAccepted() {
        assertTrue(isUnderBasePath("/home/me/proj/src/Main.kt", "/home/me/proj"))
    }

    @Test
    fun basePathItselfIsAccepted() {
        assertTrue(isUnderBasePath("/home/me/proj", "/home/me/proj"))
    }

    @Test
    fun siblingWithSharedPrefixIsRejected() {
        // /home/me/proj2 must NOT count as being under /home/me/proj
        assertFalse(isUnderBasePath("/home/me/proj2/Main.kt", "/home/me/proj"))
    }

    @Test
    fun nullBasePathIsRejected() {
        assertFalse(isUnderBasePath("/home/me/proj/Main.kt", null))
    }

    @Test
    fun outsideBasePathIsRejected() {
        assertFalse(isUnderBasePath("/tmp/other/Main.kt", "/home/me/proj"))
    }
}
```

- [ ] **Step 2: Test ausführen, Fehlschlag verifizieren**

Run (bare command, cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew test --tests "com.leanctx.plugin.EditorFocusReporterTest"
```
Erwartung: **Compile-Fehler** — `EditorFocusReporter` (und `isUnderBasePath`) existiert noch nicht.

- [ ] **Step 3: Minimale Implementierung** — `EditorFocusReporter.kt` anlegen (nur die Companion-Funktion; der Rest folgt in Task 2/3):

```kotlin
package com.leanctx.plugin

/**
 * Editor focus signal (#500), JetBrains producer side. Reports the focused file
 * path to lean-ctx via `lean-ctx editor-signal --file <path>` so the context
 * engine ranks it up. Paths only — never content — and only files inside the
 * current project. 1:1 parity with vscode-extension/src/editor-signal.ts.
 *
 * The core (isUnderBasePath / maybeReport) operates on primitives so it is unit
 * testable without an IDE platform driver. The VirtualFile adapter, the spawn,
 * and the debounce Alarm are platform-bound and covered by the manual runIde gate.
 */
class EditorFocusReporter {

    companion object {
        /** Debounce window, identical to VS Code's DEBOUNCE_MS. */
        const val DEBOUNCE_MS = 2_000

        /**
         * True iff [path] is [basePath] itself or sits under it on a path
         * boundary. VS Code uses a plain startsWith; we additionally require a
         * '/' segment boundary so /foo/bar2 is not treated as under /foo/bar.
         */
        fun isUnderBasePath(path: String, basePath: String?): Boolean {
            if (basePath.isNullOrEmpty()) return false
            return path == basePath || path.startsWith("$basePath/")
        }
    }
}
```

- [ ] **Step 4: Test ausführen, Erfolg verifizieren**

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew test --tests "com.leanctx.plugin.EditorFocusReporterTest"
```
Erwartung: **PASS** (5 Tests grün).

---

## Task 2: `EditorFocusReporter` — Filter, Registry-Gate, Dedup (injizierbar, TDD)

Jetzt der testbare Kern `maybeReport(isLocal, isDirectory, path)`: Registry-Gate, Filter (lokal & keine Directory & unter basePath), Pfad-Dedup, dann Scheduler→Spawn. Scheduler/Spawn/Registry-Gate sind injizierbar, sodass der Test ohne Plattform läuft (synchroner Scheduler + capturing Spawn).

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`
- Modify (Test): `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt`

- [ ] **Step 1: Failing-Tests ergänzen** — folgende Methoden + Helper in `EditorFocusReporterTest` einfügen (innerhalb der Klasse, nach den Task-1-Tests). Der `newReporter`-Helper konstruiert einen Reporter mit synchronem Scheduler (`schedule { it() }` = sofort) und einer Capturing-Spawn-Liste:

```kotlin
    // --- Helpers for the injectable core (no IDE platform needed) ---

    private val spawned = mutableListOf<String>()

    private fun newReporter(
        basePath: String? = "/home/me/proj",
        enabled: Boolean = true,
    ): EditorFocusReporter {
        spawned.clear()
        return EditorFocusReporter(
            parentDisposable = com.intellij.openapi.util.Disposer.newDisposable(),
            basePath = basePath,
            isEnabled = { enabled },
            spawn = { path -> spawned.add(path) },
            schedule = { action -> action() }, // run synchronously, bypass the Alarm
        )
    }

    @Test
    fun localProjectFileTriggersOneSpawn() {
        val reporter = newReporter()
        reporter.maybeReport(isLocal = true, isDirectory = false, path = "/home/me/proj/A.kt")
        assertEquals(listOf("/home/me/proj/A.kt"), spawned)
    }

    @Test
    fun directoryIsRejected() {
        val reporter = newReporter()
        reporter.maybeReport(isLocal = true, isDirectory = true, path = "/home/me/proj/sub")
        assertTrue(spawned.isEmpty())
    }

    @Test
    fun nonLocalFileIsRejected() {
        val reporter = newReporter()
        reporter.maybeReport(isLocal = false, isDirectory = false, path = "/home/me/proj/A.kt")
        assertTrue(spawned.isEmpty())
    }

    @Test
    fun fileOutsideProjectIsRejected() {
        val reporter = newReporter()
        reporter.maybeReport(isLocal = true, isDirectory = false, path = "/tmp/other/A.kt")
        assertTrue(spawned.isEmpty())
    }

    @Test
    fun samePathTwiceDedupsToOneSpawn() {
        val reporter = newReporter()
        reporter.maybeReport(isLocal = true, isDirectory = false, path = "/home/me/proj/A.kt")
        reporter.maybeReport(isLocal = true, isDirectory = false, path = "/home/me/proj/A.kt")
        assertEquals(listOf("/home/me/proj/A.kt"), spawned)
    }

    @Test
    fun registryDisabledSuppressesSpawn() {
        val reporter = newReporter(enabled = false)
        reporter.maybeReport(isLocal = true, isDirectory = false, path = "/home/me/proj/A.kt")
        assertTrue(spawned.isEmpty())
    }
```

Den Import-Block oben in der Datei um `assertEquals` ergänzen:
```kotlin
import org.junit.Assert.assertEquals
```

- [ ] **Step 2: Test ausführen, Fehlschlag verifizieren**

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew test --tests "com.leanctx.plugin.EditorFocusReporterTest"
```
Erwartung: **Compile-Fehler** — der `EditorFocusReporter`-Konstruktor (mit `parentDisposable`/`basePath`/`isEnabled`/`spawn`/`schedule`) und `maybeReport(...)` existieren noch nicht.

- [ ] **Step 3: Implementierung** — `EditorFocusReporter.kt` so erweitern, dass die Klasse den injizierbaren Konstruktor + `maybeReport` bekommt. Die Datei sieht danach komplett so aus:

```kotlin
package com.leanctx.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.util.registry.Registry
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.util.Alarm

/**
 * Editor focus signal (#500), JetBrains producer side. Reports the focused file
 * path to lean-ctx via `lean-ctx editor-signal --file <path>` so the context
 * engine ranks it up. Paths only — never content — and only files inside the
 * current project. 1:1 parity with vscode-extension/src/editor-signal.ts.
 *
 * The core (isUnderBasePath / maybeReport) operates on primitives so it is unit
 * testable without an IDE platform driver. The VirtualFile adapter, the spawn,
 * and the debounce Alarm are platform-bound and covered by the manual runIde gate.
 *
 * @param parentDisposable project-scoped disposable; the debounce Alarm is bound
 *   to it so it is cancelled on project close (no leak, no spawn after close).
 * @param basePath the project base path; files outside it are not reported.
 * @param isEnabled producer-side opt-out gate (registry key, evaluated per event).
 * @param spawn fire-and-forget binary call; injectable for tests.
 * @param schedule debounce scheduler; null uses a 2s POOLED_THREAD Alarm.
 *   Injectable for tests (e.g. synchronous `{ it() }`).
 */
class EditorFocusReporter(
    parentDisposable: Disposable,
    private val basePath: String?,
    private val isEnabled: () -> Boolean = { Registry.`is`("leanctx.editor.signal.enabled", true) },
    private val spawn: (String) -> Unit = ::defaultSpawn,
    schedule: ((() -> Unit) -> Unit)? = null,
) {
    private var lastSent: String? = null

    /** Debounce: cancel any pending request and (re)schedule, collapsing rapid tab hops. */
    private val schedule: (() -> Unit) -> Unit = schedule ?: run {
        val alarm = Alarm(Alarm.ThreadToUse.POOLED_THREAD, parentDisposable)
        { action -> alarm.cancelAllRequests(); alarm.addRequest(action, DEBOUNCE_MS) }
    }

    /** Thin platform adapter: extract primitives from the VirtualFile, then delegate. */
    fun onFileFocused(file: VirtualFile?) {
        if (file == null) return
        maybeReport(file.isInLocalFileSystem, file.isDirectory, file.path)
    }

    /**
     * Core decision, testable without a VirtualFile/Registry/Alarm:
     * registry gate → real-local-project-file filter → path dedup → debounced spawn.
     */
    internal fun maybeReport(isLocal: Boolean, isDirectory: Boolean, path: String) {
        if (!isEnabled()) return
        if (!isLocal || isDirectory) return
        if (!isUnderBasePath(path, basePath)) return
        // Dedup before debounce: same path back-to-back schedules at most one spawn.
        if (path == lastSent) return
        lastSent = path
        schedule { spawn(path) }
    }

    companion object {
        /** Debounce window, identical to VS Code's DEBOUNCE_MS. */
        const val DEBOUNCE_MS = 2_000

        /**
         * True iff [path] is [basePath] itself or sits under it on a path
         * boundary. VS Code uses a plain startsWith; we additionally require a
         * '/' segment boundary so /foo/bar2 is not treated as under /foo/bar.
         */
        fun isUnderBasePath(path: String, basePath: String?): Boolean {
            if (basePath.isNullOrEmpty()) return false
            return path == basePath || path.startsWith("$basePath/")
        }
    }
}
```

> Hinweis: `::defaultSpawn` referenziert eine top-level Datei-private Funktion, die in **Task 3** angelegt wird. Bis dahin schlägt der Compile fehl — **das ist erwartet**; deshalb wird in diesem Step nur die Logik geschrieben und in Task 3 sofort der Spawn nachgezogen. Wer Task 2 isoliert grün sehen will, kommentiert in der Default-Parameterliste `spawn` temporär nicht aus, sondern legt zuerst den `defaultSpawn`-Stub aus Task 3 Step 3 an. **Empfehlung: Task 2 Step 3 und Task 3 Step 3 zusammen anwenden, dann Step 4 beider Tasks fahren.**

- [ ] **Step 4: Test ausführen, Erfolg verifizieren** (nach Anlegen von `defaultSpawn`, siehe Task 3)

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew test --tests "com.leanctx.plugin.EditorFocusReporterTest"
```
Erwartung: **PASS** (11 Tests grün: 5 aus Task 1 + 6 neue).

---

## Task 3: `defaultSpawn` — fire-and-forget Binary-Call (Plattform, kein Unit-Test)

Der Produktions-Spawn: Registry-Gate hat bereits gegriffen (in `maybeReport`); hier nur Binary auflösen und `lean-ctx editor-signal --file <path>` auf einem Pool-Thread feuern — **nie** auf dem EDT, Exceptions geschluckt. Nicht unit-getestet (Plattform/Prozess); abgedeckt im runIde-Gate (Task 7).

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`

- [ ] **Step 1: Implementierung** — am Ende von `EditorFocusReporter.kt` (außerhalb der Klasse, top-level Datei-privat) die Spawn-Funktion + Imports einfügen.

Imports oben in der Datei ergänzen:
```kotlin
import com.intellij.util.concurrency.AppExecutorUtil
import java.util.concurrent.TimeUnit
```

Top-level Funktion am Dateiende (nach der Klasse):
```kotlin
/**
 * Fire-and-forget producer call. Runs on a pooled thread (never the EDT). A lost
 * signal is harmless: the next tab change resends. A binary that is missing or
 * too old (no `editor-signal` subcommand) is swallowed silently, mirroring VS
 * Code's `.catch()`. We waitFor with a short timeout only to reap the short-lived
 * child, never to block UI.
 */
private fun defaultSpawn(path: String) {
    val binary = BinaryResolver.resolve() ?: return
    AppExecutorUtil.getAppExecutorService().execute {
        try {
            val process = ProcessBuilder(binary, "editor-signal", "--file", path)
                .redirectErrorStream(true)
                .start()
            process.waitFor(5, TimeUnit.SECONDS)
        } catch (_: Exception) {
            // missing/old binary or IO error — a lost signal is harmless
        }
    }
}
```

- [ ] **Step 2: Test ausführen, Erfolg verifizieren** (gemeinsam mit Task 2 Step 4)

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew test --tests "com.leanctx.plugin.EditorFocusReporterTest"
```
Erwartung: **PASS** (11 Tests grün). `defaultSpawn` selbst ist nicht unit-getestet (wird im Gate verifiziert), aber der Code muss compilieren und die injizierten Tests müssen grün sein.

---

## Task 4: `plugin.xml` — Registry-Key Opt-out

Registriert `leanctx.editor.signal.enabled` (default `true`), den `maybeReport` per `Registry.is(...)` auswertet. Kein `PersistentStateComponent`/`Configurable` (Spec §2/§8).

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml`

- [ ] **Step 1: `<registryKey>` einfügen** — im Block `<extensions defaultExtensionNs="com.intellij">`, direkt nach der `<postStartupActivity .../>`-Zeile:

```xml
        <registryKey key="leanctx.editor.signal.enabled"
                     defaultValue="true"
                     description="Report the focused editor file path to lean-ctx for context ranking (#500). Path only, never content. Disable to opt out."/>
```

Der Block sieht danach so aus:
```xml
    <extensions defaultExtensionNs="com.intellij">
        <statusBarWidgetFactory
                implementation="com.leanctx.plugin.LeanCtxStatusBarFactory"
                id="com.leanctx.statusBar"
                order="after encodingWidget"/>
        <postStartupActivity implementation="com.leanctx.plugin.LeanCtxStartupActivity"/>
        <registryKey key="leanctx.editor.signal.enabled"
                     defaultValue="true"
                     description="Report the focused editor file path to lean-ctx for context ranking (#500). Path only, never content. Disable to opt out."/>
    </extensions>
```

- [ ] **Step 2: Build verifizieren** (plugin.xml wird beim Build validiert)

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew buildPlugin
```
Erwartung: **BUILD SUCCESSFUL** — die `plugin.xml` validiert (kein „unknown extension"-Fehler für `registryKey`).

---

## Task 5: `LeanCtxStartupActivity` — Listener-Verdrahtung + Initial-Melden

Abonniert beim Projekt-Start `selectionChanged` auf dem Projekt-MessageBus und meldet die initial offene Datei. `project` dient als Disposable (MessageBus-Connection + Alarm-Parent) → automatisches Cleanup bei Projekt-Schließung.

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt`

- [ ] **Step 1: Imports ergänzen** — zu den bestehenden Imports hinzufügen:

```kotlin
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerEvent
import com.intellij.openapi.fileEditor.FileEditorManagerListener
```

- [ ] **Step 2: `startEditorFocus` aufrufen** — in `execute(project)` nach `startBackend(project)`:

```kotlin
        startBackend(project)
        startEditorFocus(project)
```

- [ ] **Step 3: `startEditorFocus`-Methode hinzufügen** — als neue private Methode in der Klasse (z.B. nach `startBackend`):

```kotlin
    /**
     * Wire the editor-focus producer (#500): subscribe to tab-selection changes on
     * the project message bus and report the file that is already open. The reporter,
     * its debounce Alarm, and the bus connection are all bound to `project` (a
     * Disposable) → cleaned up on project close. Failures must never break the IDE.
     */
    private fun startEditorFocus(project: Project) {
        try {
            val reporter = EditorFocusReporter(parentDisposable = project, basePath = project.basePath)
            project.messageBus.connect(project).subscribe(
                FileEditorManagerListener.FILE_EDITOR_MANAGER,
                object : FileEditorManagerListener {
                    override fun selectionChanged(event: FileEditorManagerEvent) {
                        reporter.onFileFocused(event.newFile)
                    }
                }
            )
            // Report the file that is already open when the activity runs.
            reporter.onFileFocused(FileEditorManager.getInstance(project).selectedFiles.firstOrNull())
        } catch (e: Exception) {
            log.warn("lean-ctx editor-focus reporter failed to start", e)
        }
    }
```

- [ ] **Step 4: Build + bestehende Tests verifizieren**

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew build
```
Erwartung: **BUILD SUCCESSFUL** — kompiliert, `EditorFocusReporterTest` (11) + `StatsReaderTest` + `PortFileHygieneTest` grün.

---

## Task 6: Runbook — manuelles runIde-Editor-Focus-Gate (Liefergegenstand)

Schreibt das manuelle Akzeptanz-Gate (Spec §6) als Runbook, Muster wie `runide-rename-gate.md`.

**Files:**
- Create: `docs/lean-md/runbooks/runide-editor-focus-gate.md`

- [ ] **Step 1: Runbook anlegen** mit folgendem Inhalt:

```markdown
# Runbook: runIde-Editor-Focus-Gate (#500-Producer-Parität, Live-Verifikation)

Verifiziert den JetBrains-Producer für #500 live: das Plugin meldet den
fokussierten Editor-Pfad via `lean-ctx editor-signal --file <path>`, sodass
`~/.lean-ctx/editor_signal.json` und die Dashboard-„Editor focus"-Kachel den
aktiven File spiegeln — 1:1 zum VS-Code-Verhalten.

Bezug: Spec `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-editor-focus-design.md`.

## Voraussetzungen
- `lean-ctx` gebaut/installiert mit `editor-signal`-Subcommand (3.8.3+):
  `lean-ctx editor-signal --help` zeigt `--file`.
- Plugin-Modul: `packages/jetbrains-lean-ctx`.
- Ein Test-Projekt mit mindestens zwei Dateien A und B.

## 1. Launch — Sandbox-IDE
```
./gradlew runIde
```
(cwd=`packages/jetbrains-lean-ctx`)
Test-Projekt öffnen, **Indizierung abwarten** (Statusleiste idle).

## 2. Gate-Checks

| # | Schritt | Soll-Ergebnis |
| 1 | Datei A öffnen, ~2s warten | `~/.lean-ctx/editor_signal.json` → `active_file` endet auf A |
| 2 | Datei B öffnen, ~2s warten | `active_file` → B; A taucht in `recent_files` auf |
| 3 | Dashboard öffnen (`lean-ctx dashboard`), „Editor focus"-Kachel | zeigt B als frisch (innerhalb Freshness-Fenster 120s) |
| 4 | `leanctx.editor.signal.enabled = false` setzen (Sandbox: `Help → Find Action → Registry…`), dann A↔B wechseln, ~2s warten | `editor_signal.json` **ändert sich nicht** (kein neues Signal) |
| 5 | Registry-Key wieder `true`, Binary-Pfad temporär unauffindbar machen (z.B. `lean-ctx` aus PATH/Standardorten entfernen) oder mit fehlendem Binary starten, Tab wechseln | **kein Crash**, IDE bleibt stabil, keine Fehler-Notification-Flut |

`editor_signal.json` inspizieren:
```
cat ~/.lean-ctx/editor_signal.json
```
(Felder: `active_file`, `recent_files[(path, ts)]`, `updated_at`.)

## 3. Teardown
- Sandbox-IDE schließen (Alarm + MessageBus-Connection werden disposed).
- `~/.lean-ctx/editor_signal.json` darf liegen bleiben (globaler Ranking-Hinweis).

## Notizen für die PR-/Merge-Beschreibung
- Beobachtete Werte aus #1–#3 notieren (Beleg der Producer-Parität).
- Bekannte #500-Grenze (Spec §5): mehrere IDE-Fenster → globale Datei,
  last-write-wins (gilt identisch für VS Code, kein JetBrains-Regress).
```

- [ ] **Step 2: Existenz verifizieren**

Run (cwd=`/home/tholo/Scripts/lean-ctx`):
```
test -f docs/lean-md/runbooks/runide-editor-focus-gate.md && echo OK
```
Erwartung: `OK`.

---

## Task 7: Manuelles runIde-Gate fahren (Akzeptanz)

> **Hinweis:** Dieser Schritt ist **manuell/interaktiv** (Sandbox-IDE) und kann von einem autonomen Subagenten **nicht** abgeschlossen werden. Wenn die Ausführung subagent-getrieben läuft: hier an den Menschen übergeben (Status `NEEDS_CONTEXT`/Gate-Pause), das Runbook abarbeiten und die Beobachtungen für die Commit-/PR-Beschreibung festhalten.

**Files:** keine (Verifikation).

- [ ] **Step 1:** `docs/lean-md/runbooks/runide-editor-focus-gate.md` Schritt für Schritt durchführen.
- [ ] **Step 2:** Ergebnisse von Check #1–#5 notieren (für die Commit-Message in Task 8).
- [ ] **Step 3:** Bei Abweichung → `superpowers:systematic-debugging`, **nicht** committen, bis das Gate grün ist.

---

## Task 8: Gate, Reformat & Commit (EIN Commit, Spec §10)

Finales Gradle-Gate, dann Reformat aller geänderten Dateien (Projekt-Rule), dann **ein** Commit der gesamten Phase.

**Files:** alle obigen.

- [ ] **Step 1: Voll-Build-Gate**

Run (cwd=`packages/jetbrains-lean-ctx`):
```
./gradlew build
```
Erwartung: **BUILD SUCCESSFUL**; alle Tests grün (`EditorFocusReporterTest` 11, `StatsReaderTest`, `PortFileHygieneTest`).

- [ ] **Step 2: Reformat jeder geänderten/neuen Datei** (Projekt-Rule, vor `git add`)

Via `mcp__jetbrains__reformat_file` auf:
- `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt`
- `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt`
- `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt`
- `packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml`

(Falls `mcp__jetbrains__reformat_file` deferred ist: zuerst `ToolSearch(query="select:mcp__jetbrains__reformat_file")`.)

- [ ] **Step 3: Geänderten Stand prüfen**

Run (bare command, cwd=`/home/tholo/Scripts/lean-ctx`):
```
git status --short
```
Erwartung: 2 neue `.kt`, 1 neuer Test, 1 geänderte `.kt`, 1 geänderte `plugin.xml`, 1 neues Runbook.

- [ ] **Step 4: Stagen & committen** (EIN Commit für die Phase, erst nach grünem Gate)

Run (bare commands, cwd=`/home/tholo/Scripts/lean-ctx`):
```
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/EditorFocusReporter.kt packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/EditorFocusReporterTest.kt packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml docs/lean-md/runbooks/runide-editor-focus-gate.md docs/lean-md/plans/2026-06-13-leanctx-jetbrains-editor-focus.md
```
```
git commit -m "feat(jetbrains): editor-focus reporter (#500 producer parity)

JetBrains plugin reports the focused editor path via
`lean-ctx editor-signal --file <path>`, mirroring the VS Code producer.
EditorFocusReporter: registry gate (leanctx.editor.signal.enabled) →
local-project-file filter → path dedup → 2s debounce → fire-and-forget spawn.
Wired in LeanCtxStartupActivity via FileEditorManagerListener + initial report.
No Rust change (record_focus stays the only signal truth). runIde gate: runbook.

Tests: EditorFocusReporterTest (filter/dedup/registry/basePath), gradle build green."
```
Erwartung: Commit auf `feat-jetbrains-plugin` angelegt.

---

## Self-Review (vom Plan-Autor durchgeführt)

**1. Spec-Abdeckung:**
- §3 `EditorFocusReporter.kt` NEU → Task 1–3. ✓
- §3 `LeanCtxStartupActivity.kt` Listener + Initial-Melden → Task 5. ✓
- §3 `plugin.xml` `<registryKey>` → Task 4. ✓
- §4 Datenfluss (Registry → Filter → Dedup → Alarm → Spawn → record_focus) → Task 2 (`maybeReport`) + Task 3 (`defaultSpawn`). ✓
- §4.1 Filter (`isInLocalFileSystem`, `!isDirectory`, unter `basePath`, Pfad-only) → Task 2 `maybeReport`-Tests + `isUnderBasePath`. ✓
- §5 EDT-Sicherheit (Pool-Thread, nie EDT), `resolve()==null` still inert, Spawn-Exception geschluckt → Task 3 `defaultSpawn`. ✓
- §5 Lifecycle (Alarm an Projekt-Disposable) → Task 2 Konstruktor `parentDisposable` + Task 5 `project` als Parent. ✓
- §6 Kotlin-Unit-Tests (Filter/Dedup/Registry-Gate) → Task 1+2 (11 Tests). ✓ Debounce-Kollaps bewusst manuell im Gate (Spec erlaubt) → Task 7 Runbook #4-Umfeld. ✓
- §6 Runbook → Task 6; Gate fahren → Task 7. ✓
- §6 `gradlew build` grün, keine Rust-Tests → Task 8 Step 1 + Constraints. ✓
- §10 EIN Commit pro Phase nach Gate, kein worktree, reformat vor add → Task 8 + Constraints. ✓

**2. Placeholder-Scan:** Kein TBD/TODO; jeder Code-Step zeigt vollständigen Code; Test-Steps zeigen exakten Test-Code + erwartete Ausgabe. ✓

**3. Typ-Konsistenz:** `isUnderBasePath(path: String, basePath: String?)`, `maybeReport(isLocal: Boolean, isDirectory: Boolean, path: String)`, `onFileFocused(file: VirtualFile?)`, Konstruktor-Parameter (`parentDisposable`, `basePath`, `isEnabled`, `spawn`, `schedule`), `DEBOUNCE_MS`, `defaultSpawn(path: String)` — über Task 1/2/3/5 und die Test-Helper konsistent benannt. ✓
- Bekannte Reihenfolge-Falle: Task 2 Step 3 referenziert `::defaultSpawn` aus Task 3 Step 1 → im Plan explizit als „zusammen anwenden" markiert (Task 2 Step 3 Hinweis). ✓
```
