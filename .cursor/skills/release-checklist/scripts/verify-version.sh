#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 X.Y.Z [oss-root] [homebrew-root] [aur-source-root] [aur-bin-root]" >&2
  exit 2
}

version_arg=${1:-}
[[ "$version_arg" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || usage

oss_root=${2:-$(git rev-parse --show-toplevel)}
projects_root=$(dirname "$oss_root")
homebrew_root=${3:-$projects_root/homebrew-lean-ctx}
aur_source_root=${4:-$projects_root/aur-lean-ctx}
aur_bin_root=${5:-$projects_root/aur-lean-ctx-bin}
failed=0

pass() { printf 'ok: %s\n' "$1"; }
fail() { printf 'FAIL: %s\n' "$1" >&2; failed=1; }

require_fixed() {
  local needle=$1 file=$2 label=$3
  if [[ -f "$file" ]] && grep -Fq -- "$needle" "$file"; then
    pass "$label"
  else
    fail "$label ($file)"
  fi
}

require_fixed "version = \"$version_arg\"" "$oss_root/rust/Cargo.toml" "Cargo package version"
require_fixed "## [$version_arg]" "$oss_root/CHANGELOG.md" "CHANGELOG section"

lock_version=$(awk '
  $0 == "name = \"lean-ctx\"" {
    getline
    gsub(/^version = \"|\"$/, "")
    print
    exit
  }
' "$oss_root/rust/Cargo.lock")
[[ "$lock_version" == "$version_arg" ]] \
  && pass "Cargo.lock package version" \
  || fail "Cargo.lock package version is $lock_version"

for package_file in \
  "$oss_root/packages/pi-lean-ctx/package.json" \
  "$oss_root/packages/lean-ctx-bin/package.json"; do
  package_version=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$package_file")
  [[ "$package_version" == "$version_arg" ]] \
    && pass "$(basename "$(dirname "$package_file")") version" \
    || fail "$(basename "$(dirname "$package_file")") is $package_version"
done

if [[ ${VERIFY_DISTRIBUTION:-0} == 1 ]]; then
  if [[ -f "$homebrew_root/Formula/lean-ctx.rb" ]]; then
    require_fixed "version \"$version_arg\"" "$homebrew_root/Formula/lean-ctx.rb" "Homebrew formula version"
  else
    fail "Homebrew repo not found at $homebrew_root"
  fi

  for aur_root in "$aur_source_root" "$aur_bin_root"; do
    if [[ -f "$aur_root/PKGBUILD" ]]; then
      require_fixed "pkgver=$version_arg" "$aur_root/PKGBUILD" "$(basename "$aur_root") pkgver"
      require_fixed "pkgver = $version_arg" "$aur_root/.SRCINFO" "$(basename "$aur_root") .SRCINFO"
    else
      fail "AUR repo not found at $aur_root"
    fi
  done
else
  echo "skip: distribution repos (set VERIFY_DISTRIBUTION=1 after publication)"
fi

if git -C "$oss_root" rev-parse -q --verify "refs/tags/v$version_arg" >/dev/null; then
  [[ ${ALLOW_EXISTING_TAG:-0} == 1 ]] \
    && pass "existing tag v$version_arg allowed" \
    || fail "local tag v$version_arg already exists"
else
  pass "tag v$version_arg is available locally"
fi

exit "$failed"
