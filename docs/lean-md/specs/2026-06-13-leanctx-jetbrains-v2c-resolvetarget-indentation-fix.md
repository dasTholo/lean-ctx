# Spec: v2c resolveTarget-Indentation-Fix (Mover / Deleter / Refactorer / ReferenceFinder)

**Datum:** 2026-06-13
**Branch:** feat-jetbrains-plugin
**Bezug:** Live-Gate-Befund v2d, `docs/lean-md/runbooks/runide-inline-reformat-gate.md`
(Abschnitt „Task 7 `runInline` — Live-Gate-Befund / Begleitender Fix")
**Präzedenz-Fix:** `SymbolInliner.resolveTarget` (Commit `41ea601c`)

## 1. Problem

Die PSI-Symbolauflösung der v2c-Refactorer adressiert Ziele **zeilenbasiert mit
`character = 0`** (`range.start`). `PsiFile.findElementAt(offset)` landet bei
eingerückten Deklarationen damit auf der führenden `PsiWhiteSpace`. Der
anschließende Aufstieg —

- `PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)` (Mover,
  Deleter, Refactorer), bzw.
- `generateSequence(element) { it.parent }.firstOrNull { it is PsiNamedElement }`
  (ReferenceFinder)

— greift dann den **umschließenden** benannten Knoten statt der Deklaration auf der
Zeile: Member → umschließende Klasse, lokale Variable → umschließende Funktion.

Das Muster ist in allen vier Klassen **latent**: v2c-Tests und -Live-Gates
verwendeten Symbole auf **Spalte 0** (Top-Level-Deklarationen), sodass `findElementAt`
nie auf Einrückung traf. Erst der v2d-Inline-Live-Gate (eingerücktes Member) hat die
Naht exponiert; der Fix wurde dort in `SymbolInliner.resolveTarget` eingebaut, die
v2c-Geschwister aber bewusst unangetastet gelassen (Runbook-Notiz).

## 2. Ziel

Den bereits in `SymbolInliner` verifizierten Fix auf die vier latenten Träger
portieren und ihn — anders als bei v2d — mit **RED-first-Unit-Tests** absichern, da
der Resolve-Schritt über die Preview-Pfade headless-testbar ist (kein Processor, kein
modaler Dialog, keine Headless-Grenze).

## 3. Scope

**In Scope** (alle vier Träger des Musters):

| Datei | Methode | Idiom | Zeile (Stand HEAD) |
|---|---|---|---|
| `SymbolDeleter.kt` | `resolveTarget` | `getParentOfType` | 106–108 |
| `SymbolMover.kt` | `resolveSource` | `getParentOfType` | 138–140 |
| `SymbolRefactorer.kt` | `resolveTarget` | `getParentOfType` | 206–208 |
| `ReferenceFinder.kt` | `resolveTarget` | `generateSequence { parent }` | 59–63 |

Pfade relativ zu
`packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/psi/`.

**Out of Scope:**

- `SymbolInliner.resolveTarget` — bereits gefixt (`41ea601c`), Referenz-Implementierung.
- `ImplementationFinder` / `TypeHierarchyResolver` (`findElementAt` ohne den
  `getParentOfType`/Parent-Walk-Aufstieg auf char 0) — kein latentes Muster dieser Art;
  nicht Teil dieser Spec.
- Keine Änderung an Processor-/Apply-Pfaden, an DTOs, am Rust-Gate oder am Routing.

## 4. Fix-Design

### 4.1 `getParentOfType`-Idiom (Deleter / Mover / Refactorer)

Vor dem `getParentOfType`-Aufruf einfügen — wortgleich zur `SymbolInliner`-Vorlage:

```kotlin
val offset = locator.offsetOf(file, line, character)
var at = file.findElementAt(offset)
    ?: throw BackendException("NO_SYMBOL", "no element at $line:$character")
// Line-addressed targets (char 0) land on the leading indentation; skip it so
// getParentOfType resolves the declaration ON the line, not its enclosing
// class/function. Top-level (col-0) symbols never hit this; surfaced at the v2d
// inline live-gate, ported to the v2c siblings.
if (at is PsiWhiteSpace) {
    at = PsiTreeUtil.nextLeaf(at) ?: at
}
val named = PsiTreeUtil.getParentOfType(at, PsiNamedElement::class.java, false)
```

`val at` wird zu `var at`. Bestehende `throw`-Pfade und Rückgaben unverändert.

### 4.2 `generateSequence`-Idiom (ReferenceFinder)

`ReferenceFinder.resolveTarget` versucht zuerst `findReferenceAt` (Caret auf einer
Nutzung). Schlägt das fehl, läuft der `findElementAt` → Parent-Walk; dort denselben
Whitespace-Skip einfügen, bevor der `generateSequence`-Aufstieg startet:

```kotlin
var element = file.findElementAt(offset)
    ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no element at $line:$character")
if (element is PsiWhiteSpace) {
    element = PsiTreeUtil.nextLeaf(element) ?: element
}
return generateSequence(element) { it.parent }
    .firstOrNull { it is PsiNamedElement }
    ?: throw BackendException("NO_SYMBOL_AT_POSITION", "no named symbol at $line:$character")
```

Nötige Imports je Datei prüfen/ergänzen: `com.intellij.psi.PsiWhiteSpace`,
`com.intellij.psi.util.PsiTreeUtil` (in den `getParentOfType`-Dateien bereits präsent).

### 4.3 Sicherheitseigenschaft

Bei **Realspalten**-Adressierung (`character` = echte Spalte der Deklaration) ist
`at` kein `PsiWhiteSpace` → der Guard ist ein **No-op**. Der Fix ist rein additiv und
kann kein bestehendes (Spalte-0-Top-Level-)Verhalten verändern. Genau diese Eigenschaft
macht die Portierung auf alle vier Klassen risikofrei.

## 5. Verifikation — RED-first TDD über die Preview-Pfade

Die öffentlichen `preview(req)`-Methoden (Deleter/Mover/Refactorer) bzw.
`ReferenceFinder.find(...)` lösen das Zielsymbol auf und melden dessen **Usages** —
ohne Processor und ohne modalen Dialog, also headless-sicher (im Gegensatz zum
v2d-`inline_apply`, das an der Kotlin-Headless-Grenze scheiterte).

### 5.1 Test-Harness

`BasePlatformTestCase` (Muster wie `ReferenceFinderTest` /
`RequestRouterRefactorTest`), Verzeichnis
`packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/psi/`.

- **Neu:** `SymbolDeleterTest`, `SymbolMoverTest`, `SymbolRefactorerTest`.
- **Erweitert:** `ReferenceFinderTest` (neuer Test, bestehender bleibt).

### 5.2 Fixture & Diskriminator

Eingerücktes Member in umschließender Klasse, adressiert auf **`character = 0`** der
Member-Zeile, plus eine Call-Site, die das Member referenziert:

```kotlin
class Outer {
    fun target() {}
}
fun a() { Outer().target() }
```

- **RED** (ohne Fix): Auflösung trifft `Outer` (umschließende Klasse) → Preview meldet
  die `Outer()`-Konstruktor-Usage / einen `Outer`-Snippet.
- **GREEN** (mit Fix): Auflösung trifft `target` → Preview meldet die `.target()`-Usage.

Assertion auf den gemeldeten Usage-**Snippet**/-**Range** (`RenamePreviewResponse`)
diskriminiert eindeutig zwischen richtigem und falschem Symbol. Für `ReferenceFinder`
analog über die Usage-Locations (`find(...)`-Ergebnis): das eingerückte `target` hat
genau eine Call-Site, die umschließende `Outer` eine andere/keine.

### 5.3 Threading

`SymbolRefactorer`/`SymbolMover` rufen für Kotlin die Analysis API (`KaSession`) —
auf dem EDT verboten. Falls der Preview-Pfad das auslöst, Test über das
`routeOffEdt`-Muster aus `RequestRouterRefactorTest` (pooled thread +
`PlatformTestUtil.waitForFuture`). Wenn der direkte Konstruktoraufruf in
`inSmartReadAction` genügt (wie `ReferenceFinderTest`), diesen einfacheren Weg nehmen.
Die Threading-Wahl pro Klasse beim Schreiben des jeweiligen RED-Tests bestimmen.

### 5.4 Ablauf

Pro Klasse: RED-Test zeigen (fehlschlagend ohne Fix) → Fix einbauen → GREEN.
Gesamtlauf:

```
./gradlew test
```

(cwd=`packages/jetbrains-lean-ctx`; bare command, kein `cd … &&`, kein `| tail`).
Kein Live-`runIde` erforderlich — der Preview-Pfad ist headless deterministisch. Ein
optionaler Live-Gate-Abgleich kann später über das bestehende Runbook erfolgen.

## 6. Editier-Disziplin (Projekt-Hard-Rules)

- `*.kt`-Edits **ausschließlich** über Serena-Tools
  (`replace_symbol_body` / `insert_before_symbol` / `replace_content`), nie native
  `Edit`/`ctx_edit`.
- Vor `git add`: `mcp__jetbrains__reformat_file` auf jede geänderte Datei.
- Tests: `./gradlew test`, bare command via `ctx_shell` mit `cwd=`.

## 7. Akzeptanzkriterien

1. Alle vier `resolveTarget`/`resolveSource`-Methoden überspringen führende
   `PsiWhiteSpace` vor dem Parent-Aufstieg.
2. Vier RED-first-Tests existieren; jeder schlägt ohne den jeweiligen Fix fehl und ist
   mit Fix grün.
3. `./gradlew test` vollständig grün; keine Regression in bestehenden
   `RequestRouter*Test`/`ReferenceFinderTest`-Fällen.
4. Geänderte `.kt`-Dateien sind reformatiert (JetBrains-Formatter).
5. Runbook-Notiz „Begleitender Fix" in
   `docs/lean-md/runbooks/runide-inline-reformat-gate.md` aktualisiert: die
   v2c-Geschwister sind **nicht mehr latent**, Fix portiert + unit-bewiesen (Verweis
   auf die neuen Tests).

## 8. Risiken & Annahmen

- **Annahme:** Die Preview-Pfade lösen das Symbol über dieselbe `resolveTarget`-Naht
  auf, die der Apply-Pfad nutzt — durch Code-Lesung bestätigt (gemeinsame private
  Methode je Klasse).
- **Risiko (gering):** Kotlin-`KaSession`-Threading im Preview könnte je nach Klasse
  EDT-Marshalling erzwingen → über `routeOffEdt` adressiert (§5.3).
- **Risiko (sehr gering):** Fix ist No-op bei Realspalten-Adressierung (§4.3) → keine
  Verhaltensänderung für bestehende col-0-Pfade zu erwarten.
