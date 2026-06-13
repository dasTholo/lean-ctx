# JetBrains Plugin — Funktions-Inventar & Gap-Analyse

> Temporäres Prozessartefakt. Erstellt von Task A1.
> Treibt Tasks A2b–A5 (Dokumentationsergänzungen).
> Wird in Teil B aus dem Branch-Diff ausgeschlossen.

Quellen: `packages/jetbrains-lean-ctx/src/main/resources/META-INF/plugin.xml`,
`packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/{endpoint,server,toolwindow,actions,util}/…`

---

## 1. Extension-Registrierungen (plugin.xml)

| Funktion | Klasse / Schlüssel | In Doku §X? | Quelle |
|---|---|---|---|
| Status-Bar-Widget (Token-Savings-Anzeige) | `LeanCtxStatusBarFactory` / `LeanCtxStatusBarWidget` | **GAP** — nicht dokumentiert | plugin.xml `statusBarWidgetFactory`; `LeanCtxStatusBarFactory.kt` |
| HTTP-Server-Boot beim Projektstart | `LeanCtxStartupActivity` | §1 (Architektur, implizit) — kein eigener Abschnitt | plugin.xml `postStartupActivity`; `LeanCtxStartupActivity.kt` |
| Gain Tool Window (`LeanCtxGain`) | `LeanCtxGainToolWindowFactory` | **GAP** — nicht dokumentiert | plugin.xml `toolWindow id=LeanCtxGain`; `toolwindow/*.kt` |
| Editor-Focus Reporter (Registry-Key opt-out) | `leanctx.editor.signal.enabled` | **GAP** — nicht dokumentiert | plugin.xml `registryKey`; `EditorFocusReporter.kt` |
| K2-Modus-Unterstützung | `supportsKotlinPluginMode supportsK2="true"` | **GAP** — nicht dokumentiert | plugin.xml `<extensions defaultExtensionNs="org.jetbrains.kotlin">` |

## 2. Actions-Gruppe (plugin.xml → ToolsMenu)

| Action-ID | Klasse | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `LeanCtx.Setup` | `SetupAction` | Führt `lean-ctx setup` aus, zeigt Ausgabe in Messages-Popup | **GAP** — nicht dokumentiert | plugin.xml; `LeanCtxActions.kt` |
| `LeanCtx.Doctor` | `DoctorAction` | Führt `lean-ctx doctor` aus, zeigt ANSI-bereinigte Ausgabe in Messages-Popup | **GAP** — nicht dokumentiert | plugin.xml; `LeanCtxActions.kt` |
| `LeanCtx.Gain` | `GainAction` | Öffnet das Gain Tool Window (`LeanCtxGain`) | **GAP** — nicht dokumentiert | plugin.xml; `LeanCtxActions.kt` |
| `LeanCtx.Dashboard` | `DashboardAction` | Führt `lean-ctx dashboard` aus (fire-and-forget) | **GAP** — nicht dokumentiert | plugin.xml; `LeanCtxActions.kt` |

## 3. HTTP-Endpunkte (RequestRouter.kt)

### 3.1 Health

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/health` | GET | Liveness-Check; antwortet `{"status":"ok","ideVersion":…,"project":…}` | §1.2 (Port-Discovery, implizit) — kein eigener Endpunkt-Eintrag | `RequestRouter.kt`; §1.2 |

### 3.2 Navigation (NavHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/references` | POST | Semantische Verwendungen eines Symbols im Projekt | §2.1 + Appendix | `NavHandlers.kt`; `RequestRouter.kt` |
| `/definition` | POST | Sprung zur Definition | §2.1 + Appendix | `NavHandlers.kt`; `RequestRouter.kt` |
| `/implementations` | POST | Implementierungen / Overrides | §2.1 + Appendix | `NavHandlers.kt`; `RequestRouter.kt` |
| `/declaration` | POST | Deklaration (≡ definition, Backing-B-only) | §2.1 + Appendix | `NavHandlers.kt`; `RequestRouter.kt` |

### 3.3 Struktur (StructureHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/type_hierarchy` | POST | Super-/Subtypen-Baum (Backing-B-only) | §2.2 + Appendix | `StructureHandlers.kt`; `RequestRouter.kt` |
| `/symbols_overview` | POST | Top-Level-Symbole einer Datei (headless-Fallback) | §2.2 + Appendix | `StructureHandlers.kt`; `RequestRouter.kt` |

### 3.4 Inspektionen (InspectionHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/inspections` | POST | Inspektionen auf einer Datei ausführen (`mode=run`) | §2.3 + Appendix | `InspectionHandlers.kt`; `RequestRouter.kt` |
| `/list_inspections` | POST | Aktivierte Inspektionen auflisten (`mode=list`) | §2.3 + Appendix | `InspectionHandlers.kt`; `RequestRouter.kt` |

### 3.5 Symbol-Body-Edits (EditHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/replaceSymbolBody` | POST | Symbol-Rumpf ersetzen (WriteCommandAction) | §2.4 + Appendix | `EditHandlers.kt`; `RequestRouter.kt` |
| `/insertBeforeSymbol` | POST | Geschwister-Element vor Symbol einfügen | §2.4 + Appendix | `EditHandlers.kt`; `RequestRouter.kt` |
| `/insertAfterSymbol` | POST | Geschwister-Element nach Symbol einfügen | §2.4 + Appendix | `EditHandlers.kt`; `RequestRouter.kt` |

### 3.6 Refactoring — Rename (RefactorHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/renamePreview` | POST | Rename Phase 1: Verwendungen + Konflikte sammeln, `plan_hash` bilden | §3.1 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |
| `/renameApply` | POST | Rename Phase 2: Multi-File-Transaktion ausführen | §3.1 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |

### 3.7 Refactoring — Reformat (RefactorHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/reformat` | POST | Datei in-place nach IDE-Code-Style formatieren (single-phase) | §3.2 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |

### 3.8 Refactoring — Move (RefactorHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/movePreview` | POST | Move Phase 1: Betroffene Dateien + Konflikte, `plan_hash` | §3.3 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |
| `/moveApply` | POST | Move Phase 2: Symbol + Referenzen verschieben | §3.3 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |

### 3.9 Refactoring — Safe Delete (RefactorHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/safeDeletePreview` | POST | Safe Delete Phase 1: Usages als Konflikte melden | §3.4 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |
| `/safeDeleteApply` | POST | Safe Delete Phase 2: Löschen oder CONFLICT | §3.4 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |

### 3.10 Refactoring — Inline (RefactorHandlers.kt)

| Endpunkt | Methode | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| `/inlinePreview` | POST | Inline Phase 1: Aufrufstellen + Konflikte | §3.5 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |
| `/inlineApply` | POST | Inline Phase 2: Symbol durch Rumpf ersetzen, Deklaration entfernen | §3.5 + Appendix | `RefactorHandlers.kt`; `RequestRouter.kt` |

## 4. Utility-Komponenten (kein eigener Endpunkt)

| Komponente | Klasse / Datei | Beschreibung | In Doku §X? | Quelle |
|---|---|---|---|---|
| ANSI-Strip-Util | `AnsiText.kt` (`stripAnsi`) | Entfernt ANSI-CSI-Sequenzen aus CLI-Ausgabe vor Messages-Popup-Anzeige | **GAP** — nicht dokumentiert | `util/AnsiText.kt`; verwendet in `LeanCtxActions.kt` |
| Binary Resolver | `BinaryResolver.kt` | Sucht + validiert lean-ctx-Binärdatei; führt Sub-Commands aus | §1 (implizit in Startup) — kein eigener Abschnitt | `BinaryResolver.kt` |
| Stats Reader | `StatsReader.kt` | Liest Token-Savings-Statistiken für Status-Bar-Widget + Gain-Panel | **GAP** — nicht dokumentiert | `StatsReader.kt`; `LeanCtxStatusBarFactory.kt` |
| Port File Heartbeat / Watcher / Reaper | `PortFileHeartbeat.kt`, `PortFileWatcher.kt`, `StalePortFileReaper.kt` | Port-Datei-Lebenszyklus-Management | §1.2 (Port-Discovery) — interne Mechanik nicht dokumentiert | `server/*.kt` |

---

## 5. Zusammenfassung der Gaps

Folgende Funktionen sind im Code implementiert, aber **nicht** in `19-jetbrains-plugin-de.md` §0–8 oder `appendix-jetbrains-plugin-de.md` dokumentiert:

| Gap | Zuständige Folge-Task |
|---|---|
| **Gain Tool Window** — `LeanCtxGain`, `LeanCtxGainToolWindowFactory`, `GainPanel`, `GainPollController`, `GainService` | A3 |
| **Editor-Focus Reporter** — `EditorFocusReporter`, Registry-Key `leanctx.editor.signal.enabled`, `lean-ctx editor-signal --file` | A4 |
| **Status-Bar-Widget** — `LeanCtxStatusBarFactory`/`Widget`, Klick öffnet Gain Tool Window, 30s-Polling-Timer | A5 |
| **Tools-Menü-Actions** — `LeanCtx.Setup`, `LeanCtx.Doctor`, `LeanCtx.Gain`, `LeanCtx.Dashboard`; `LeanCtxCommandAction`-Basis | A5 |
| **K2-Modus** — `supportsKotlinPluginMode supportsK2="true"` | A5 |
| **ANSI-Strip-Util** — `stripAnsi` in `util/AnsiText.kt`; verhindert Rendering-Artefakte in Swing-Dialogen | A5 |

### Vollständig dokumentiert (§0–8 + Appendix)

Navigation (`/references`, `/definition`, `/implementations`, `/declaration`),
Struktur (`/type_hierarchy`, `/symbols_overview`),
Inspektionen (`/inspections`, `/list_inspections`),
Symbol-Body-Edits (`/replaceSymbolBody`, `/insertBeforeSymbol`, `/insertAfterSymbol`),
Refactoring (`/renamePreview`+`/renameApply`, `/reformat`, `/movePreview`+`/moveApply`,
`/safeDeletePreview`+`/safeDeleteApply`, `/inlinePreview`+`/inlineApply`),
Architektur (Backing A/B/Headless, Port-Discovery-Konzept, PathJail, BLAKE3-Guard, Auth, Fehler-Katalog).

---

## 6. Zeilen-Vollständigkeitsprüfung (Step 4)

**Extensions (plugin.xml):** 5 Zeilen (statusBarWidgetFactory, postStartupActivity, toolWindow, registryKey, supportsK2) — alle in Abschnitt 1. ✓

**Actions (plugin.xml):** 4 Zeilen (Setup, Doctor, Gain, Dashboard) — alle in Abschnitt 2. ✓

**HTTP-Endpunkte (RequestRouter.kt):** 21 Routen:
- GET /health (1)
- POST Nav: /references, /definition, /implementations, /declaration (4)
- POST Struktur: /type_hierarchy, /symbols_overview (2)
- POST Inspektionen: /inspections, /list_inspections (2)
- POST Edits: /replaceSymbolBody, /insertBeforeSymbol, /insertAfterSymbol (3)
- POST Refactoring: /renamePreview, /renameApply, /reformat, /movePreview, /moveApply, /safeDeletePreview, /safeDeleteApply, /inlinePreview, /inlineApply (9)

Alle 21 Endpunkte in Abschnitt 3. ✓

**Utility-Komponenten:** 4 Zeilen (AnsiText, BinaryResolver, StatsReader, Port-File-Mechanik) — alle in Abschnitt 4. ✓

**Gesamtzeilen:** 5 + 4 + 21 + 4 = **34 Einträge** — keine Funktion fehlt. ✓
