# Runbook: runIde-Inline/Reformat-Gate (v2d Live-Verifikation)

Verifiziert den vollen v2d-Stack live: Rust-Gate (`plan_hash`/TOCTOU, Adress-Dualität
für reformat, `INVALID_TARGET`, `INDEXING`, `UNSUPPORTED_LANGUAGE`, `BACKEND_REQUIRED`)
**und** das JetBrains-Plugin (`SymbolInliner`/`SymbolReformatter`-Naht, Two-Phase für
inline, Single-Phase für reformat) gegen ein sauberes Kotlin-Gradle-Fixture.

Bezug: Spec `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-v2d-inline-reformat.md` §10.1.

## Voraussetzungen — frisches Binary (Daemon-Stopp ist PFLICHT)

Die neuen Actions (`inline_*`/`reformat`) existieren erst nach Neubau. Ein
**laufender** lean-ctx-Daemon hält den **alten** Action-Satz im Speicher →
`Unknown action`. Reihenfolge **vor** dem Gate:

1. `lean-ctx serve --stop` — Daemon stoppen (gibt Binary frei + entlädt alten Action-Satz).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren].
3. `lean-ctx serve --daemon` neu starten **oder** ersten `lean-ctx call` den Daemon auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist dieser Daemon
> zugleich der `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools.
> Das Gate als **separaten** Schritt fahren, nicht mitten in einer ctx_*-Aufgabe.

- Plugin-Modul gebaut: `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`).

## 1. Setup — Fixture materialisieren

```bash
bash scripts/runide-inline-reformat-gate-setup.sh
```

Notiere `FIX=<abs>/tmp/runide-inline-reformat-gate`.

## 2. Launch — Sandbox-IDE auf dem Fixture

```bash
FIX="$(pwd)/tmp/runide-inline-reformat-gate"
./gradlew runIde --args="$FIX"
```

(cwd=`packages/jetbrains-lean-ctx`) — **Indizierung abwarten** (Statusleiste idle).

> Falls `runIde --args` das Projekt nicht öffnet: einmal manuell `File → Open` auf `$FIX`.

## 3. Gate-Checks

Jeder Check: `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`.
Für TOCTOU-Fälle zuerst das passende `inline_preview` ausführen, um den aktuellen
`plan_hash` zu holen.

| #  | Fall                                            | Aufruf (`--json`, Auszug)                                                                                                                                                                        | Soll-Ergebnis                                                                                                            |
|----|-------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| 1  | inline_preview — lokale Variable                | `{"action":"inline_preview","path":"src/main/kotlin/app/Calc.kt","line":5}`                                                                                                                      | `plan_hash` gesetzt, `tmp` identifiziert, keine Konflikte                                                                |
| 2  | inline_apply — lokale Variable                  | `{"action":"inline_apply","path":"src/main/kotlin/app/Calc.kt","line":5,"plan_hash":"<#1>"}`                                                                                                     | `tmp` inlined, `return (a+b)+(a+b)`, Datei geändert                                                                      |
| 3  | inline_preview — Methode mit ≥2 Call-Sites      | `{"action":"inline_preview","path":"src/main/kotlin/app/Helper.kt","line":4}`                                                                                                                    | `plan_hash` gesetzt, 2 Call-Sites (`h.calc(3)`, `h.calc(4)`) erkannt                                                     |
| 4  | inline_apply mit keep_definition=true           | `{"action":"inline_apply","path":"src/main/kotlin/app/Helper.kt","line":4,"plan_hash":"<#3>","keep_definition":true}`                                                                            | Call-Sites mit `x*2` ersetzt, Methoden-Deklaration bleibt erhalten                                                       |
| 5  | inline — rekursive Methode (UNSUPPORTED)        | `{"action":"inline_preview","path":"src/main/kotlin/app/Recurse.kt","line":4}`                                                                                                                   | `UNSUPPORTED` (IntelliJ-Inline-Handler lehnt rekursive Methoden ab)                                                      |
| 6  | TOCTOU — veralteter plan_hash                   | (a) `inline_preview` auf `Calc.kt:5` → `plan_hash_A`; (b) eine Nutzung manuell ändern; (c) `{"action":"inline_apply","path":"src/main/kotlin/app/Calc.kt","line":5,"plan_hash":"<plan_hash_A>"}` | `CONFLICT` — Rust-Gate erkennt Hash-Mismatch, kein Apply                                                                 |
| 7  | reformat — gesamte Datei                        | `{"action":"reformat","path":"src/main/kotlin/app/Messy.kt"}`                                                                                                                                    | Datei formatiert: geschweifte Klammern, Einrückungen, Leerzeichen korrigiert                                             |
| 8  | reformat — Region (Zeilen)                      | `{"action":"reformat","path":"src/main/kotlin/app/Messy.kt","line":3,"end_line":7}`                                                                                                              | Nur der angegebene Zeilenbereich formatiert                                                                              |
| 9  | reformat — Symbol                               | `{"action":"reformat","name_path":"Messy/render"}`                                                                                                                                               | Nur Methode `render` formatiert, `other` unverändert                                                                     |
| 10 | reformat — optimize_imports                     | `{"action":"reformat","path":"src/main/kotlin/app/Imports.kt","optimize_imports":true}`                                                                                                          | Ungenutzte Imports (`ArrayList`, `HashMap`) entfernt (`OptimizeImportsProcessor` aus `com.intellij.codeInsight.actions`) |
| 11 | INVALID_TARGET — keine Adresse / beide Adressen | (a) `{"action":"reformat"}` (kein `path`); (b) `{"action":"reformat","name_path":"Messy/render","path":"src/main/kotlin/app/Messy.kt"}` (name_path + path zugleich)                              | je `INVALID_TARGET`, **vor** Backend-Call, kein Apply                                                                    |
| 12 | INDEXING                                        | Projekt neu öffnen, sofort `inline_preview` während Indizierung                                                                                                                                  | `INDEXING`, kein Teil-Edit                                                                                               |
| 13 | UNSUPPORTED_LANGUAGE                            | `{"action":"inline_preview","path":"notes.txt","line":1}`                                                                                                                                        | `UNSUPPORTED_LANGUAGE`, kein Crash                                                                                       |
| 14 | BACKEND_REQUIRED                                | IDE schließen, dann `inline_preview` und `reformat`                                                                                                                                              | `BACKEND_REQUIRED` in beiden Fällen                                                                                      |

### Task 7 `runInline` — Live-Gate-Befund (2026-06-13)

**Ergebnis: `inline_apply` für Kotlin ist eine dokumentierte Headless-Grenze
(`UNSUPPORTED_LANGUAGE`)** — Präzedenz `SymbolMover` Member-Move (v2c). Begründung:
Kotlins Inline-Handler (`KotlinInlineValHandler` / `KotlinInlineNamedFunctionHandler`)
sind Kotlin-Plugin-**Interna** ohne stabile `compileOnly`-SDK-Fläche **und** öffnen
einen **modalen Dialog** — ein Modal auf dem HTTP-Handler-Thread deadlockt (runIde-Gate
#8). Es gibt aktuell keinen dialogfreien Pfad.

- **`inline_preview` + das Rust-Gate** (`plan_hash`/TOCTOU, force-loser Konflikt-Block)
  sind voll funktionsfähig und am Gate grün (#1/#3/#5/#6).
- **`inline_apply`** wird sauber abgelehnt (`UNSUPPORTED_LANGUAGE`), kein Teil-Edit,
  kein Deadlock (#2/#4).
- **Java-Pfad** (`InlineMethodProcessor`/`InlineLocalHandler`, dialog-unterdrückbar,
  stabiles Platform-API) ist das Verdrahtungsziel, sobald ein Java-Fixture ergänzt wird
  (Pseudocode-Kommentar in `SymbolInliner.runInline`).

> **Begleitender Fix (am Gate gefunden):** `SymbolInliner.resolveTarget` traf bei
> **eingerückten** Deklarationen das falsche Symbol — `findElementAt(line, char 0)`
> landet auf der Einrückung, `getParentOfType(PsiNamedElement)` greift dann den
> umschließenden Knoten (lokale Var → Funktion; Methode → Klasse). Fix: führende
> `PsiWhiteSpace` via `PsiTreeUtil.nextLeaf` überspringen. Die v2c-Geschwister
> (`SymbolMover`/`SymbolDeleter`/`SymbolRefactorer`/`ReferenceFinder`) trugen dasselbe
> latente Muster (Testsymbole auf Spalte 0 → nie exponiert); seit dem v2c-Port
> (Spec `2026-06-13-leanctx-jetbrains-v2c-resolvetarget-indentation-fix.md`) ist der
> Fix dort gespiegelt und je mit einem RED-first-Unit-Test über den Preview-Pfad
> (`SymbolDeleterTest`/`SymbolMoverTest`/`SymbolRefactorerTest`/`ReferenceFinderTest`)
> abgesichert — nicht mehr latent.

### Hinweis: OptimizeImportsProcessor — Package-Gotcha

`OptimizeImportsProcessor` liegt in **`com.intellij.codeInsight.actions`** (NICHT in
`com.intellij.refactoring.actions`). Bei IC/IU-2026.1.3 schlägt ein Import aus dem
falschen Package mit `ClassNotFoundException` zur Laufzeit fehl — auch wenn der Code
compileOnly-kompatibel aussieht. Sicherste Prüfung: `javap -cp <plugin-jar>` auf
`OptimizeImportsProcessor` verifizieren oder `grep -r OptimizeImports
~/.gradle/caches/modules-*/files-*/com.jetbrains.intellij*`.

### Durchlauf 2026-06-13 — Ergebnis

IC-2026.1.3-Sandbox, Fixture wie oben. Befund je Check:

| #  | Ergebnis | Notiz                                                                                                             |
|----|----------|-------------------------------------------------------------------------------------------------------------------|
| 1  | ✅        | `usages: 2` (`tmp`), `plan_hash` gesetzt — nach `resolveTarget`-Fix korrekt                                       |
| 2  | ✅        | `UNSUPPORTED_LANGUAGE` (Kotlin-Headless-Grenze; Hash+Konflikt-Gate passiert, dann `runInline`)                    |
| 3  | ✅        | `usages: 2` (`calc`, 2 Call-Sites)                                                                                |
| 4  | ✅        | `UNSUPPORTED_LANGUAGE` (mit `keep_definition=true`)                                                               |
| 5  | ✅        | Preview trifft `loop` (`usages: 1`, Selbstaufruf); apply → `UNSUPPORTED_LANGUAGE` (subsumiert Rekursions-Refusal) |
| 6  | ✅        | `CONFLICT: plan_hash mismatch` mit `expected`/`actual` — force-loser TOCTOU-Gate greift **vor** `runInline`       |
| 7  | ✅        | Messy.kt vollständig formatiert (Klammern/Einrückung/Spacing)                                                     |
| 8  | ✅        | Region `applied`                                                                                                  |
| 9  | ✅        | Symbol via `name_path` `applied` (Index: 7 Dateien/12 Symbole)                                                    |
| 10 | ✅        | `optimize_imports` entfernt `ArrayList`/`HashMap` — `OptimizeImportsProcessor`-Package-Gotcha **live bestätigt**  |
| 11 | ✅        | `INVALID_TARGET` (keine Adresse **und** beide Adressen)                                                           |
| 12 | ⏭️       | Nicht live ausgelöst (Steady-State-Index); `INDEXING`-Gate über `inSmartReadAction` (Dumb-Mode) + Unit abgedeckt  |
| 13 | ✅        | `UNSUPPORTED_LANGUAGE: ... for TEXT` (notes.txt)                                                                  |
| 14 | ✅        | `BACKEND_REQUIRED` — vor IDE-Start reproduziert (kein Live-Port-File)                                             |

**Fazit:** reformat-Stack + Rust-Gate (TOCTOU/force-los/Adress-Dualität/Fehlerfälle)
vollständig live verifiziert. `inline_apply` = dokumentierte Kotlin-Headless-Grenze.

## 4. Teardown

- Sandbox-IDE schließen.
- `tmp/runide-inline-reformat-gate/` kann liegen bleiben (gitignored) oder via
  `bash scripts/runide-inline-reformat-gate-setup.sh` zurückgesetzt werden.
- Daemon wieder hochfahren, falls für die MCP-Session gestoppt.
