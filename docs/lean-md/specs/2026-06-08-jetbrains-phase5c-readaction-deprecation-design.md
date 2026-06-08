# Phase 5c — Deprecation-Fix: `ReadAction.compute` → `runReadAction`

- **Datum:** 2026-06-08
- **Branch:** feat-jetbrains-plugin
- **Plattform:** IntelliJ IDEA 2026.1.3, `sinceBuild=261`, Kotlin-Sources
- **Status:** Design (genehmigt)

## Problem

`PsiLocator.inSmartReadAction` (`packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/PsiLocator.kt:79`)
nutzt das deprecated statische `ReadAction.compute<T, E>(ThrowableComputable<T, E>): T`:

```kotlin
return ReadAction.compute<T, RuntimeException> { body() }
```

IDE-Warning:

```
w: PsiLocator.kt:79:27 'static fun <T : Any!, E : Throwable!> compute(
   p0: ThrowableComputable<T!, E!>): T!' is deprecated. Deprecated in Java.
```

Es ist der **einzige** Treffer von `ReadAction.compute` im Modul und zugleich
**geteilte Infrastruktur**: alle Endpoints (nav / hierarchy / overview /
inspections) routen durch diese Methode. Das Warning ist vorbestehend aus
Phase 3 (`PsiLocator.kt` ist im Working-Tree unverändert, nicht Teil der
Phase-5b-Dateiliste) — **keine Regression von Phase 5b**. Daher: eigener,
scoped Commit als Phase 5c, nicht in den 5b-Commit gemischt.

## Scope

### In Scope
- Genau **eine** Methode: `PsiLocator.inSmartReadAction`, Zeile 79.
- `ReadAction.compute<T, RuntimeException> { body() }` → `runReadAction { body() }`.
- Import anpassen: `com.intellij.openapi.application.ReadAction` →
  `com.intellij.openapi.application.runReadAction`.
- `DumbService.isDumb`-Check (`INDEXING`-BackendException) bleibt **unverändert**.

### Out of Scope
- Coroutine-Migration (`smartReadAction(project) { }`) — würde alle Handler auf
  `suspend` umstellen (großer Blast-Radius), passt nicht zu „scoped".
- Alle anderen Deprecation-Warnings im Modul.
- Jede Handler-Signatur / jeder Aufrufer von `inSmartReadAction`.

## Architektur / Datenfluss

Unverändert. `runReadAction` ist die Kotlin-Top-Level-Funktion
`com.intellij.openapi.application.runReadAction`, die
`Application.runReadAction(Computable)` kapselt: synchron, blockierend,
semantisch identisch zur jetzigen statischen `ReadAction.compute`. Die
`RuntimeException` aus `body()` propagiert weiterhin — das generische
Throwable-Parameter `E` entfällt ersatzlos.

Zielzustand der Methode:

```kotlin
fun <T> inSmartReadAction(body: () -> T): T {
    if (DumbService.getInstance(project).isDumb) {
        throw BackendException("INDEXING", "IDE is indexing; retry shortly")
    }
    return runReadAction { body() }
}
```

## Tasks

### T0 — Spike (Risiko-Gate)
Gegen IC-2026.1.3 verifizieren:
- (a) `runReadAction` ist selbst **nicht** deprecated.
- (b) Smart-Mode-Semantik bleibt erhalten (synchron, blockierend, Read-Lock).

Quelle: IDE-Symbol-Info / Plattform-Doc. **Fallback**, falls `runReadAction`
doch deprecated ist: `ReadAction.nonBlocking<T> { body() }.executeSynchronously()`.

### T1 — Migration
Edit via **Serena** (`*.kt` → Projektregel, kein nativer `Edit`/`ctx_edit`):
Import + Zeile 79 gemäß Zielzustand.

### T2 — Verifikation
1. Kotlin kompiliert **warning-frei** — Deprecation an Z. 79 verschwunden
   (Warning-as-Evidence).
2. `./gradlew test` (Kotlin-Modul) — bestehende PSI-Unit-Tests grün
   (`DefinitionResolverTest`, `ReferenceFinderTest`, `ImplementationFinderTest`,
   `TypeHierarchyResolverTest` nutzen alle `inSmartReadAction`).
3. **runIde-Live-Gate** (wie Phase-5b T11): einen Endpoint live gegen die
   laufende IDE aufrufen; Read-Action-Pfad + `INDEXING`-Pfad bestätigen.

### T3 — Commit
EIN scoped Commit. Vorher `reformat_file` auf die geänderte Datei (Projektregel).
Vorschlag:
`fix(jetbrains): migrate deprecated ReadAction.compute to runReadAction (Phase 5c)`

## Risiken & Mitigationen

| Risiko | Mitigation |
|--------|------------|
| `runReadAction` selbst deprecated gegen IC-2026.1.3 | T0-Spike vor Migration; Fallback `ReadAction.nonBlocking().executeSynchronously()` |
| Exception-/Semantik-Drift (RuntimeException-Propagation) | Bestehende PSI-Unit-Tests + runIde-Gate |
| Reformat fehlt vor `git add` | `mcp__jetbrains__reformat_file` auf `PsiLocator.kt` (Projektregel) |

## Verifikations-Evidenz (Definition of Done)

- [ ] Deprecation-Warning an `PsiLocator.kt:79` verschwunden.
- [ ] Kotlin kompiliert warning-frei.
- [ ] Alle PSI-Unit-Tests grün.
- [ ] runIde-Smoke: mind. ein Endpoint live erfolgreich, INDEXING-Pfad bestätigt.
- [ ] Genau ein scoped Commit, Datei vorher reformatiert.
