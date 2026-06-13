#!/usr/bin/env bash
# Materializes the runRustRover cross-IDE-gate Cargo fixture into
# tmp/runrustrover-cross-ide-gate/. Idempotent: re-running fully resets it.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/tmp/runrustrover-cross-ide-gate"
rm -rf "$DEST"
mkdir -p "$DEST/src"
cat > "$DEST/Cargo.toml" <<'EOF'
[package]
name = "runrustrover-cross-ide-gate"
version = "0.1.0"
edition = "2021"
[[bin]]
name = "gate"
path = "src/main.rs"
EOF
# trait Shape + two impls (Circle, Square) → Trait→Impl "hierarchy" via implementations.
cat > "$DEST/src/shapes.rs" <<'EOF'
pub trait Shape {
  fn area(&self) -> f64;
}
pub struct Circle {
  pub r: f64,
}
impl Shape for Circle {
  fn area(&self) -> f64 {
    std::f64::consts::PI * self.r * self.r
}}
pub struct Square {
  pub s: f64,
}
impl Shape for Square {
  fn area(&self) -> f64 {
    self.s * self.s
}}
EOF
# A free function with >=2 call sites (for references + ctx_callgraph callers/callees).
cat > "$DEST/src/main.rs" <<'EOF'
mod shapes;
use shapes::{Circle, Shape, Square};
fn total_area(shapes: &[&dyn Shape]) -> f64 {
  shapes.iter().map(|s| s.area()).sum()
}
fn main() {
  let c = Circle { r: 1.0 };
  let sq = Square { s: 2.0 };
  let first = total_area(&[&c]);
  let second = total_area(&[&c, &sq]);
  println!("{} {}", first, second);
}
EOF
# Deliberately misformatted Rust file (reformat gate, check #6).
cat > "$DEST/src/messy.rs" <<'EOF'
pub struct   Messy{pub x:i32}
impl Messy{
pub fn   render(&self)->i32{
let    y=self.x;
    y+y
}}
EOF
echo "fixture ready: $DEST"
