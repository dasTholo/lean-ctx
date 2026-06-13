# Spec: JetBrains-Plugin sprach-/IDE-neutral (lädt in RustRover, PyCharm, …)

- **Datum:** 2026-06-13
- **Branch:** feat-jetbrains-plugin
- **Status:** Entwurf zur Review
- **Betroffene Pakete:** `packages/jetbrains-lean-ctx`, Doc `docs/reference/19-jetbrains-plugin.md`

---

## 1. Problem

Das Plugin lädt heute **nur in IntelliJ IDEA und Android Studio**. In RustRover
(und PyCharm, GoLand, …) bricht der Start mit:

> „Requires plugin 'com.intellij.modules.java-capable' to be installed."

`com.intellij.modules.java-capable` ist **kein installierbares Plugin**, sondern
ein Plattform-Fähigkeitsmodul, das ausschließlich JVM-fähige IDEs (IDEA, Android
Studio) mitliefern. RustRover/PyCharm können es nicht nachrüsten.

### 1.1 Root Cause (auditiert)

Auslöser ist `<depends>org.jetbrains.kotlin</depends>` in `plugin.xml`, kombiniert
mit genau **zwei** Quelldateien, die gegen JVM-PSI compilen:

| Datei → Feature                                    | Kopplung                                                                                                                                         |
|----------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `psi/TypeHierarchyResolver.kt` → `type_hierarchy`  | Java-PSI (`PsiClass`, `PsiMethod`, `ClassInheritorsSearch`, `OverridingMethodsSearch` ⇒ `java-capable`) **+** Kotlin `asJava`-Light-Class-Brücke |
| `psi/FileStructureScanner.kt` → `symbols_overview` | Kotlin-only (`KtFile`, `KtClass`, `KtNamedFunction`, …)                                                                                          |

`ctx_search "import org.jetbrains.kotlin"` über `src/main` findet **nur** diese
beiden Dateien. Kein weiterer Endpoint hat eine Kotlin-Plugin-Abhängigkeit.

### 1.2 Was bereits sprach-neutral ist (kein `java-capable`)

Alle übrigen Endpoints nutzen Core-Plattform-API (`com.intellij.modules.lang`),
die in **jeder** IntelliJ-IDE vorhanden ist:

| Feature                                                                | API                                                        | Quelle                                                                       |
|------------------------------------------------------------------------|------------------------------------------------------------|------------------------------------------------------------------------------|
| Navigation (`references`/`definition`/`implementations`/`declaration`) | `PsiTreeUtil`, generisches `PsiReference`/`PsiElement`     | `psi/ReferenceFinder.kt`, `DefinitionResolver.kt`, `ImplementationFinder.kt` |
| Symbol-Edits (`replaceSymbolBody`/`insert*`)                           | Document/PSI generisch                                     | `psi/SymbolEditor.kt`                                                        |
| `rename`                                                               | `RenameProcessor`/`RenamePsiElementProcessor`/`RenameUtil` | `psi/SymbolRefactorer.kt`                                                    |
| `move`                                                                 | `MoveFilesOrDirectoriesProcessor`                          | `psi/SymbolMover.kt`                                                         |
| `safe_delete`                                                          | direkt-PSI (SafeDeleteProcessor bereits entfernt)          | `psi/SymbolDeleter.kt`                                                       |
| `inline`                                                               | generisch (`PsiMethod` nur im Kommentar)                   | `psi/SymbolInliner.kt`                                                       |
| `reformat`                                                             | `CodeStyleManager`                                         | `psi/SymbolReformatter.kt`                                                   |
| `inspections`                                                          | `com.intellij.codeInspection`                              | `psi/InspectionRunner.kt`                                                    |
| Gain-Tool-Window, Status-Bar, Doctor/Dashboard, Editor-Signal          | Subprozess / Plattform                                     | `toolwindow/*`, `actions/*`, `EditorFocusReporter.kt`                        |

> Begründung Sprach-Neutralität: `RenameProcessor`, `MoveFilesOrDirectoriesProcessor`,
> `CodeStyleManager` und die Inspection-API liegen im Core-Refactoring-/Lang-Modul
> (`com.intellij.modules.lang`), nicht im Java-Modul. RustRover/PyCharm liefern
> diese für ihre Host-Sprache mit (Rust-Rename, Python-Rename existieren).

---

## 2. Ziele / Nicht-Ziele

### Ziele

1. Plugin **lädt in allen IntelliJ-IDEs** (RustRover, PyCharm, GoLand, WebStorm,
   IDEA, …) — keine harte `java-capable`/Kotlin-Abhängigkeit mehr.
2. **Alle in `19-jetbrains-plugin.md` dokumentierten Funktionen** sind in
   RustRover/PyCharm für deren Host-Sprache lauffähig — *soweit die Host-IDE die
   Fähigkeit für die Sprache bereitstellt*.
3. **Kein Funktionsverlust in IDEA/Android Studio** — `type_hierarchy` +
   Kotlin-`symbols_overview` bleiben dort voll aktiv.
4. **Für Rust (und andere Nicht-JVM-Sprachen) nutzt lean-ctx die eigenen Tools**
   statt IDE-PSI (Architektur-Vorgabe des Users).

### Nicht-Ziele

- Kein eigener Sprach-Parser / eigene PSI-Definition (Slint-Modell) — wäre 18×
  Sprach-Plugins nachbauen; redundant zu Host-IDE-PSI **und** tree-sitter.
- Kein sprach-neutraler Neubau von `type_hierarchy`/`symbols_overview` auf
  `typeHierarchyProvider`/`StructureView` in dieser Iteration (als Option erwogen,
  verworfen wegen Doppelabdeckung mit dem lean-ctx-Rust-Pfad).

---

## 3. Design

### 3.1 Drei-Tier-Plugin

**Tier 1 — Core (sprach-neutral, lädt überall).**
`plugin.xml` deklariert nur noch `<depends>com.intellij.modules.platform</depends>`.
Enthält: Navigation, Symbol-Edits, `rename`/`move`/`safe_delete`/`inline`,
`reformat`, `inspections` sowie alle UI-Features (Status-Bar, Gain-Tool-Window,
Tools-Menü, Editor-Signal-Reporter). Funktioniert in jeder IntelliJ-IDE für die
Host-Sprache.

**Tier 2 — JVM-PSI-Optionalmodul.**
Neue optionale Config-Datei `META-INF/leanctx-jvm.xml`, eingebunden via:

```xml

<depends optional="true" config-file="leanctx-jvm.xml">org.jetbrains.kotlin</depends>
```

`leanctx-jvm.xml` registriert ausschließlich, was JVM-PSI braucht:
`type_hierarchy` + Kotlin-`symbols_overview`. `TypeHierarchyResolver.kt` und
`FileStructureScanner.kt` verbleiben hier (compilen gegen JVM-PSI, werden zur
Laufzeit **nur** geladen, wenn das Kotlin-Plugin präsent ist). In RustRover/PyCharm
wird das Modul nie klassengeladen → kein `NoClassDefFoundError`.

**Tier 3 — Rust/Non-JVM-Routing (Rust-Backend, unverändert vorhanden).**
Für Nicht-JVM-Sprachen liefert lean-ctx das Äquivalent ohne IDE-PSI:

| Bedarf                  | lean-ctx-Pfad                                       | Status                            |
|-------------------------|-----------------------------------------------------|-----------------------------------|
| Symbol-Overview         | `ctx_outline` / `overview_from_index` (tree-sitter) | vorhanden (§2.2 headless default) |
| Trait-/Typ-„Hierarchie" | `implementations` via Backing A (rust-analyzer)     | vorhanden (§1.1 Mandatory-Tier)   |
| Call-Hierarchie         | `ctx_callgraph` (callers/callees/trace/risk)        | vorhanden                         |

### 3.2 Feature × IDE — Zielzustand

| Feature                                       | RustRover (Rust)                         | PyCharm (Python)    | IDEA (JVM)                       |
|-----------------------------------------------|------------------------------------------|---------------------|----------------------------------|
| Navigation (refs/def/impl/decl)               | ✅ Plugin-PSI (Rust) / Backing-A-Fallback | ✅ Plugin-PSI        | ✅ Plugin-PSI                     |
| `symbols_overview`                            | ✅ lean-ctx tree-sitter                   | ✅ tree-sitter       | ✅ IDE-PSI (Kotlin) + tree-sitter |
| `type_hierarchy`                              | → `implementations`/`ctx_callgraph`      | → `implementations` | ✅ IDE-PSI (Java+Kotlin)          |
| Edits / Refactor / `reformat` / `inspections` | ✅ Plugin (Plattform)                     | ✅                   | ✅                                |
| UI (Gain, Status-Bar, Doctor, Editor-Signal)  | ✅                                        | ✅                   | ✅                                |

### 3.3 Graceful Degradation

- Fragt eine Host-IDE eine Fähigkeit für eine Sprache nicht bereit → bestehende
  Fehlercodes `UNSUPPORTED_LANGUAGE` / `BACKEND_REQUIRED` (Error-Katalog §9),
  **kein** Absturz/Class-Loading-Fehler.
- `type_hierarchy` in Nicht-JVM-IDE: Endpoint ist dort nicht registriert →
  Rust-Backend liefert wie bisher die degradierte Antwort bzw. routet auf
  `implementations`.

---

## 4. Konkrete Änderungen

### 4.1 `src/main/resources/META-INF/plugin.xml`

- `− <depends>org.jetbrains.kotlin</depends>`
- `+ <depends optional="true" config-file="leanctx-jvm.xml">org.jetbrains.kotlin</depends>`
- Den `org.jetbrains.kotlin`-Extension-Block (`<supportsKotlinPluginMode supportsK2="true"/>`)
  **nach `leanctx-jvm.xml` verschieben** (gehört zum optionalen Modul).
- `statusBarWidgetFactory`, `postStartupActivity`, `toolWindow`, `registryKey`,
  `actions` bleiben im Core.
- Die Registrierung der `type_hierarchy`/`symbols_overview`-PSI-Handler (sofern
  via EP) wandert nach `leanctx-jvm.xml`.

### 4.2 `src/main/resources/META-INF/leanctx-jvm.xml` (neu)

Enthält nur die JVM-PSI-gebundenen Registrierungen + den K2-Mode-Block.

### 4.3 `build.gradle.kts`

- `bundledPlugin("org.jetbrains.kotlin")` bleibt als **Compile-Abhängigkeit**
  (die zwei JVM-PSI-Klassen müssen weiter compilen). Es erzeugt **keine** harte
  Runtime-Dep, solange `plugin.xml` die Kotlin-Abhängigkeit nur **optional**
  deklariert.
- Kotlin bleibt Implementierungssprache (stdlib ist plattform-gebündelt → zieht
  **kein** `java-capable`).
- Prüfen: `ideaVersion.sinceBuild`/Produktkompatibilität so, dass RustRover/PyCharm
  das Plugin akzeptieren (nur `com.intellij.modules.platform` im Core-Manifest →
  wird akzeptiert).
- **Sandbox-Run-Tasks für Nicht-JVM-IDEs** (IntelliJ Platform Gradle Plugin 2.x):
  zusätzliche `runIde`-Varianten registrieren, um das Plugin in RustRover/PyCharm
  zu starten (Verifikation, §5.2):

  ```kotlin
  intellijPlatformTesting.runIde {
      register("runRustRover") {
          type = org.jetbrains.intellij.platform.gradle.IntelliJPlatformType.RustRover
          version = "2026.1"
      }
      register("runPyCharm") {
          type = org.jetbrains.intellij.platform.gradle.IntelliJPlatformType.PyCharmCommunity
          version = "2026.1"
      }
  }
  ```

  Alternativ `local(file("<pfad-zur-RustRover-Installation>"))` für eine lokal
  installierte IDE.

### 4.4 Endpoint-/Router-Seite

- `endpoint/StructureHandlers.kt` instanziiert `TypeHierarchyResolver` +
  `FileStructureScanner` **bedingt** bzw. wird selbst Teil des Optionalmoduls,
  sodass es in Nicht-JVM-IDEs nicht klassengeladen wird.
- `RequestRouter`: Routen `/type_hierarchy`, `/symbols_overview` (IDE-PSI-Variante)
  nur registrieren, wenn das JVM-Modul aktiv ist; sonst greift die Rust-seitige
  Backend-Auswahl (tree-sitter / rust-analyzer) wie in §1.1 dokumentiert.

### 4.5 Doku

- `docs/reference/19-jetbrains-plugin.md`: §2.2 + §6.3 (K2) um die IDE-Matrix
  ergänzen (welche Features in welcher IDE-Klasse aktiv sind); klarstellen, dass
  `type_hierarchy`/Kotlin-`symbols_overview` JVM-IDE-exklusiv sind und Rust/Python
  über lean-ctx-Tools (`ctx_outline`, `ctx_callgraph`, `implementations`) bedient
  werden.

---

## 5. Tests / Verifikation

### 5.1 Automatisiert

- **Rust-Suite:** `cargo nextest run` (Backend-Routing, drift-Tests unverändert grün).
- **Plugin (Gradle):** bestehende JVM-PSI-Tests laufen weiter gegen IDEA-Target.
- **Manifest-Verifikation:** JetBrains Plugin Verifier gegen RustRover- **und**
  PyCharm-IDE → keine ungelösten `java-capable`/Kotlin-Abhängigkeiten im Core-Manifest.

### 5.2 Cross-IDE-Rust-Gate (Live-Runbook — PFLICHT gegen echte RustRover-IDE)

> Verifikation **muss live gegen die RustRover-IDE** laufen (eigene
> `runRustRover`-Sandbox, §4.3), **nicht** gegen IDEA. Das IDEA-`runIde` würde den
> java-capable-Block nie reproduzieren und damit nichts beweisen.

Neues Runbook `docs/lean-md/runbooks/runrustrover-cross-ide-gate.md` (Muster:
`runide-inline-reformat-gate.md`). Ablauf:

**Voraussetzungen:** frisches Binary (Daemon-Stopp, `cargo build`); Plugin gebaut
(`./gradlew buildPlugin`).

**Setup — Cargo-Fixture** (neues `scripts/runrustrover-cross-ide-gate-setup.sh`):
minimales Cargo-Projekt mit `trait Shape { fn area(&self)->f64; }`, zwei Impls
(`Circle`, `Square`), einer Funktion mit ≥2 Call-Sites und einer absichtlich
fehlformatierten Datei.

**Launch — RustRover-Sandbox auf dem Fixture:**

```bash
FIX="$(pwd)/tmp/runrustrover-cross-ide-gate"
./gradlew runRustRover --args="$FIX"   # cwd=packages/jetbrains-lean-ctx
```

Indizierung abwarten (Cargo-Projekt erkannt, Statusleiste idle).

**Gate-Checks** (`lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`):

| #  | Fall                                       | Soll-Ergebnis                                                                                                  |
|----|--------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| 0  | **Plugin lädt** in RustRover               | KEIN „java-capable"-Fehler; Port-File geschrieben; `GET /health` ok                                            |
| 1  | UI sichtbar                                | Status-Bar-Widget `⚡ lean-ctx`, Tools-Menü, Gain-Tool-Window vorhanden                                         |
| 2  | `references` auf `area` (Rust)             | Usages über Impls/Call-Sites gefunden                                                                          |
| 3  | `definition` / `declaration` (Rust)        | korrekte Zielposition                                                                                          |
| 4  | `implementations` auf `trait Shape` (Rust) | `Circle`+`Square` als Impls (Trait→Impl-„Hierarchie")                                                          |
| 5  | `rename` (Two-Phase, Rust-Symbol)          | Preview liefert Usages; Apply benennt projektweit um (eine Transaktion)                                        |
| 6  | `reformat` der fehlformatierten Rust-Datei | korrekt formatiert (CodeStyleManager, Rust)                                                                    |
| 7  | `inspections mode=run` (Rust-Datei)        | Diagnostics geliefert oder sauber leer (kein Crash)                                                            |
| 8  | `symbols_overview` (Rust)                  | via lean-ctx tree-sitter — Top-Level-Symbole, kein IDE-PSI nötig                                               |
| 9  | `type_hierarchy` (Rust)                    | **sauber degradiert** (Endpoint nicht registriert → Routing auf `implementations`/`ctx_callgraph`, kein Crash) |
| 10 | `ctx_callgraph` callers/callees (Rust-Fn)  | Call-Hierarchie geliefert (lean-ctx-Pfad)                                                                      |
| 11 | Editor-Signal                              | Fokuswechsel auf Rust-Datei → `editor-signal` emittiert (Pfad-only)                                            |

**Ergebnis-Tabelle** (analog Runbook) nach dem Live-Durchlauf eintragen
(IDE-Version, Befund je Check ✅/⏭️).

### 5.3 Regressions-Gate IDEA

`type_hierarchy` + Kotlin-`symbols_overview` in IDEA-`runIde` unverändert aktiv
(Optionalmodul lädt dort). Bestehendes `runide-inline-reformat-gate.md` bleibt grün.

### 5.4 Optional — PyCharm-Gate

Analog §5.2 mit `./gradlew runPyCharm` + Python-Fixture (Klasse + Subklasse): Core +
Navigation/Rename/Reformat für Python; `type_hierarchy` degradiert auf
`implementations`.

---

## 6. Risiken / offene Punkte

- **Compile-vs-Runtime-Trennung:** Sicherstellen, dass keine Core-Klasse statisch
  auf `TypeHierarchyResolver`/`FileStructureScanner` referenziert (sonst wird das
  JVM-Modul transitiv geladen). Instanziierung muss hinter dem Optionalmodul liegen.
- **Plugin-Verifier-Kompatibilität:** Build-Target (IDEA) vs. Lauf-Target
  (RustRover) — `sinceBuild`/`untilBuild` + `productDescriptor` so wählen, dass
  Nicht-JVM-IDEs das Artefakt annehmen.
- **K2-Block** gehört zwingend ins Optionalmodul (referenziert
  `org.jetbrains.kotlin`-Namespace).
- **Doppelabdeckung Navigation:** In RustRover können sowohl Plugin-PSI (Rust) als
  auch Backing A (rust-analyzer) `references`/`implementations` liefern — die
  bestehende `select_backend`-Logik (§1.1) entscheidet; kein neuer Konflikt, aber
  Verhalten dokumentieren.

---

## 7. Quellen / Referenzen

- `docs/reference/19-jetbrains-plugin.md` (§0–§11, Funktions-/Architektur-Referenz)
- `docs/reference/appendix-mcp-tools.md` (`ctx_refactor`, `ctx_outline`, `ctx_callgraph`)
- IntelliJ Platform SDK: [Plugin Compatibility](https://plugins.jetbrains.com/docs/intellij/plugin-compatibility.html),
  [Structure View](https://plugins.jetbrains.com/docs/intellij/structure-view.html)
- Recherche: PyCharm hat Type Hierarchy (Python); RustRover hat Call Hierarchy
  (2026.1, trait-aware) + Go-to-Implementations, **kein** klassisches Type
  Hierarchy (Rust ohne Klassen-Vererbung).
