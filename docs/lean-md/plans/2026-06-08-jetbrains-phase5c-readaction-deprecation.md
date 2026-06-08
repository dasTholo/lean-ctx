# Phase 5c — `ReadAction.compute` → `runReadAction` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Die deprecated statische `ReadAction.compute` in `PsiLocator.inSmartReadAction` durch die nicht-deprecated Kotlin-Top-Level-Funktion `runReadAction` ersetzen, ohne Semantik-Drift.

**Architecture:** Ein einziger Chokepoint (`PsiLocator.inSmartReadAction`) kapselt jeden PSI-Read aller Endpoints (nav/hierarchy/overview/inspections). Der Fix ist auf diese eine Methode + den zugehörigen Import begrenzt; der `DumbService.isDumb`-Check (INDEXING) bleibt unverändert. Verifikation über bestehende PSI-Unit-Tests (`BasePlatformTestCase`, rufen `inSmartReadAction` direkt) plus ein runIde-Live-Gate.

**Tech Stack:** Kotlin, IntelliJ Platform SDK (IC-2026.1.3, `sinceBuild=261`), Gradle (`org.jetbrains.intellij.platform`), JUnit3-Style `BasePlatformTestCase`.

---

## File Structure

- **Modify:** `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt`
  - Import-Block (Z. 3): `ReadAction` → `runReadAction`.
  - Methode `inSmartReadAction` (Z. 75–80): Body-Zeile 79.
  - **Keine** weiteren Dateien. Aufrufer/Handler bleiben unangetastet (Signatur unverändert).

**Edit-Regel:** `*.kt` → Serena-Tools (`mcp__serena__replace_content`), kein nativer `Edit`/`ctx_edit`. Vor `git add`: `mcp__jetbrains__reformat_file` auf `PsiLocator.kt`.

**Working dir für `ctx_shell`:** `cwd=/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx` (eigenes `gradlew`). Bare command, kein `cd … &&`, kein `2>&1`, kein `| tail` bei Test-Runnern.

---

## Task 0: Spike — Ersatz-API gegen IC-2026.1.3 verifizieren (Risiko-Gate)

**Files:** keine (read-only Verifikation).

- [ ] **Step 1: `runReadAction`-Deprecation-Status prüfen**

Symbol-Info aus der laufenden IDE holen. Falls deferred: `ToolSearch(query="select:mcp__jetbrains__get_symbol_info")` zuerst.

```
mcp__jetbrains__get_symbol_info(
  filePath="packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt",
  line=79, column=27,
  projectPath="/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx"
)
```

Alternativ Plattform-Quelle lesen: `runReadAction` ist in
`com.intellij.openapi.application.actions` / `Application` definiert.

- [ ] **Step 2: Entscheidung dokumentieren**

Expected: `runReadAction` ist **nicht** `@Deprecated` in IC-2026.1.3 → Plan fährt mit Task 1 fort (Variante A).

**Fallback (Variante B), nur falls `runReadAction` doch deprecated:** statt `runReadAction { body() }` →
`ReadAction.nonBlocking<T> { body() }.executeSynchronously()` (Import `ReadAction` bleibt dann erhalten, kein `runReadAction`-Import). Alle weiteren Tasks gelten analog mit dieser Zeile.

- [ ] **Step 3: Befund festhalten**

```
ctx_knowledge action=remember category=decision content="Phase5c T0: runReadAction non-deprecated@IC-2026.1.3 → Variante A. (sonst nonBlocking().executeSynchronously())"
```

---

## Task 1: Migration — `PsiLocator.inSmartReadAction`

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt:3` (Import)
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt:79` (Body)

- [ ] **Step 1: Import anpassen (Serena `replace_content`)**

```
mcp__serena__replace_content(
  relative_path="packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt",
  content_to_replace="import com.intellij.openapi.application.ReadAction",
  new_content="import com.intellij.openapi.application.runReadAction"
)
```

- [ ] **Step 2: Body-Zeile 79 anpassen (Serena `replace_content`)**

```
mcp__serena__replace_content(
  relative_path="packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt",
  content_to_replace="        return ReadAction.compute<T, RuntimeException> { body() }",
  new_content="        return runReadAction { body() }"
)
```

Zielzustand der Methode:

```kotlin
fun <T> inSmartReadAction(body: () -> T): T {
    if (DumbService.getInstance(project).isDumb) {
        throw BackendException("INDEXING", "IDE is indexing; retry shortly")
    }
    return runReadAction { body() }
}
```

- [ ] **Step 3: Edit verifizieren (changed lines)**

```
ctx_delta path="packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt"
```

Expected: nur Import-Zeile + Body-Zeile geändert; kein verbliebenes `ReadAction.compute`, kein verbliebener `ReadAction`-Import (Variante A).

---

## Task 2: Verifikation

**Files:** keine (build + test + live gate).

- [ ] **Step 1: Warning-frei kompilieren (Warning-as-Evidence)**

```
ctx_shell command="./gradlew compileKotlin" cwd="/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx"
```

Expected: `BUILD SUCCESSFUL`, **keine** `w: PsiLocator.kt:79 … is deprecated`-Zeile mehr im Output.

- [ ] **Step 2: PSI-Unit-Tests grün**

```
ctx_shell command="./gradlew test" cwd="/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx"
```

Expected: `BUILD SUCCESSFUL`. Abgedeckt: `DefinitionResolverTest` (inkl. `testNoSymbolThrows` → BackendException-Propagation durch `inSmartReadAction`), `ReferenceFinderTest`, `ImplementationFinderTest`, `TypeHierarchyResolverTest`.

- [ ] **Step 3: runIde-Live-Gate (wie Phase-5b T11)**

Plugin läuft gegen die geöffnete IDE (Port wie in Session E1, Projekt `packages/jetbrains-lean-ctx`). Einen PSI-Endpoint live aufrufen, z. B. `symbols_overview` oder `type_hierarchy` auf eine bekannte Datei, und einen 200-Erfolg bestätigen. Damit ist der `runReadAction`-Pfad im echten Plugin (off-EDT, smart mode) belegt.

Expected: erfolgreiche Antwort eines PSI-Endpoints; kein Read-Lock-/Threading-Fehler. INDEXING-Pfad bleibt durch `testNoSymbolThrows`-Analogie + unveränderten `DumbService`-Check abgedeckt.

- [ ] **Step 4: Evidenz festhalten**

```
ctx_knowledge action=remember category=decision content="Phase5c verifiziert: compileKotlin warning-frei, ./gradlew test grün, runIde-Endpoint 200 → runReadAction OK."
```

---

## Task 3: Commit

**Files:** `PsiLocator.kt` (+ ggf. der bereits committete Spec/Plan).

- [ ] **Step 1: Datei reformatieren (Projektregel, vor `git add`)**

```
mcp__jetbrains__reformat_file(
  path="/home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt"
)
```

- [ ] **Step 2: Staged-Diff prüfen**

```
ctx_shell command="git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt" cwd="/home/tholo/Scripts/lean-ctx"
ctx_shell command="git diff --cached --stat" cwd="/home/tholo/Scripts/lean-ctx"
```

Expected: genau **eine** geänderte Source-Datei (`PsiLocator.kt`).

- [ ] **Step 3: Commit (EIN scoped Commit)**

```
ctx_shell command="git commit -m 'fix(jetbrains): migrate deprecated ReadAction.compute to runReadAction (Phase 5c)'" cwd="/home/tholo/Scripts/lean-ctx"
```

Expected: ein Commit mit nur `PsiLocator.kt`.

---

## Definition of Done

- [ ] Deprecation-Warning an `PsiLocator.kt:79` verschwunden (Step 2.1 Output).
- [ ] `./gradlew compileKotlin` + `./gradlew test` grün.
- [ ] runIde-Smoke: ein PSI-Endpoint live erfolgreich.
- [ ] Genau ein scoped Commit, `PsiLocator.kt` vorher reformatiert.
- [ ] Kein verbliebenes `ReadAction.compute` / (Variante A) kein `ReadAction`-Import im Modul.
