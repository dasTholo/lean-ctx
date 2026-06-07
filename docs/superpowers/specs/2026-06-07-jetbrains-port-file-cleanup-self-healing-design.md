# JetBrains Port-Datei: Cleanup + Self-Healing — Design

- **Datum:** 2026-06-07
- **Branch:** feat-jetbrains-plugin
- **Scope:** Rein Kotlin-Plugin (`packages/jetbrains-lean-ctx/`). **Rust bleibt unverändert.**
- **Status:** Design genehmigt, bereit für Implementierungsplan

## 1. Kontext & Motivation

### 1.1 Zwei getrennte Kanäle (Klarstellung)

Die Umstellung des MCP-Transports von HTTP auf **globalen stdio** betrifft nur
**Kanal 1** und hat die Plugin-Integration **nicht** gebrochen:

- **Kanal 1 — MCP-Transport: Claude Code ↔ lean-ctx.** stdio (Eltern-Kind-Pipe),
  keine Auth nötig. Das ist, was umgestellt wurde.
- **Kanal 2 — Backend-Kanal: lean-ctx ↔ JetBrains-Plugin.** Das Plugin ist
  HTTP-**Server** auf `127.0.0.1:<zufallsport>` (`BackendHttpServer.kt`); lean-ctx
  (Rust) ist der HTTP-**Client**, der das Plugin über eine Port-Datei findet
  (`port_discovery.rs:44`) und PSI-Daten/Refactorings abruft.

Kanal 2 **muss** ein localhost-Socket sein (zwei unverwandte Prozesse: Rust-Binary
vs. IDE-JVM, keine stdio-Pipe möglich) und behält daher zwingend den
`X-LeanCtx-Token` — er schützt den localhost-Port davor, dass fremde lokale
Prozesse PSI-Daten abgreifen. Der Token ist transport-agnostisch und entfällt mit
stdio **nicht**.

Port-Discovery ist transport-agnostisch: `port_file_path = data_dir +
jetbrains-<projecthash>.port`, abhängig nur von `data_dir` + `project_root`.
`data_dir` löst auf beiden Seiten (`core/data_dir.rs`, `LeanCtxPaths.kt`)
byte-identisch auf (hier: legacy `~/.lean-ctx`, deterministisch & env-robust).

### 1.2 Das eigentliche Problem

Beobachtung: 16 `jetbrains-*.port`-Dateien in `~/.lean-ctx/`. Erklärung: eine Datei
**pro Projekt-Pfad** (`projecthash = sha256(canonical(project_root))[..16]`), die je
mit installiertem Plugin geöffnet wurde (inkl. `runIde`-Sandboxes).

Lebenszyklus heute (`BackendHttpServer.kt`):
- **Geschrieben:** genau **einmal** in `start()` beim Projekt-Öffnen
  (`LeanCtxStartupActivity`, `ProjectActivity`). Kein Timer, kein Heartbeat.
- **Gelöscht:** in `dispose()` via `Disposer.register(project, server)` — nur bei
  **graceful** `projectClosing`.

Daraus zwei Lücken:

1. **Kein Stale-Cleanup.** Bei Hart-Kill (IDE-Crash, `kill -9`, Gradle-Sandbox-Stop)
   läuft `dispose()` nie → die Datei bleibt als Stale liegen. Niemand räumt je auf,
   kein TTL. Funktional harmlos (lean-ctx validiert pro Lookup via
   `pid_alive`/`health_ok` und fasst fremde Dateien nie an), aber unhygienisch und
   unbegrenzt wachsend.
2. **Kein Self-Healing.** Die Datei wird nur beim Projekt-Öffnen geschrieben.
   Verschwindet sie zur Laufzeit (manuelles Löschen, Cleanup einer anderen
   Instanz), kehrt sie erst beim nächsten Projekt-Öffnen zurück — bis dahin findet
   lean-ctx dieses Plugin nicht und fällt auf den externen LSP-Backend zurück.

## 2. Ziel

Beide Lücken plugin-seitig schließen, ohne Rust anzufassen:
- Stale-Port-Dateien toter IDEs automatisch entfernen.
- Eigene Port-Datei robust wiederherstellen, falls sie zur Laufzeit verschwindet.

## 3. Nicht-Ziele (YAGNI)

- Keine Rust-Änderung (`pid_alive`/`health_ok` bleiben finale Validierung).
- Kein redundanter Cleanup auf Rust-Seite.
- Kein neues Wire-/Port-Datei-Format (snake_case-JSON bleibt).
- Kein Token-Rotation/-Erneuerung beim Re-Write (Identität stabil, s. §5.3).

## 4. Architektur-Entscheidungen

| # | Entscheidung | Begründung |
|---|--------------|-----------|
| D1 | **Watcher + Heartbeat** für Self-Healing | Watcher: sofortige Reaktion auf Löschung; Heartbeat: Fallback (verpasste Events) + Cleanup-Tick. Höchste Robustheit. |
| D2 | **pid-only** Liveness fürs Cleanup (kein HTTP-`/health`) | Billig, kein Netzwerk, kein Token-Lesen fremder Dateien. Rust macht ohnehin finalen `health_ok`. pid-Recycling vernachlässigbar & harmlos. |
| D3 | **30s** Heartbeat-Intervall | Balance aus Reaktionszeit (Fallback) und IO-Last. |
| D4 | Logik im **`BackendHttpServer`** gebündelt (Owner von token/port/portFile/Lifecycle) | Natürlicher Owner; Disposable-Lifecycle vorhanden. |
| D5 | `ProcessHandle.of(pid)` statt Linux-`/proc` | Cross-platform; robuster als Rusts plattformspezifischer Check. |

## 5. Komponenten (klein, isoliert, je testbar)

### 5.1 `ProcessLiveness`
`isAlive(pid: Long): Boolean` via `ProcessHandle.of(pid).isPresent`. Einziger
Liveness-Helfer; von Reaper genutzt.

### 5.2 `PortFileReader`
Gegenstück zu `PortFileWriter`. Extrahiert mindestens `pid` (bei Bedarf weitere
Felder) aus einer Port-Datei. Round-trip-kompatibel zum manuellen snake_case-JSON
des Writers. Fehlertolerant: unlesbare/kaputte Datei → `null` (nie Exception nach
außen).

### 5.3 `StalePortFileReaper`
- Scannt `dataDir/jetbrains-*.port`.
- Liest `pid` via `PortFileReader`, löscht Datei wo `!ProcessLiveness.isAlive(pid)`.
- Lässt die **eigene** Datei explizit aus — doppelt geschützt: (a) eigener pid lebt,
  (b) Pfad-Skip der eigenen `portFile`.
- Kaputte/unparsbare Datei: konservativ **nicht** löschen (kein Datenverlust durch
  Parse-Fehler); optional später als eigene Heuristik.
- Best-effort: einzelne Lösch-/Lesefehler brechen den Scan nicht ab.

### 5.4 `PortFileWatcher` (`Closeable`)
- `WatchService` auf `dataDir`, registriert `ENTRY_DELETE`.
- Bei Delete-Event der **eigenen** Datei → sofortiges Re-Write (Callback auf den
  Server bzw. übergebenes Re-Write-Lambda).
- Eigener Lifecycle/Thread; `close()` beendet sauber.

### 5.5 `PortFileHeartbeat`
- Scheduled über `AppExecutorUtil` (Plattform-Scheduler), Intervall 30s (D3).
- Tick: `reaper.reap()` **und** eigene Datei wiederherstellen, falls fehlend.
- Fallback zum Watcher (deckt verpasste Events ab).
- `cancel()` stoppt den ScheduledFuture.

### 5.6 Integration in `BackendHttpServer`
`start()`-Reihenfolge:
1. `http.start()`, `server` gesetzt.
2. **reap einmal** (Stale-Cleanup beim Boot).
3. eigene Port-Datei schreiben (wie heute).
4. `PortFileWatcher` starten.
5. `PortFileHeartbeat` schedulen.

`dispose()`-Reihenfolge (zusätzlich zu heute):
1. Heartbeat `cancel()`.
2. Watcher `close()`.
3. `server.stop(0)` + `executor.shutdownNow()` (wie heute).
4. eigene Port-Datei löschen (wie heute).

Re-Write nutzt **identische** `port`/`token`/`pid`/`startedAt` (Socket lebt weiter →
Identität stabil). `PortFileWriter.write` ist atomar (temp + `ATOMIC_MOVE`, 0600) →
Watcher und Heartbeat können gefahrlos gleichzeitig schreiben (idempotent).

## 6. Nebenläufigkeit & Races (selbstheilend)

- **Mehrere IDEs:** jede reapt nur **fremde tote** pids; lebende Dateien bleiben.
  Eigene Datei nie gelöscht (D2-Schutz, §5.3).
- **Reap trifft startende Instanz** (Datei noch nicht/gerade geschrieben): deren
  Watcher/Heartbeat stellt sofort wieder her.
- **Watcher + Heartbeat schreiben gleichzeitig:** atomarer Write → idempotent, kein
  Schaden.

## 7. Tests (JUnit, wie Bestand in `src/test/kotlin/.../server/`)

| Test | Prüft |
|------|-------|
| `ProcessLivenessTest` | aktueller pid lebt; absurd hoher pid tot |
| `PortFileReaderTest` | pid-Round-trip mit `PortFileWriter`; kaputte Datei → `null` |
| `StalePortFileReaperTest` | tote Datei gelöscht; lebende (self) + Nicht-Port-Dateien unberührt; kaputte Datei bleibt |
| `PortFileWatcherTest` | Delete der eigenen Datei → wiederhergestellt |
| `PortFileHeartbeatTest` | fehlende Datei → wiederhergestellt; Tick reapt Stale |
| `BackendHttpServerTest` (erweitern) | `dispose()` stoppt Watcher+Heartbeat ohne Leaks; Datei gelöscht |

## 8. Betroffene Dateien

**Neu (main):**
- `.../server/ProcessLiveness.kt`
- `.../server/PortFileReader.kt`
- `.../server/StalePortFileReaper.kt`
- `.../server/PortFileWatcher.kt`
- `.../server/PortFileHeartbeat.kt`

**Geändert (main):**
- `.../server/BackendHttpServer.kt` (Integration, §5.6)

**Neu/geändert (test):** entsprechend §7.

**Rust:** keine.

## 9. Offene Punkte

- Heartbeat-Intervall (30s) ggf. später konfigurierbar — vorerst Konstante.
- `runIde`-Sandbox-Verifikation: Stale-Datei eines hart-gekillten Vorlaufs wird beim
  nächsten Boot gereapt (E2E-Gate analog Phase 2 T8).
