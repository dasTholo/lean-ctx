#!/usr/bin/env bash
# Materializes the runIde rename-gate Kotlin fixture into tmp/runide-rename-gate/.
# Idempotent: re-running fully resets the fixture to a clean state.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/tmp/runide-rename-gate"

rm -rf "$DEST"
mkdir -p "$DEST/src/main/kotlin"

cat > "$DEST/settings.gradle.kts" <<'EOF'
rootProject.name = "runide-rename-gate"
EOF

# Kotlin jvm version: see plan Step 3 verification. 2.1.0 is a safe stable
# default; adjust only if Gradle sync against the sandbox IDE fails.
cat > "$DEST/build.gradle.kts" <<'EOF'
plugins {
    kotlin("jvm") version "2.1.0"
}

repositories {
    mavenCentral()
}
EOF

cat > "$DEST/src/main/kotlin/Widget.kt" <<'EOF'
package p

// Rename target for the gate (rename_preview/rename_apply name_path=Widget).
class Widget
EOF

cat > "$DEST/src/main/kotlin/Usage.kt" <<'EOF'
package p

// Cross-file reference to Widget — proves findUsages spans files.
fun use(): Widget = Widget()
EOF

cat > "$DEST/src/main/kotlin/Gadget.kt" <<'EOF'
package p

// Pre-existing name → collision target for the conflict (±force) gate case.
class Gadget
EOF

cat > "$DEST/notes.txt" <<'EOF'
Plain text file — used for the UNSUPPORTED_LANGUAGE gate case.
EOF

echo "fixture ready: $DEST"
