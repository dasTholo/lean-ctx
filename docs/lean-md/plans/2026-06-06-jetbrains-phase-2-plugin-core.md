# JetBrains Phase 2 — Plugin-Kern (HTTP-Lifecycle) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Das bestehende `com.leanctx.plugin` Companion-Plugin um einen localhost-HTTP-Server pro `Project` erweitern, der sich via Port-/Token-Datei an die (Phase-1-)Rust-Seite meldet und token-geschütztes `/health` beantwortet — plus Build-Modernisierung auf IC 2026.1 / Kotlin 2.3.20 und den Phase-1-Begleit-Fix `port_file_path → lean_ctx_data_dir()`.

**Architecture:** Plugin ist HTTP-**Server** (`com.sun.net.httpserver.HttpServer`, `127.0.0.1:0`), den lean-ctx-Rust als HTTP-**Client** aufruft. Token generiert das Plugin (`SecureRandom`), schreibt es inline in `<data_dir>/jetbrains-<hash>.port` (`0600`, atomar); Rust liest es und sendet es als `X-LeanCtx-Token`. Noch **keine** PSI-Logik (Phase 3+). Server-Lifecycle hängt als `Disposable` am `Project` (auto-cleanup bei `projectClosing`).

**Tech Stack:** Kotlin 2.3.20, IntelliJ Platform Gradle Plugin 2.16.0, IntelliJ IDEA 2026.1.3 (bündelt Kotlin 2.3.20), Gradle 9.5.0, JDK 21 (`/usr/lib/jvm/java-21-openjdk`), JUnit4 (plain-JVM-Tests); Rust-Seite: `cargo nextest`.

**Spec:** `docs/lean-md/specs/2026-06-05-leanctx-jetbrains-psi-backend-design.md` (§5.5, §15, §15.7).

---

## Hard Rules (gelten für JEDEN Task)

- **Rust (`*.rs`) Edits NUR via Serena** (`mcp__serena__jet_brains_find_symbol`, `replace_symbol_body`, `insert_after_symbol`, `replace_content`) — nie native `Edit`/`ctx_edit`/`sed`.
- **Kotlin/Gradle-Dateien:** neue Dateien via `Write`; Änderungen via `mcp__lean-ctx__ctx_edit`.
- **Tests Rust:** immer `cargo nextest run`, nie `cargo test`.
- **Vor `git add`:** geänderte Dateien mit `mcp__jetbrains__reformat_file` formatieren (falls IDE-MCP verfügbar).
- **Gradle/`java`:** `java`/`gradlew` sind ggf. nicht in der lean-ctx-Shell-Allowlist. Falls `BLOCKED`: einmalig `lean-ctx allow java` ausführen, dann `bash ./gradlew …`. Alternativ führt der User die `./gradlew`-/`runIde`-Schritte im echten Terminal (`! ./gradlew …`) aus.
- **Commit-Disziplin:** je Task ein Commit für Review-Hygiene. **Optionaler Schluss-Schritt** (§12.3 des Specs „ein Commit pro Phase"): die Task-Commits am Phase-Gate zu **einem** Phase-2-Commit zusammenfassen.

---

## File Structure

**Rust (Phase-1-Begleit-Fix):**
- Modify: `rust/src/lsp/port_discovery.rs` — `port_file_path` nutzt `lean_ctx_data_dir()`; 2 neue Tests.

**Gradle-Build (Modernisierung):**
- Create: `packages/jetbrains-lean-ctx/gradle.properties`
- Rewrite: `packages/jetbrains-lean-ctx/settings.gradle.kts`
- Rewrite: `packages/jetbrains-lean-ctx/build.gradle.kts`
- Create (vom Template kopiert): `gradlew`, `gradlew.bat`, `gradle/wrapper/gradle-wrapper.jar`, `gradle/wrapper/gradle-wrapper.properties`

**Kotlin Plugin-Kern (neu, Package `com.leanctx.plugin.server`):**
- Create: `…/server/LeanCtxPaths.kt` — Data-Dir-Resolver + `projectHash` (rein, unit-getestet)
- Create: `…/server/PortFileWriter.kt` — atomares Schreiben/Löschen der `.port`-Datei (hand-rolled JSON)
- Create: `…/server/RequestRouter.kt` — Token-Check + Routing → `HttpResult` (rein, unit-getestet)
- Create: `…/server/BackendHttpServer.kt` — `Disposable`, bindet HttpServer, schreibt Port-Datei
- Modify: `…/LeanCtxStartupActivity.kt` — Server pro Project booten + an `Project` disposen
- Tests: `…/test/kotlin/com/leanctx/plugin/server/{LeanCtxPathsTest,PortFileWriterTest,RequestRouterTest,BackendHttpServerTest}.kt`

> **JSON hand-rolled (kein gson) in Phase 2:** Nur zwei flache Objekte (Port-Datei + Health/Error). Vermeidet die `compileOnly`-gson-Falle im plain-JVM-Test-Classpath. gson kommt erst in Phase 3 für die reicheren PSI-DTOs.
>
> **Wire-Contract (KRITISCH):** Die Port-Datei-JSON-Keys sind **snake_case** (`port`, `token`, `pid`, `project_root`, `ide_version`, `started_at`) — exakt die serde-Felder der Rust-`PortFile`-Struct in `port_discovery.rs`. NICHT camelCase.

---

## Task 1: Rust — Phase-1-Begleit-Fix `port_file_path → lean_ctx_data_dir()`

**Files:**
- Modify: `rust/src/lsp/port_discovery.rs:41-47` (`port_file_path`) + Modul-Doc oben + Tests-Modul
- Test: derselbe `#[cfg(test)] mod tests` in `port_discovery.rs`

Hintergrund: `port_file_path` hardcodet heute `dirs::home_dir().join(".lean-ctx")` und ignoriert damit `LEAN_CTX_DATA_DIR`/XDG. Der kanonische Resolver ist `crate::core::data_dir::lean_ctx_data_dir()` (`core/mod.rs:302 pub mod data_dir`).

- [ ] **Step 1: Failing-Tests einfügen** (Serena `insert_after_symbol` nach `port_file_absent_for_unlikely_root`)

Symbol-Pfad: `tests/port_file_absent_for_unlikely_root`. Einzufügender Code:

```rust

    #[test]
    fn project_hash_matches_known_vector() {
        // sha256("/some/project")[..8] — canonicalize fails (path absent) → raw fallback.
        // Shared parity anchor with the Kotlin LeanCtxPaths test.
        assert_eq!(project_hash("/some/project"), "a0317725f24b01df");
    }

    #[test]
    fn port_file_path_honors_data_dir_env() {
        let _lock = crate::core::data_dir::test_env_lock();
        let dir = std::env::temp_dir().join("lc_jb_portfile_env");
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("LEAN_CTX_DATA_DIR", dir.to_str().unwrap());
        let p = port_file_path("/some/project").unwrap();
        std::env::remove_var("LEAN_CTX_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(p, dir.join("jetbrains-a0317725f24b01df.port"));
    }
```

- [ ] **Step 2: Tests laufen lassen — Rot erwartet**

Run: `cargo nextest run -p lean-ctx port_discovery 2>&1`
Expected: `project_hash_matches_known_vector` PASS (Anker), `port_file_path_honors_data_dir_env` **FAIL** (port_file_path nutzt noch `~/.lean-ctx`, ignoriert das Env → Pfad ≠ erwartet).

- [ ] **Step 3: `port_file_path` umstellen** (Serena `replace_symbol_body`, Symbol `port_file_path`)

Neuer Symbol-Körper (inkl. Doc-Kommentar):

```rust
/// `<data_dir>/jetbrains-<projecthash>.port` — `<data_dir>` via
/// `core::data_dir::lean_ctx_data_dir()` (LEAN_CTX_DATA_DIR → ~/.lean-ctx → XDG),
/// NICHT hardcoded `~/.lean-ctx` (spec §5.5 / §15.5). Kotlin spiegelt diese Auflösung.
pub fn port_file_path(project_root: &str) -> Option<std::path::PathBuf> {
    let dir = crate::core::data_dir::lean_ctx_data_dir().ok()?;
    Some(dir.join(format!("jetbrains-{}.port", project_hash(project_root))))
}
```

- [ ] **Step 4: Modul-Doc oben anpassen** (Serena `replace_content`)

Old:
```rust
//! The plugin writes `~/.lean-ctx/jetbrains-<projecthash>.port` (JSON, 0600).
```
New:
```rust
//! The plugin writes `<data_dir>/jetbrains-<projecthash>.port` (JSON, 0600), where
//! `<data_dir>` = core::data_dir::lean_ctx_data_dir() (LEAN_CTX_DATA_DIR → ~/.lean-ctx → XDG).
```

- [ ] **Step 5: Tests laufen lassen — Grün erwartet**

Run: `cargo nextest run -p lean-ctx port_discovery 2>&1`
Expected: beide neuen Tests + die zwei bestehenden (`project_hash_is_stable_and_16_hex`, `port_file_absent_for_unlikely_root`) PASS.

- [ ] **Step 6: Voller Gate-Lauf + clippy**

Run: `cargo nextest run -p lean-ctx 2>&1` und `cargo clippy -p lean-ctx --all-targets 2>&1`
Expected: keine NEUEN Failures ggü. Baseline (Baseline = 3 env-leak pathjail-Failures, siehe knowledge `gate-baseline-pathjail-env-failures`); clippy für `port_discovery.rs` clean.

- [ ] **Step 7: Reformat + Commit**

Reformat: `mcp__jetbrains__reformat_file` auf `rust/src/lsp/port_discovery.rs` (falls IDE-MCP verfügbar; sonst `cargo fmt -p lean-ctx`).
```bash
git add rust/src/lsp/port_discovery.rs
git commit -m "fix(lsp): port_file_path uses lean_ctx_data_dir() not hardcoded ~/.lean-ctx (Phase-2 prep)"
```

---

## Task 2: Gradle-Build modernisieren (IC 2026.1 / Kotlin 2.3.20 + Wrapper)

**Files:**
- Create: `packages/jetbrains-lean-ctx/gradle.properties`
- Rewrite: `packages/jetbrains-lean-ctx/settings.gradle.kts`
- Rewrite: `packages/jetbrains-lean-ctx/build.gradle.kts`
- Create: Wrapper-Dateien (vom JetBrains-Template, Gradle 9.5.0)

Kontext: Aktuell alte DSL (`create("IC","2024.1")`, Plugin-Versionen inline, `kotlinOptions.jvmTarget="17"`), **kein** `gradle.properties`, **kein** Gradle-Wrapper.

- [ ] **Step 1: `gradle.properties` anlegen** (`Write`)

`packages/jetbrains-lean-ctx/gradle.properties`:
```properties
group = com.leanctx
version = 1.0.0

# Kotlin stdlib is provided by the IDE at runtime — do not bundle it.
kotlin.stdlib.default.dependency = false

org.gradle.configuration-cache = true
org.gradle.caching = true
```

- [ ] **Step 2: `settings.gradle.kts` ersetzen** (`ctx_edit`, ganze Datei)

old_string (komplett):
```kotlin
rootProject.name = "lean-ctx"
```
new_string:
```kotlin
import org.jetbrains.intellij.platform.gradle.extensions.intellijPlatform

rootProject.name = "lean-ctx"

pluginManagement {
    plugins {
        id("org.jetbrains.kotlin.jvm") version "2.3.20"
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
    id("org.jetbrains.intellij.platform.settings") version "2.16.0"
}

@Suppress("UnstableApiUsage")
dependencyResolutionManagement {
    repositories {
        mavenCentral()
        intellijPlatform {
            defaultRepositories()
        }
    }
}
```

- [ ] **Step 3: `build.gradle.kts` ersetzen** (`ctx_edit`, ganze Datei)

old_string = der vollständige aktuelle Inhalt (siehe `ctx_read`), new_string:
```kotlin
import org.jetbrains.intellij.platform.gradle.TestFrameworkType
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm")
    id("org.jetbrains.intellij.platform")
}

group = "com.leanctx"
version = "1.0.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")

    intellijPlatform {
        intellijIdea("2026.1.3")
        testFramework(TestFrameworkType.Platform)
    }
}

intellijPlatform {
    pluginConfiguration {
        name = "lean-ctx"
        version = project.version.toString()
        ideaVersion {
            sinceBuild = "261"
            // untilBuild absichtlich offen (Privat-Plugin, kein Marketplace).
        }
        vendor {
            name = "lean-ctx"
            url = "https://github.com/yvgude/lean-ctx"
        }
    }
}

kotlin {
    jvmToolchain(21)
    compilerOptions {
        jvmTarget = JvmTarget.JVM_21
    }
}
```

- [ ] **Step 4: Gradle-Wrapper vom Template übernehmen** (`ctx_shell`)

```bash
cd /home/tholo/Scripts/lean-ctx/packages/jetbrains-lean-ctx
mkdir -p gradle/wrapper
gh api -H "Accept: application/vnd.github.raw" repos/JetBrains/intellij-platform-plugin-template/contents/gradle/wrapper/gradle-wrapper.properties > gradle/wrapper/gradle-wrapper.properties
gh api -H "Accept: application/vnd.github.raw" repos/JetBrains/intellij-platform-plugin-template/contents/gradle/wrapper/gradle-wrapper.jar > gradle/wrapper/gradle-wrapper.jar
gh api -H "Accept: application/vnd.github.raw" repos/JetBrains/intellij-platform-plugin-template/contents/gradlew > gradlew
gh api -H "Accept: application/vnd.github.raw" repos/JetBrains/intellij-platform-plugin-template/contents/gradlew.bat > gradlew.bat
chmod +x gradlew
```
Verify: `cat gradle/wrapper/gradle-wrapper.properties` enthält `gradle-9.5.0-bin.zip`; `gradle/wrapper/gradle-wrapper.jar` ist nicht leer (`stat -c%s gradle/wrapper/gradle-wrapper.jar` > 50000).

- [ ] **Step 5: Build verifizieren** (lädt IC 2026.1.3 + Gradle 9.5 — groß/langsam)

Run (im Terminal oder nach `lean-ctx allow java`): `bash ./gradlew build 2>&1`
Expected: `BUILD SUCCESSFUL`. Kompiliert den bestehenden Companion-Code (`StatsReader`, `LeanCtxStatusBarFactory`, `BinaryResolver`, `actions/`) unverändert unter Kotlin 2.3.20 → keine Regression.

- [ ] **Step 6: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/gradle.properties packages/jetbrains-lean-ctx/settings.gradle.kts packages/jetbrains-lean-ctx/build.gradle.kts packages/jetbrains-lean-ctx/gradlew packages/jetbrains-lean-ctx/gradlew.bat packages/jetbrains-lean-ctx/gradle/wrapper/gradle-wrapper.jar packages/jetbrains-lean-ctx/gradle/wrapper/gradle-wrapper.properties
git commit -m "build(jetbrains): modernize to IC 2026.1.3 / Kotlin 2.3.20 + Gradle 9.5 wrapper (template DSL)"
```

---

## Task 3: `LeanCtxPaths` — Data-Dir-Resolver + `projectHash`

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/LeanCtxPaths.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/LeanCtxPathsTest.kt`

Rein (keine IntelliJ-Abhängigkeit) → als plain-JVM-JUnit4-Test prüfbar. `resolveDataDir` nimmt `env` + `home` als Parameter (statt Prozess-Env zu mutieren) → testbar.

- [ ] **Step 1: Failing-Test schreiben** (`Write`)

`…/server/LeanCtxPathsTest.kt`:
```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Test
import java.nio.file.Files
import java.nio.file.Paths

class LeanCtxPathsTest {
    @Test
    fun projectHashMatchesRustVector() {
        // sha256("/some/project")[..8]; path absent → raw fallback, identical to Rust project_hash.
        assertEquals("a0317725f24b01df", LeanCtxPaths.projectHash("/some/project"))
    }

    @Test
    fun envOverrideWins() {
        val home = Files.createTempDirectory("lc-home")
        val data = Files.createTempDirectory("lc-data")
        val env = mapOf("LEAN_CTX_DATA_DIR" to data.toString())
        assertEquals(data, LeanCtxPaths.resolveDataDir(env, home))
    }

    @Test
    fun legacyWinsWhenItHasData() {
        val home = Files.createTempDirectory("lc-home2")
        Files.createDirectories(home.resolve(".lean-ctx"))
        Files.writeString(home.resolve(".lean-ctx/stats.json"), "{}")
        assertEquals(home.resolve(".lean-ctx"), LeanCtxPaths.resolveDataDir(emptyMap(), home))
    }

    @Test
    fun xdgWhenLegacyEmpty() {
        val home = Files.createTempDirectory("lc-home3")
        val xdgBase = Files.createTempDirectory("lc-xdg")
        Files.createDirectories(xdgBase.resolve("lean-ctx"))
        Files.writeString(xdgBase.resolve("lean-ctx/config.toml"), "")
        val env = mapOf("XDG_CONFIG_HOME" to xdgBase.toString())
        assertEquals(xdgBase.resolve("lean-ctx"), LeanCtxPaths.resolveDataDir(env, home))
    }

    @Test
    fun portFileName() {
        val data = Paths.get("/tmp/lcdata")
        assertEquals(
            data.resolve("jetbrains-a0317725f24b01df.port"),
            LeanCtxPaths.portFile(data, "/some/project")
        )
    }
}
```

- [ ] **Step 2: Test laufen lassen — Rot erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.LeanCtxPathsTest" 2>&1`
Expected: Compile-Fehler / FAIL — `LeanCtxPaths` existiert noch nicht.

- [ ] **Step 3: `LeanCtxPaths` implementieren** (`Write`)

`…/server/LeanCtxPaths.kt`:
```kotlin
package com.leanctx.plugin.server

import java.nio.file.Path
import java.nio.file.Paths
import java.security.MessageDigest

/**
 * Path resolution mirroring the Rust side (core/data_dir.rs + lsp/port_discovery.rs).
 * Rust and Kotlin MUST resolve byte-identically (spec §5.5).
 */
object LeanCtxPaths {
    private val DATA_MARKERS = listOf("stats.json", "config.toml", "sessions")

    /** Priority: LEAN_CTX_DATA_DIR → ~/.lean-ctx (if has data) → $XDG_CONFIG_HOME/lean-ctx (default ~/.config/lean-ctx). */
    fun resolveDataDir(env: Map<String, String>, home: Path): Path {
        env["LEAN_CTX_DATA_DIR"]?.trim()?.takeIf { it.isNotEmpty() }?.let { return Paths.get(it) }
        val legacy = home.resolve(".lean-ctx")
        if (hasDataFiles(legacy)) return legacy
        val xdgBase = env["XDG_CONFIG_HOME"]?.trim()?.takeIf { it.isNotEmpty() }
            ?.let { Paths.get(it) } ?: home.resolve(".config")
        val xdg = xdgBase.resolve("lean-ctx")
        if (hasDataFiles(xdg)) return xdg
        return if (legacy.toFile().exists()) legacy else xdg
    }

    /** Production resolver using the real process environment + user.home. */
    fun dataDir(): Path = resolveDataDir(System.getenv(), Paths.get(System.getProperty("user.home")))

    private fun hasDataFiles(dir: Path): Boolean = DATA_MARKERS.any { dir.resolve(it).toFile().exists() }

    /** sha256(canonical(root))[..8] as 16 lowercase hex; mirrors Rust project_hash. */
    fun projectHash(projectRoot: String): String {
        val canonical = try {
            Paths.get(projectRoot).toRealPath().toString()
        } catch (_: Exception) {
            projectRoot
        }
        return sha256Prefix16(canonical)
    }

    fun sha256Prefix16(s: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(s.toByteArray(Charsets.UTF_8))
        return buildString(16) { for (i in 0 until 8) append("%02x".format(digest[i])) }
    }

    fun portFile(dataDir: Path, projectRoot: String): Path =
        dataDir.resolve("jetbrains-${projectHash(projectRoot)}.port")
}
```

- [ ] **Step 4: Test laufen lassen — Grün erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.LeanCtxPathsTest" 2>&1`
Expected: 5 Tests PASS.

- [ ] **Step 5: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/LeanCtxPaths.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/LeanCtxPathsTest.kt
git commit -m "feat(jetbrains): LeanCtxPaths data-dir resolver + projectHash (Rust parity)"
```

---

## Task 4: `PortFileWriter` — atomares Schreiben/Löschen

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/PortFileWriter.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/PortFileWriterTest.kt`

Hand-rolled snake_case-JSON (= Rust `PortFile`-serde). Atomar via temp + `ATOMIC_MOVE`, `0600`.

- [ ] **Step 1: Failing-Test schreiben** (`Write`)

`…/server/PortFileWriterTest.kt`:
```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Files
import java.nio.file.attribute.PosixFilePermissions

class PortFileWriterTest {
    @Test
    fun writesSnakeCaseJsonAtomicallyWith0600() {
        val dir = Files.createTempDirectory("lc-pf")
        val target = dir.resolve("jetbrains-abc.port")
        PortFileWriter.write(
            target,
            PortFileData(port = 54321, token = "deadbeef", pid = 4242L,
                projectRoot = "/x/y", ideVersion = "IC-2026.1.3", startedAt = 1700000000000L)
        )
        val json = Files.readString(target)
        assertTrue(json.contains("\"port\":54321"))
        assertTrue(json.contains("\"token\":\"deadbeef\""))
        assertTrue(json.contains("\"pid\":4242"))
        assertTrue(json.contains("\"project_root\":\"/x/y\""))
        assertTrue(json.contains("\"ide_version\":\"IC-2026.1.3\""))
        assertTrue(json.contains("\"started_at\":1700000000000"))
        assertFalse("must not emit camelCase", json.contains("projectRoot"))
        val perms = PosixFilePermissions.toString(Files.getPosixFilePermissions(target))
        assertEquals("rw-------", perms)
    }

    @Test
    fun deleteRemovesFile() {
        val dir = Files.createTempDirectory("lc-pf2")
        val target = dir.resolve("jetbrains-x.port")
        PortFileWriter.write(
            target,
            PortFileData(1, "t", 1L, "/r", "v", 1L)
        )
        assertTrue(Files.exists(target))
        PortFileWriter.delete(target)
        assertFalse(Files.exists(target))
    }
}
```

- [ ] **Step 2: Test laufen lassen — Rot erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.PortFileWriterTest" 2>&1`
Expected: FAIL — `PortFileWriter`/`PortFileData` existieren nicht.

- [ ] **Step 3: `PortFileWriter` implementieren** (`Write`)

`…/server/PortFileWriter.kt`:
```kotlin
package com.leanctx.plugin.server

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermissions

/** Port-file payload. JSON keys are snake_case to match the Rust PortFile serde struct. */
data class PortFileData(
    val port: Int,
    val token: String,
    val pid: Long,
    val projectRoot: String,
    val ideVersion: String,
    val startedAt: Long,
)

object PortFileWriter {
    /** Atomically write target (temp + ATOMIC_MOVE), 0600 perms. */
    fun write(target: Path, data: PortFileData) {
        Files.createDirectories(target.parent)
        val tmp = Files.createTempFile(target.parent, ".jetbrains-", ".port.tmp")
        Files.writeString(tmp, toJson(data))
        setOwnerOnly(tmp)
        Files.move(tmp, target, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE)
        setOwnerOnly(target)
    }

    fun delete(target: Path) {
        try { Files.deleteIfExists(target) } catch (_: Exception) { /* best effort */ }
    }

    private fun toJson(d: PortFileData): String = buildString {
        append('{')
        append("\"port\":").append(d.port).append(',')
        append("\"token\":").append(quote(d.token)).append(',')
        append("\"pid\":").append(d.pid).append(',')
        append("\"project_root\":").append(quote(d.projectRoot)).append(',')
        append("\"ide_version\":").append(quote(d.ideVersion)).append(',')
        append("\"started_at\":").append(d.startedAt)
        append('}')
    }

    private fun quote(s: String): String =
        "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

    private fun setOwnerOnly(p: Path) {
        try {
            Files.setPosixFilePermissions(p, PosixFilePermissions.fromString("rw-------"))
        } catch (_: UnsupportedOperationException) { /* non-POSIX FS */ }
    }
}
```

- [ ] **Step 4: Test laufen lassen — Grün erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.PortFileWriterTest" 2>&1`
Expected: 2 Tests PASS.

- [ ] **Step 5: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/PortFileWriter.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/PortFileWriterTest.kt
git commit -m "feat(jetbrains): PortFileWriter atomic 0600 write/delete (snake_case wire)"
```

---

## Task 5: `RequestRouter` — Token-Check + Routing

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterTest.kt`

Reine Routing-Logik (kein HttpExchange) → unit-testbar. Token-Check VOR Routing: falsch/fehlend → 401. `GET /health` → 200. Sonst 404.

- [ ] **Step 1: Failing-Test schreiben** (`Write`)

`…/server/RequestRouterTest.kt`:
```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class RequestRouterTest {
    private val router = RequestRouter(token = "secret", ideVersion = "IC-2026.1.3", projectName = "demo")

    @Test
    fun healthWithValidTokenReturns200() {
        val r = router.route("GET", "/health", "secret")
        assertEquals(200, r.status)
        assertTrue(r.body.contains("\"status\":\"ok\""))
        assertTrue(r.body.contains("\"ideVersion\":\"IC-2026.1.3\""))
        assertTrue(r.body.contains("\"project\":\"demo\""))
    }

    @Test
    fun missingTokenReturns401() {
        val r = router.route("GET", "/health", null)
        assertEquals(401, r.status)
        assertTrue(r.body.contains("UNAUTHORIZED"))
    }

    @Test
    fun wrongTokenReturns401() {
        assertEquals(401, router.route("GET", "/health", "nope").status)
    }

    @Test
    fun unknownPathWithValidTokenReturns404() {
        val r = router.route("GET", "/nope", "secret")
        assertEquals(404, r.status)
    }
}
```

- [ ] **Step 2: Test laufen lassen — Rot erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.RequestRouterTest" 2>&1`
Expected: FAIL — `RequestRouter`/`HttpResult` existieren nicht.

- [ ] **Step 3: `RequestRouter` implementieren** (`Write`)

`…/server/RequestRouter.kt`:
```kotlin
package com.leanctx.plugin.server

data class HttpResult(val status: Int, val body: String)

/** Token-guarded request routing. Phase 2 serves only GET /health. */
class RequestRouter(
    private val token: String,
    private val ideVersion: String,
    private val projectName: String,
) {
    fun route(method: String, path: String, headerToken: String?): HttpResult {
        if (headerToken != token) {
            return HttpResult(401, errorJson("UNAUTHORIZED", "missing or invalid token"))
        }
        return when {
            method == "GET" && path == "/health" -> HttpResult(
                200,
                "{\"status\":\"ok\",\"ideVersion\":${q(ideVersion)},\"project\":${q(projectName)}}"
            )
            else -> HttpResult(404, errorJson("NOT_FOUND", "no route for $method $path"))
        }
    }

    private fun errorJson(code: String, msg: String) =
        "{\"error\":{\"code\":${q(code)},\"message\":${q(msg)}}}"

    private fun q(s: String) = "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\""
}
```

- [ ] **Step 4: Test laufen lassen — Grün erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.RequestRouterTest" 2>&1`
Expected: 4 Tests PASS.

- [ ] **Step 5: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/RequestRouter.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/RequestRouterTest.kt
git commit -m "feat(jetbrains): RequestRouter token-check + /health routing"
```

---

## Task 6: `BackendHttpServer` — Lifecycle (`Disposable`) + Integrationstest

**Files:**
- Create: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt`
- Test: `packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/BackendHttpServerTest.kt`

IDE-spezifische Werte (ideVersion/projectName/projectRoot/dataDir/startedAt) werden **injiziert** → der Server ist in plain-JVM testbar (echter Socket, `java.net.http.HttpClient`). Einzige IntelliJ-Referenz: das `Disposable`-Interface (auf dem Test-Classpath via `testFramework`).

- [ ] **Step 1: Failing-Integrationstest schreiben** (`Write`)

`…/server/BackendHttpServerTest.kt`:
```kotlin
package com.leanctx.plugin.server

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.file.Files

class BackendHttpServerTest {
    private fun get(port: Int, token: String?): HttpResponse<String> {
        val b = HttpRequest.newBuilder(URI.create("http://127.0.0.1:$port/health")).GET()
        if (token != null) b.header("X-LeanCtx-Token", token)
        return HttpClient.newHttpClient().send(b.build(), HttpResponse.BodyHandlers.ofString())
    }

    @Test
    fun startWritesPortFileAndServesHealth() {
        val dataDir = Files.createTempDirectory("lc-srv")
        val server = BackendHttpServer(
            dataDir = dataDir, projectRoot = "/some/project",
            ideVersion = "IC-2026.1.3", projectName = "demo", startedAt = 1L
        )
        try {
            server.start()
            val portFile = LeanCtxPaths.portFile(dataDir, "/some/project")
            assertTrue(Files.exists(portFile))
            val json = Files.readString(portFile)
            assertTrue(json.contains("\"port\":${server.port}"))
            assertTrue(json.contains("\"project_root\":\"/some/project\""))

            assertEquals(200, get(server.port, server.tokenForTest).statusCode())
            assertEquals(401, get(server.port, null).statusCode())
            assertEquals(401, get(server.port, "wrong").statusCode())
        } finally {
            server.dispose()
        }
        assertFalse(Files.exists(LeanCtxPaths.portFile(dataDir, "/some/project")))
    }
}
```

- [ ] **Step 2: Test laufen lassen — Rot erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.BackendHttpServerTest" 2>&1`
Expected: FAIL — `BackendHttpServer` existiert nicht.

- [ ] **Step 3: `BackendHttpServer` implementieren** (`Write`)

`…/server/BackendHttpServer.kt`:
```kotlin
package com.leanctx.plugin.server

import com.intellij.openapi.Disposable
import com.sun.net.httpserver.HttpServer
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Path
import java.security.SecureRandom
import java.util.concurrent.Executors

/**
 * Per-project localhost HTTP server. lean-ctx (Rust) is the client; this is the server.
 * Disposable → registered against the Project, so projectClosing stops it + deletes the port file.
 */
class BackendHttpServer(
    private val dataDir: Path,
    private val projectRoot: String,
    private val ideVersion: String,
    private val projectName: String,
    private val startedAt: Long,
) : Disposable {
    private val token: String = newToken()
    private var server: HttpServer? = null
    private var portFile: Path? = null

    val port: Int get() = server?.address?.port ?: -1
    val tokenForTest: String get() = token

    fun start() {
        val http = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        val router = RequestRouter(token, ideVersion, projectName)
        http.executor = Executors.newCachedThreadPool()
        http.createContext("/") { exchange ->
            try {
                val headerToken = exchange.requestHeaders.getFirst("X-LeanCtx-Token")
                val result = router.route(exchange.requestMethod, exchange.requestURI.path, headerToken)
                val bytes = result.body.toByteArray(StandardCharsets.UTF_8)
                exchange.responseHeaders.add("Content-Type", "application/json")
                exchange.sendResponseHeaders(result.status, bytes.size.toLong())
                exchange.responseBody.use { it.write(bytes) }
            } finally {
                exchange.close()
            }
        }
        http.start()
        server = http

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
    }

    override fun dispose() {
        server?.stop(0)
        server = null
        portFile?.let { PortFileWriter.delete(it) }
        portFile = null
    }

    private fun newToken(): String {
        val bytes = ByteArray(32)
        SecureRandom().nextBytes(bytes)
        return buildString(64) { bytes.forEach { append("%02x".format(it)) } }
    }
}
```

- [ ] **Step 4: Test laufen lassen — Grün erwartet**

Run: `bash ./gradlew test --tests "com.leanctx.plugin.server.BackendHttpServerTest" 2>&1`
Expected: 1 Test PASS (Port-Datei geschrieben, /health 200, ohne/falscher Token 401, nach dispose Datei weg).

- [ ] **Step 5: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/server/BackendHttpServer.kt packages/jetbrains-lean-ctx/src/test/kotlin/com/leanctx/plugin/server/BackendHttpServerTest.kt
git commit -m "feat(jetbrains): BackendHttpServer per-project lifecycle + /health (JVM integration test)"
```

---

## Task 7: `LeanCtxStartupActivity` — Server pro Project booten

**Files:**
- Modify: `packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt`

Nach dem bestehenden Binary-Null-Check zusätzlich den Server booten und an das `Project` disposen. Verifikation = manuelles `runIde`-Gate (Task 8), da `ApplicationInfo`/`Project` die Plattform brauchen.

- [ ] **Step 1: `execute` erweitern** (`ctx_edit`)

old_string:
```kotlin
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity

class LeanCtxStartupActivity : ProjectActivity {
    override suspend fun execute(project: Project) {
        val binary = BinaryResolver.resolve()
        if (binary == null) {
            NotificationGroupManager.getInstance()
                .getNotificationGroup("lean-ctx")
                .createNotification(
                    "lean-ctx binary not found",
                    "Install with: cargo install lean-ctx\nOr: npm install -g lean-ctx-bin",
                    NotificationType.WARNING
                )
                .notify(project)
        }
    }
}
```
new_string:
```kotlin
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.application.ApplicationInfo
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity
import com.intellij.openapi.util.Disposer
import com.leanctx.plugin.server.BackendHttpServer
import com.leanctx.plugin.server.LeanCtxPaths

class LeanCtxStartupActivity : ProjectActivity {
    private val log = Logger.getInstance(LeanCtxStartupActivity::class.java)

    override suspend fun execute(project: Project) {
        val binary = BinaryResolver.resolve()
        if (binary == null) {
            NotificationGroupManager.getInstance()
                .getNotificationGroup("lean-ctx")
                .createNotification(
                    "lean-ctx binary not found",
                    "Install with: cargo install lean-ctx\nOr: npm install -g lean-ctx-bin",
                    NotificationType.WARNING
                )
                .notify(project)
        }
        startBackend(project)
    }

    /** Boot the per-project HTTP backend; failures must never break the IDE/companion. */
    private fun startBackend(project: Project) {
        val root = project.basePath ?: return
        try {
            val server = BackendHttpServer(
                dataDir = LeanCtxPaths.dataDir(),
                projectRoot = root,
                ideVersion = ApplicationInfo.getInstance().fullVersion,
                projectName = project.name,
                startedAt = System.currentTimeMillis(),
            )
            server.start()
            Disposer.register(project, server)
            log.info("lean-ctx backend listening on 127.0.0.1:${server.port} for $root")
        } catch (e: Exception) {
            log.warn("lean-ctx backend failed to start", e)
        }
    }
}
```

- [ ] **Step 2: Kompilieren** (kein neuer Unit-Test — Plattform-Glue)

Run: `bash ./gradlew compileKotlin 2>&1`
Expected: `BUILD SUCCESSFUL`.

- [ ] **Step 3: Volle Test-Suite** (Regression der Tasks 3–6)

Run: `bash ./gradlew test 2>&1`
Expected: alle Tests (LeanCtxPaths 5, PortFileWriter 2, RequestRouter 4, BackendHttpServer 1) PASS.

- [ ] **Step 4: Reformat + Commit**

```bash
git add packages/jetbrains-lean-ctx/src/main/kotlin/com/leanctx/plugin/LeanCtxStartupActivity.kt
git commit -m "feat(jetbrains): boot BackendHttpServer per project, disposed with Project"
```

---

## Task 8: Manuelles `runIde`-Gate (End-to-End-Verifikation)

**Files:** keine — Verifikations-Checkliste (Spec §15.6 / §10).

Dieser Schritt läuft im echten Terminal des Users (Sandbox-IDE). Lädt beim ersten Lauf die IC-2026.1.3-Distribution.

- [ ] **Step 1: Sandbox-IDE starten**

Run: `cd packages/jetbrains-lean-ctx && bash ./gradlew runIde 2>&1`
Expected: Eine IntelliJ-Sandbox öffnet sich. Ein Projekt öffnen (oder ein vorhandenes Java/Kotlin-Projekt laden).

- [ ] **Step 2: Port-Datei prüfen**

Im Haupt-Terminal (Data-Dir = Ausgabe von `lean_ctx_data_dir()`, i. d. R. `~/.lean-ctx`):
```bash
ls -l ~/.lean-ctx/jetbrains-*.port
stat -c '%a' ~/.lean-ctx/jetbrains-*.port
cat ~/.lean-ctx/jetbrains-*.port
```
Expected: Datei existiert, Permissions `600`, JSON enthält `port`, `token`, `pid`, `project_root`, `ide_version`, `started_at` (snake_case).

- [ ] **Step 3: `/health` mit/ohne Token**

```bash
PORT=$(cat ~/.lean-ctx/jetbrains-*.port | jq -r .port)
TOKEN=$(cat ~/.lean-ctx/jetbrains-*.port | jq -r .token)
curl -s -o /dev/null -w '%{http_code}\n' -H "X-LeanCtx-Token: $TOKEN" http://127.0.0.1:$PORT/health
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:$PORT/health
```
Expected: erstes `200`, zweites `401`.

- [ ] **Step 4: Rust-Discovery end-to-end**

```bash
curl -s -H "X-LeanCtx-Token: $TOKEN" http://127.0.0.1:$PORT/health
```
Expected: `{"status":"ok","ideVersion":"…","project":"…"}`. (Optional: in der Sandbox prüfen, dass die Companion-Statusbar weiterhin `⚡ … saved`/`⚡ lean-ctx` zeigt → keine Regression.)

- [ ] **Step 5: Cleanup bei Projektschluss**

In der Sandbox das Projekt schließen (File → Close Project) oder die Sandbox-IDE beenden.
```bash
ls -l ~/.lean-ctx/jetbrains-*.port
```
Expected: Keine passende `.port`-Datei mehr (von `dispose()` gelöscht).

- [ ] **Step 6 (optional, §12.3): Phase-2-Commits zu einem Phase-Commit zusammenfassen**

Falls die Team-Konvention „ein Commit pro Phase" gilt (Spec §12.3), die Task-Commits 2–7 (Plugin-Seite; Task 1 = eigener Rust-Fix kann separat bleiben oder mit rein) soft zusammenfassen:
```bash
# Beispiel — Basis = Commit VOR Task 2:
git reset --soft <commit-vor-task-2>
git commit -m "feat(jetbrains): Phase 2 — plugin core HTTP lifecycle + build modernization

- Build: IC 2026.1.3 / Kotlin 2.3.20 / Gradle 9.5 (template DSL)
- server/: LeanCtxPaths (Rust-parity resolver+hash), PortFileWriter (atomic 0600, snake_case wire),
  RequestRouter (token+/health), BackendHttpServer (per-project Disposable lifecycle)
- LeanCtxStartupActivity boots backend per project
- Gate: gradle test green; manual runIde gate passed"
```

---

## Self-Review

**1. Spec-Coverage (§15 / §15.7 / §5.5):**
- §15.7 Build-Modernisierung → Task 2 (Kotlin 2.3.20, platform 2.16.0, IC 2026.1.3, jvmTarget 21/`compilerOptions`, `kotlin.stdlib.default.dependency=false`, config-cache, Wrapper, Template-DSL). ✓
- §15.2 `LeanCtxPaths` → Task 3 ✓ · `PortFileWriter` → Task 4 ✓ · `RequestRouter`+Health → Task 5 ✓ · `BackendHttpServer` → Task 6 ✓ · `LeanCtxStartupActivity` erweitern → Task 7 ✓
- §15.5 Phase-1-Begleit-Fix (`port_file_path`) → Task 1 ✓
- §5.5 Data-Dir-Parität + `projectHash`-Parität → Task 1 (Rust-Anker `a0317725f24b01df`) + Task 3 (Kotlin gleicher Vektor) ✓ · snake_case-Wire → Task 4 ✓
- §15.6 Gate (runIde, Port-Datei 0600, /health 200/401, projectClosing löscht, Kotlin-Unit, cargo nextest) → Task 8 + Tasks 1/3/6 ✓
- §15.4 kein neuer `postStartupActivity` (Disposer.register statt projectService) → Task 7 ✓
- **Bewusste Abweichung von §15.2:** `dto/`+gson entfallen in Phase 2 (hand-rolled JSON), gson erst Phase 3 — im File-Structure-Block dokumentiert.

**2. Placeholder-Scan:** Keine TBD/TODO; jeder Code-Step zeigt vollständigen Code; Test-Vektor `a0317725f24b01df` ist konkret (sha256("/some/project")[..8]).

**3. Typ-Konsistenz:** `PortFileData(port,token,pid,projectRoot,ideVersion,startedAt)` identisch in Task 4 (Def), Task 6 (Nutzung). `LeanCtxPaths.{resolveDataDir,dataDir,projectHash,sha256Prefix16,portFile}` konsistent Task 3↔6. `RequestRouter(token,ideVersion,projectName)` + `HttpResult(status,body)` konsistent Task 5↔6. `BackendHttpServer(dataDir,projectRoot,ideVersion,projectName,startedAt)` konsistent Task 6↔7. JSON-Keys snake_case konsistent Rust (Task 1) ↔ Kotlin (Task 4) ↔ Test (Task 6).
