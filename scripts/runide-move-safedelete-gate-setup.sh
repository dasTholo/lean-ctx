#!/usr/bin/env bash
# Materializes the runIde move/safe_delete-gate Kotlin fixture into
# tmp/runide-move-safedelete-gate/. Idempotent: re-running fully resets it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/tmp/runide-move-safedelete-gate"

rm -rf "$DEST"
mkdir -p "$DEST/src/main/kotlin/app"
mkdir -p "$DEST/src/main/kotlin/app/moved"

cat > "$DEST/settings.gradle.kts" <<'EOF'
rootProject.name = "runide-move-safedelete-gate"
EOF

cat > "$DEST/build.gradle.kts" <<'EOF'
plugins {
    kotlin("jvm") version "2.1.0"
}

repositories {
    mavenCentral()
}
EOF

# Movable top-level class (move_preview/move_apply name_path=Widget, target_path=app/moved).
cat > "$DEST/src/main/kotlin/app/Widget.kt" <<'EOF'
package app

// Move target for the gate. Referenced cross-file by Usage.kt (proves refs follow).
class Widget
EOF

# Cross-file reference to Widget — proves move rewrites imports/refs.
cat > "$DEST/src/main/kotlin/app/Usage.kt" <<'EOF'
package app

// Reference to Widget — must be rewritten after move; blocks safe_delete of Widget.
fun use(): Widget = Widget()
EOF

# Unused symbol — safe_delete happy path (no blocking refs).
cat > "$DEST/src/main/kotlin/app/Unused.kt" <<'EOF'
package app

// Unreferenced — safe_delete_preview should report zero blocking usages.
class Unused
EOF

# Helper with a member, for the (language-dependent) target_parent member-move case.
cat > "$DEST/src/main/kotlin/app/Helper.kt" <<'EOF'
package app

class Helper {
    fun calc(): Int = 42
}

class OtherClass
EOF

cat > "$DEST/notes.txt" <<'EOF'
Plain text file — used for the UNSUPPORTED_LANGUAGE gate case.
EOF

echo "fixture ready: $DEST"
