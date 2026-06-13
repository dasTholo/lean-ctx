#!/usr/bin/env bash
# Materializes the runIde inline/reformat-gate Kotlin fixture into
# tmp/runide-inline-reformat-gate/. Idempotent: re-running fully resets it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/tmp/runide-inline-reformat-gate"

rm -rf "$DEST"
mkdir -p "$DEST/src/main/kotlin/app"

cat > "$DEST/settings.gradle.kts" <<'EOF'
rootProject.name = "runide-inline-reformat-gate"
EOF

cat > "$DEST/build.gradle.kts" <<'EOF'
plugins {
    kotlin("jvm") version "2.1.0"
}

repositories {
    mavenCentral()
}
EOF

# inline: bare local variable (Calc.kt: val tmp = a + b; return tmp + tmp).
cat > "$DEST/src/main/kotlin/app/Calc.kt" <<'EOF'
package app

class Calc {
    fun calc(a: Int, b: Int): Int {
        val tmp = a + b
        return tmp + tmp
    }
}
EOF

# inline: method with >=2 call sites (body substitution + param binding).
cat > "$DEST/src/main/kotlin/app/Helper.kt" <<'EOF'
package app

class Helper {
    fun calc(x: Int): Int = x * 2
}

fun callsites(h: Helper): Int = h.calc(3) + h.calc(4)
EOF

# inline: recursive method → UNSUPPORTED case (#5).
cat > "$DEST/src/main/kotlin/app/Recurse.kt" <<'EOF'
package app

class Recurse {
    fun loop(n: Int): Int = if (n <= 0) 0 else loop(n - 1)
}
EOF

# reformat: badly formatted file + region + symbol (Messy.kt).
cat > "$DEST/src/main/kotlin/app/Messy.kt" <<'EOF'
package app

class Messy{
fun render( ):String{
val   x="a"
        return x
}
fun other():Int{return    1}
}
EOF

# reformat: unused imports (Imports.kt, for optimize_imports).
cat > "$DEST/src/main/kotlin/app/Imports.kt" <<'EOF'
package app

import java.util.ArrayList
import java.util.HashMap

class Imports {
    fun n(): Int = 1
}
EOF

# Plain text file — UNSUPPORTED_LANGUAGE gate case (from v2c).
cat > "$DEST/notes.txt" <<'EOF'
Plain text file — used for the UNSUPPORTED_LANGUAGE gate case.
EOF

echo "fixture ready: $DEST"
