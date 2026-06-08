# JetBrains Port-File Cleanup + Self-Healing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the JetBrains plugin remove stale `jetbrains-*.port` files of dead IDEs and robustly re-create its own port file if it disappears at runtime — without touching Rust.

**Architecture:** Five small, independently testable Kotlin units in `com.leanctx.plugin.server` (`ProcessLiveness`, `PortFileReader`, `StalePortFileReaper`, `PortFileWatcher`, `PortFileHeartbeat`), wired into the existing `BackendHttpServer` lifecycle. A `WatchService` reacts immediately to deletion of the own file; a 30s heartbeat is the fallback (missed events) and the periodic stale-cleanup tick. Liveness is pid-only via `ProcessHandle.of(pid)` (cross-platform, no `/proc`). Rust's `pid_alive`/`health_ok` stays the final validation — no Rust change, no new wire format.

**Tech Stack:** Kotlin (JVM 21), IntelliJ Platform Gradle plugin, JUnit 4 (`junit:junit:4.13.2`), `BasePlatformTestCase` for platform-bound tests, `AppExecutorUtil` for scheduling. No new runtime dependency (port-file JSON is hand-written/parsed; gson stays `compileOnly`).

---

## Background — Files You Will Touch

All paths are relative to `packages/jetbrains-lean-ctx/`.

**Existing main sources (read these first for patterns):**
- `src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt` — owner of token/port/portFile/lifecycle (`Disposable`). Integration target (§5.6 of the spec).
- `src/main/kotlin/com/leanctx/plugin/server/PortFileWriter.kt` — `PortFileData` data class + atomic `write` (temp + `ATOMIC_MOVE`, 0600) + `delete`. Emits flat snake_case JSON wrapped in `{ }`.
- `src/main/kotlin/com/leanctx/plugin/server/LeanCtxPaths.kt` — `portFile(dataDir, projectRoot)` → `dataDir/jetbrains-<projecthash>.port`.

**Existing tests (mirror their style):**
- `src/test/kotlin/com/leanctx/plugin/server/PortFileWriterTest.kt` — plain JUnit, `Files.createTempDirectory`, `assertTrue`/`assertEquals`.
- `src/test/kotlin/com/leanctx/plugin/server/BackendHttpServerTest.kt` — extends `BasePlatformTestCase` (platform required), `try { … } finally { server.dispose() }`.

**Key facts to rely on:**
- `PortFileData(port: Int, token: String, pid: Long, projectRoot: String, ideVersion: String, startedAt: Long)`.
- Port-file JSON keys are snake_case: `{"port":N,"token":"…","pid":N,"project_root":"…","ide_version":"…","started_at":N}`.
- `PortFileWriter.write(target, data)` is atomic and idempotent — safe to call concurrently from watcher + heartbeat.
- `LeanCtxPaths.portFile(dataDir, projectRoot)` returns `dataDir.resolve("jetbrains-<hash>.port")` — own-file path equality works against `DirectoryStream` entries resolved against the same `dataDir`.
- `gson` is `compileOnly` → NOT available at plugin runtime. The reader MUST parse without gson (regex), matching the writer's hand-rolled JSON.

**How to run tests** (working directory `packages/jetbrains-lean-ctx/`, via `ctx_shell` with `cwd=`):
- Single class: `./gradlew test --tests "com.leanctx.plugin.server.ProcessLivenessTest"`
- Full suite: `./gradlew test`

A test that references a not-yet-created class fails at **compilation** ("unresolved reference") — that is the valid TDD red state.

---

## Task 1: `ProcessLiveness`

Single pid-only liveness helper (spec §5.1, D5). Cross-platform via `ProcessHandle`.

**Files:**
- Create: `src/main/kotlin/com/leanctx/plugin/server/ProcessLiveness.kt`
- Test: `src/test/kotlin/com/leanctx/plugin/server/ProcessLivenessTest.kt`

- [ ] **Step 1: Write the failing test**

Create `src/test/kotlin/com/leanctx/plugin/server/ProcessLivenessTest.kt`:

```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ProcessLivenessTest {
    @Test
    fun currentProcessIsAlive() {
        val pid = ProcessHandle.current().pid()
        assertTrue(ProcessLiveness.isAlive(pid))
    }

    @Test
    fun absurdlyHighPidIsDead() {
        // No supported OS allocates a pid this large.
        assertFalse(ProcessLiveness.isAlive(Long.MAX_VALUE))
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./gradlew test --tests "com.leanctx.plugin.server.ProcessLivenessTest"`
Expected: FAIL — compilation error "unresolved reference: ProcessLiveness".

- [ ] **Step 3: Write minimal implementation**

Create `src/main/kotlin/com/leanctx/plugin/server/ProcessLiveness.kt`:

```kotlin
package com.leanctx.plugin.server

/**
 * Single pid-only liveness helper used by the reaper (spec §5.1, D5).
 * Cross-platform via ProcessHandle — no Linux /proc dependency.
 */
object ProcessLiveness {
    /** True if a process with this pid currently exists. */
    fun isAlive(pid: Long): Boolean = ProcessHandle.of(pid).isPresent
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.ProcessLivenessTest"`
Expected: PASS (2 tests).

- [ ] **Step 5: Reformat + commit**

Reformat the two changed files (`mcp__jetbrains__reformat_file` on each, per project rule before `git add`), then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/ProcessLiveness.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/ProcessLivenessTest.kt
git commit -m "feat(jetbrains): add ProcessLiveness pid-only liveness helper"
```

---

## Task 2: `PortFileReader`

Counterpart to `PortFileWriter` — extracts `pid` from a port file, fault-tolerant, no runtime JSON dependency (spec §5.2).

**Files:**
- Create: `src/main/kotlin/com/leanctx/plugin/server/PortFileReader.kt`
- Test: `src/test/kotlin/com/leanctx/plugin/server/PortFileReaderTest.kt`

- [ ] **Step 1: Write the failing test**

Create `src/test/kotlin/com/leanctx/plugin/server/PortFileReaderTest.kt`:

```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.nio.file.Files

class PortFileReaderTest {
    @Test
    fun roundTripsPidWrittenByPortFileWriter() {
        val dir = Files.createTempDirectory("lc-rd")
        val target = dir.resolve("jetbrains-abc.port")
        PortFileWriter.write(
            target,
            PortFileData(
                port = 1234, token = "tok", pid = 9988L,
                projectRoot = "/p", ideVersion = "IC-2026.1.3", startedAt = 1L
            )
        )
        assertEquals(9988L, PortFileReader.readPid(target))
    }

    @Test
    fun malformedFileYieldsNull() {
        val dir = Files.createTempDirectory("lc-rd2")
        val target = dir.resolve("jetbrains-broken.port")
        Files.writeString(target, "not json at all {{{")
        assertNull(PortFileReader.readPid(target))
    }

    @Test
    fun missingFileYieldsNull() {
        val dir = Files.createTempDirectory("lc-rd3")
        assertNull(PortFileReader.readPid(dir.resolve("nope.port")))
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileReaderTest"`
Expected: FAIL — "unresolved reference: PortFileReader".

- [ ] **Step 3: Write minimal implementation**

Create `src/main/kotlin/com/leanctx/plugin/server/PortFileReader.kt`:

```kotlin
package com.leanctx.plugin.server

import java.nio.file.Files
import java.nio.file.Path

/**
 * Counterpart to PortFileWriter (spec §5.2). Extracts the pid from a port file
 * without a runtime JSON dependency (gson is compileOnly), matching the writer's
 * hand-rolled snake_case JSON. Fault-tolerant: any unreadable or malformed file
 * yields null — never throws to the caller.
 */
object PortFileReader {
    private val PID_REGEX = Regex("\"pid\"\\s*:\\s*(\\d+)")

    /** pid from the snake_case port-file JSON, or null if missing/unreadable/malformed. */
    fun readPid(path: Path): Long? = try {
        val json = Files.readString(path)
        PID_REGEX.find(json)?.groupValues?.get(1)?.toLongOrNull()
    } catch (_: Exception) {
        null
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileReaderTest"`
Expected: PASS (3 tests).

- [ ] **Step 5: Reformat + commit**

Reformat both changed files, then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/PortFileReader.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/PortFileReaderTest.kt
git commit -m "feat(jetbrains): add PortFileReader (pid extraction, fault-tolerant)"
```

---

## Task 3: `StalePortFileReaper`

Scans `dataDir/jetbrains-*.port`, deletes dead-pid foreign files, keeps own/alive/non-port/malformed (spec §5.3, §6, D2).

**Files:**
- Create: `src/main/kotlin/com/leanctx/plugin/server/StalePortFileReaper.kt`
- Test: `src/test/kotlin/com/leanctx/plugin/server/StalePortFileReaperTest.kt`

- [ ] **Step 1: Write the failing test**

Create `src/test/kotlin/com/leanctx/plugin/server/StalePortFileReaperTest.kt`:

```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Files
import java.nio.file.Path

class StalePortFileReaperTest {
    private fun writePort(dir: Path, hash: String, pid: Long): Path {
        val p = dir.resolve("jetbrains-$hash.port")
        PortFileWriter.write(p, PortFileData(1, "t", pid, "/r", "v", 1L))
        return p
    }

    @Test
    fun deletesDeadKeepsAliveOwnAndNonPort() {
        val dir = Files.createTempDirectory("lc-reap")
        val deadPid = Long.MAX_VALUE
        val alivePid = ProcessHandle.current().pid()

        val dead = writePort(dir, "dead", deadPid)
        val aliveOther = writePort(dir, "other", alivePid)
        // own file carries a dead pid on purpose — it must survive via the path skip.
        val own = writePort(dir, "own", deadPid)
        val nonPort = dir.resolve("stats.json")
        Files.writeString(nonPort, "{}")

        StalePortFileReaper(dir, own).reap()

        assertFalse("dead foreign port file removed", Files.exists(dead))
        assertTrue("live foreign port file kept", Files.exists(aliveOther))
        assertTrue("own port file kept even with dead pid", Files.exists(own))
        assertTrue("non-port file untouched", Files.exists(nonPort))
    }

    @Test
    fun keepsMalformedFile() {
        val dir = Files.createTempDirectory("lc-reap2")
        val broken = dir.resolve("jetbrains-broken.port")
        Files.writeString(broken, "garbage")
        val own = dir.resolve("jetbrains-own.port")

        StalePortFileReaper(dir, own).reap()

        assertTrue("unparsable file conservatively kept", Files.exists(broken))
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./gradlew test --tests "com.leanctx.plugin.server.StalePortFileReaperTest"`
Expected: FAIL — "unresolved reference: StalePortFileReaper".

- [ ] **Step 3: Write minimal implementation**

Create `src/main/kotlin/com/leanctx/plugin/server/StalePortFileReaper.kt`:

```kotlin
package com.leanctx.plugin.server

import java.nio.file.Files
import java.nio.file.Path

/**
 * Scans dataDir for jetbrains-*.port files and deletes those whose owning process
 * is dead — pid-only liveness (D2). The own file is skipped explicitly (path skip,
 * §5.3) and is anyway protected because the own pid is alive. Malformed files
 * (pid unreadable) are conservatively kept — no data loss from a parse error.
 * Best-effort: a single read/delete failure never aborts the scan (§5.3, §6).
 */
class StalePortFileReaper(
    private val dataDir: Path,
    private val ownPortFile: Path,
) {
    fun reap() {
        val stream = try {
            Files.newDirectoryStream(dataDir, "jetbrains-*.port")
        } catch (_: Exception) {
            return
        }
        stream.use { entries ->
            for (entry in entries) {
                if (entry == ownPortFile) continue
                val pid = PortFileReader.readPid(entry) ?: continue // keep unparsable
                if (!ProcessLiveness.isAlive(pid)) {
                    PortFileWriter.delete(entry)
                }
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.StalePortFileReaperTest"`
Expected: PASS (2 tests).

- [ ] **Step 5: Reformat + commit**

Reformat both changed files, then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/StalePortFileReaper.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/StalePortFileReaperTest.kt
git commit -m "feat(jetbrains): add StalePortFileReaper (dead-pid cleanup, own-file safe)"
```

---

## Task 4: `PortFileWatcher`

`WatchService` on `dataDir` for `ENTRY_DELETE`; fires a callback when the own file is deleted (spec §5.4).

**Files:**
- Create: `src/main/kotlin/com/leanctx/plugin/server/PortFileWatcher.kt`
- Test: `src/test/kotlin/com/leanctx/plugin/server/PortFileWatcherTest.kt`

- [ ] **Step 1: Write the failing test**

Create `src/test/kotlin/com/leanctx/plugin/server/PortFileWatcherTest.kt`:

```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Files
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class PortFileWatcherTest {
    @Test
    fun firesOnOwnFileDelete() {
        val dir = Files.createTempDirectory("lc-watch")
        val own = dir.resolve("jetbrains-own.port")
        Files.writeString(own, "{}")
        val latch = CountDownLatch(1)

        val watcher = PortFileWatcher(dir, own) { latch.countDown() }
        try {
            // Give the watch thread a moment to register before mutating.
            Thread.sleep(200)
            Files.delete(own)
            assertTrue(
                "onOwnDeleted must fire within timeout",
                latch.await(10, TimeUnit.SECONDS)
            )
        } finally {
            watcher.close()
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileWatcherTest"`
Expected: FAIL — "unresolved reference: PortFileWatcher".

- [ ] **Step 3: Write minimal implementation**

Create `src/main/kotlin/com/leanctx/plugin/server/PortFileWatcher.kt`:

```kotlin
package com.leanctx.plugin.server

import java.io.Closeable
import java.nio.file.FileSystems
import java.nio.file.Path
import java.nio.file.StandardWatchEventKinds
import java.nio.file.WatchKey

/**
 * Watches dataDir for ENTRY_DELETE and invokes onOwnDeleted when the own port file
 * disappears, enabling immediate self-healing re-write (spec §5.4). Owns a single
 * daemon thread; close() shuts the WatchService and ends the thread.
 *
 * Note: an atomic re-write (temp + ATOMIC_MOVE into dataDir) raises CREATE/MODIFY,
 * not DELETE — so re-writing the own file does not re-trigger this watcher.
 */
class PortFileWatcher(
    private val dataDir: Path,
    private val ownPortFile: Path,
    private val onOwnDeleted: () -> Unit,
) : Closeable {
    private val watchService = FileSystems.getDefault().newWatchService()

    @Volatile
    private var running = true
    private val thread: Thread

    init {
        dataDir.register(watchService, StandardWatchEventKinds.ENTRY_DELETE)
        thread = Thread(::runLoop, "leanctx-port-watcher").apply {
            isDaemon = true
            start()
        }
    }

    private fun runLoop() {
        while (running) {
            val key: WatchKey = try {
                watchService.take()
            } catch (_: Exception) {
                return // closed or interrupted
            }
            for (event in key.pollEvents()) {
                val name = event.context() as? Path ?: continue
                if (dataDir.resolve(name) == ownPortFile) {
                    runCatching { onOwnDeleted() }
                }
            }
            if (!key.reset()) return
        }
    }

    override fun close() {
        running = false
        runCatching { watchService.close() }
        thread.interrupt()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileWatcherTest"`
Expected: PASS (1 test). (On some filesystems the polling `WatchService` can take a few seconds; the 10s latch timeout covers it.)

- [ ] **Step 5: Reformat + commit**

Reformat both changed files, then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/PortFileWatcher.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/PortFileWatcherTest.kt
git commit -m "feat(jetbrains): add PortFileWatcher (immediate self-heal on delete)"
```

---

## Task 5: `PortFileHeartbeat`

Periodic reap + self-heal fallback (spec §5.5, D3). The cleanup/self-heal cycle lives in a pure `tick()`; `start()`/`cancel()` only wrap the platform scheduler, so `tick()` is unit-testable without a running platform.

**Files:**
- Create: `src/main/kotlin/com/leanctx/plugin/server/PortFileHeartbeat.kt`
- Test: `src/test/kotlin/com/leanctx/plugin/server/PortFileHeartbeatTest.kt`

- [ ] **Step 1: Write the failing test**

Create `src/test/kotlin/com/leanctx/plugin/server/PortFileHeartbeatTest.kt`:

```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Files

class PortFileHeartbeatTest {
    @Test
    fun tickReWritesMissingOwnFile() {
        val dir = Files.createTempDirectory("lc-hb")
        val own = dir.resolve("jetbrains-own.port")
        // own file deliberately absent
        var reWrites = 0
        val hb = PortFileHeartbeat(
            reaper = StalePortFileReaper(dir, own),
            ownPortFile = own,
            reWrite = { reWrites++ },
        )

        hb.tick()

        assertEquals("reWrite invoked once when own file missing", 1, reWrites)
    }

    @Test
    fun tickKeepsExistingOwnFileAndReapsStale() {
        val dir = Files.createTempDirectory("lc-hb2")
        val own = dir.resolve("jetbrains-own.port")
        PortFileWriter.write(
            own,
            PortFileData(1, "t", ProcessHandle.current().pid(), "/r", "v", 1L)
        )
        val stale = dir.resolve("jetbrains-stale.port")
        PortFileWriter.write(stale, PortFileData(1, "t", Long.MAX_VALUE, "/r", "v", 1L))
        var reWrites = 0
        val hb = PortFileHeartbeat(
            reaper = StalePortFileReaper(dir, own),
            ownPortFile = own,
            reWrite = { reWrites++ },
        )

        hb.tick()

        assertTrue("own file kept", Files.exists(own))
        assertFalse("stale file reaped", Files.exists(stale))
        assertEquals("no reWrite when own file present", 0, reWrites)
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileHeartbeatTest"`
Expected: FAIL — "unresolved reference: PortFileHeartbeat".

- [ ] **Step 3: Write minimal implementation**

Create `src/main/kotlin/com/leanctx/plugin/server/PortFileHeartbeat.kt`:

```kotlin
package com.leanctx.plugin.server

import com.intellij.util.concurrency.AppExecutorUtil
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.ScheduledFuture
import java.util.concurrent.TimeUnit

/**
 * Periodic fallback to the watcher (covers missed events) plus a cleanup tick
 * (spec §5.5). Each tick reaps stale foreign files and re-writes the own file if
 * it vanished. Scheduling uses the platform AppExecutorUtil (D3: 30s default).
 *
 * tick() is pure logic, callable without a scheduler — that is what the unit
 * tests exercise; start()/cancel() only wrap the scheduling.
 */
class PortFileHeartbeat(
    private val reaper: StalePortFileReaper,
    private val ownPortFile: Path,
    private val reWrite: () -> Unit,
    private val intervalSeconds: Long = 30,
) {
    private var future: ScheduledFuture<*>? = null

    /** One cleanup + self-heal cycle. */
    fun tick() {
        reaper.reap()
        if (!Files.exists(ownPortFile)) reWrite()
    }

    fun start() {
        future = AppExecutorUtil.getAppScheduledExecutorService()
            .scheduleWithFixedDelay(
                { runCatching { tick() } },
                intervalSeconds, intervalSeconds, TimeUnit.SECONDS
            )
    }

    fun cancel() {
        future?.cancel(false)
        future = null
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.PortFileHeartbeatTest"`
Expected: PASS (2 tests).

- [ ] **Step 5: Reformat + commit**

Reformat both changed files, then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/PortFileHeartbeat.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/PortFileHeartbeatTest.kt
git commit -m "feat(jetbrains): add PortFileHeartbeat (30s reap + self-heal fallback)"
```

---

## Task 6: Integrate into `BackendHttpServer`

Wire reaper + watcher + heartbeat into the existing lifecycle (spec §5.6). Re-writes use the identical `port`/`token`/`pid`/`startedAt` (socket lives on → stable identity).

**Files:**
- Modify: `src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt`
- Test (extend): `src/test/kotlin/com/leanctx/plugin/server/BackendHttpServerTest.kt`

- [ ] **Step 1: Write the failing tests (extend `BackendHttpServerTest`)**

Add these two methods inside the existing `BackendHttpServerTest` class (after `testStartWritesPortFileAndServesHealth`), keeping the existing `import java.nio.file.Files` and adding `import java.nio.file.Path` if not present:

```kotlin
    fun testStartReapsStaleForeignPortFile() {
        val dataDir = Files.createTempDirectory("lc-srv2")
        // Seed a foreign stale file (dead pid) that must be reaped on boot.
        val stale = dataDir.resolve("jetbrains-stale.port")
        PortFileWriter.write(stale, PortFileData(1, "t", Long.MAX_VALUE, "/other", "v", 1L))
        val server = BackendHttpServer(
            dataDir = dataDir, project = project, projectRoot = "/some/project",
            ideVersion = "IC-2026.1.3", projectName = "demo", startedAt = 1L
        )
        try {
            server.start()
            assertFalse("stale foreign port file reaped on boot", Files.exists(stale))
            assertTrue(Files.exists(LeanCtxPaths.portFile(dataDir, "/some/project")))
        } finally {
            server.dispose()
        }
        // dispose() must stop watcher + heartbeat and remove our file (no leak).
        assertFalse(Files.exists(LeanCtxPaths.portFile(dataDir, "/some/project")))
    }

    fun testWatcherReWritesDeletedPortFile() {
        val dataDir = Files.createTempDirectory("lc-srv3")
        val server = BackendHttpServer(
            dataDir = dataDir, project = project, projectRoot = "/some/project",
            ideVersion = "IC-2026.1.3", projectName = "demo", startedAt = 1L
        )
        try {
            server.start()
            val pf = LeanCtxPaths.portFile(dataDir, "/some/project")
            assertTrue(Files.exists(pf))
            Files.delete(pf)
            // The watcher must re-create it.
            var restored = false
            val deadline = System.currentTimeMillis() + 10_000
            while (System.currentTimeMillis() < deadline) {
                if (Files.exists(pf)) {
                    restored = true
                    break
                }
                Thread.sleep(100)
            }
            assertTrue("watcher re-wrote the deleted port file", restored)
        } finally {
            server.dispose()
        }
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `./gradlew test --tests "com.leanctx.plugin.server.BackendHttpServerTest"`
Expected: FAIL — `testStartReapsStaleForeignPortFile` fails (stale file still present; no reap yet) and/or `testWatcherReWritesDeletedPortFile` fails (file not restored; no watcher yet).

- [ ] **Step 3: Add the new fields**

In `BackendHttpServer.kt`, after the existing `private var portFile: Path? = null` (line 27), add:

```kotlin
    private var portFileData: PortFileData? = null
    private var watcher: PortFileWatcher? = null
    private var heartbeat: PortFileHeartbeat? = null
```

- [ ] **Step 4: Replace the port-file block in `start()`**

In `start()`, replace the existing block (currently lines 55–67):

```kotlin
        val pf = LeanCtxPaths.portFile(dataDir, projectRoot)
        PortFileWriter.write(
            pf,
            PortFileData(
                port = http.address.port,
                token = token,
                pid = ProcessHandle.current().pid(),
                projectRoot = projectRoot,
                ideVersion = ideVersion,
                startedAt = startedAt,
            )
        )
        portFile = pf
```

with (spec §5.6 ordering: reap → write → watcher → heartbeat):

```kotlin
        val pf = LeanCtxPaths.portFile(dataDir, projectRoot)
        // 2. Stale-cleanup at boot, before writing our own file.
        val reaper = StalePortFileReaper(dataDir, pf)
        reaper.reap()

        // 3. Write our own port file. Re-writes reuse this exact identity.
        val data = PortFileData(
            port = http.address.port,
            token = token,
            pid = ProcessHandle.current().pid(),
            projectRoot = projectRoot,
            ideVersion = ideVersion,
            startedAt = startedAt,
        )
        PortFileWriter.write(pf, data)
        portFile = pf
        portFileData = data

        // 4. Watcher: immediate re-write if our file is deleted at runtime.
        watcher = PortFileWatcher(dataDir, pf, ::reWritePortFile)

        // 5. Heartbeat: periodic reap + self-heal fallback (30s).
        heartbeat = PortFileHeartbeat(reaper, pf, ::reWritePortFile).also { it.start() }
```

- [ ] **Step 5: Add the `reWritePortFile` helper**

In `BackendHttpServer.kt`, add this method (e.g. directly after `start()`'s closing brace, before `dispose()`):

```kotlin
    /** Re-write our port file with the stable identity (socket lives on). Atomic + idempotent. */
    private fun reWritePortFile() {
        val pf = portFile ?: return
        val data = portFileData ?: return
        PortFileWriter.write(pf, data)
    }
```

- [ ] **Step 6: Update `dispose()` to stop watcher + heartbeat first**

Replace the existing `dispose()` body (currently lines 70–78):

```kotlin
    override fun dispose() {
        server?.stop(0)
        server = null
        // HttpServer.stop() does not close a user-supplied executor; reclaim its threads now.
        executor?.shutdownNow()
        executor = null
        portFile?.let { PortFileWriter.delete(it) }
        portFile = null
    }
```

with (spec §5.6 dispose ordering: heartbeat → watcher → server/executor → delete):

```kotlin
    override fun dispose() {
        heartbeat?.cancel()
        heartbeat = null
        watcher?.close()
        watcher = null
        server?.stop(0)
        server = null
        // HttpServer.stop() does not close a user-supplied executor; reclaim its threads now.
        executor?.shutdownNow()
        executor = null
        portFile?.let { PortFileWriter.delete(it) }
        portFile = null
        portFileData = null
    }
```

- [ ] **Step 7: Run the full `BackendHttpServerTest` to verify it passes**

Run: `./gradlew test --tests "com.leanctx.plugin.server.BackendHttpServerTest"`
Expected: PASS — original health/token test plus `testStartReapsStaleForeignPortFile` and `testWatcherReWritesDeletedPortFile`.

- [ ] **Step 8: Reformat + commit**

Reformat both changed files, then:

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt \
        packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/BackendHttpServerTest.kt
git commit -m "feat(jetbrains): wire reaper + watcher + heartbeat into BackendHttpServer lifecycle"
```

---

## Task 7: Full Suite Green + runIde E2E Gate

Verify the whole plugin test suite is green and confirm the boot-reap behavior end-to-end in a real IDE sandbox (spec §9 — E2E gate analogous to Phase 2 T8).

**Files:** none (verification + final notes only).

- [ ] **Step 1: Run the full plugin test suite**

Run: `./gradlew test`
Expected: PASS (BUILD SUCCESSFUL) — all existing tests plus the five new test classes and the extended `BackendHttpServerTest`.

- [ ] **Step 2: runIde E2E gate — stale file reaped on next boot**

Manual verification (mirrors the Phase 2 T8 gate). The user runs the sandbox IDE — suggest typing in this session:

```text
! cd packages/jetbrains-lean-ctx && ./gradlew runIde
```

Verification checklist while/after the sandbox runs:
1. Open a project → confirm `~/.lean-ctx/jetbrains-<hash>.port` is written (snake_case, real pid).
2. Hard-kill the sandbox (`kill -9` of the runIde JVM) so `dispose()` never runs → the port file is intentionally left behind (stale).
3. Start `./gradlew runIde` again and open a project → confirm the previous stale file (dead pid) is **gone** (reaped at boot, step 2 of `start()`), while the freshly written own file is present.
4. Self-heal: with the IDE running, delete the own port file manually → confirm it reappears within a moment (watcher) or at most ~30s (heartbeat).

If any check fails, use superpowers:systematic-debugging before proceeding.

- [ ] **Step 3: Final confirmation**

Confirm: full suite green (step 1) + all four runIde checks pass (step 2). No Rust files changed (`git diff --name-only` shows only `packages/jetbrains-lean-ctx/**`). The feature is complete; commits from Tasks 1–6 are on `feat-jetbrains-plugin` (not merged — merge is a separate user decision, consistent with Phase 2).

---

## Self-Review (already applied)

- **Spec coverage:** §5.1→T1, §5.2→T2, §5.3→T3, §5.4→T4, §5.5→T5, §5.6→T6, §7 tests→T1–T6, §9 E2E gate→T7. Non-goals (§3) respected: no Rust change, no new wire format, no token rotation (re-write reuses stored `PortFileData`).
- **Type consistency:** `ProcessLiveness.isAlive(Long): Boolean`, `PortFileReader.readPid(Path): Long?`, `StalePortFileReaper(dataDir, ownPortFile).reap()`, `PortFileWatcher(dataDir, ownPortFile, onOwnDeleted).close()`, `PortFileHeartbeat(reaper, ownPortFile, reWrite, intervalSeconds=30).tick()/start()/cancel()` — names/signatures identical across tasks and integration.
- **No placeholders:** every step carries full code or an exact command + expected result.
- **Runtime-dependency check:** reader parses JSON via regex (no gson at runtime; gson stays `compileOnly`).
