#!/usr/bin/env python3
"""Check the public Rust tree against the documented open-core boundary.

GitLab CI integration TODO:

    boundary-check:
      script: python3 scripts/check-open-core-boundary.py
      rules:
        - changes: ["rust/**", "docs/contracts/**"]

The classification document is intentionally optional during the staged
rollout.  Import and strategic-data checks still run when it is absent.
"""

from __future__ import annotations

import fnmatch
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional, Sequence, Tuple


ROOT = Path(__file__).resolve().parents[1]
CLASSIFICATION_PATH = ROOT / "docs/internal/architecture/MODULE_CLASSIFICATION.md"
RUST_SOURCE_ROOT = ROOT / "rust/src"
RUST_CRATES_ROOT = ROOT / "rust/crates"

CLASS_TOKEN = re.compile(
    r"\b(?:class|classification)\s*[:=\-]?\s*([A-E])\b", re.IGNORECASE
)
INLINE_PATH = re.compile(r"(?:^|\s)(rust/(?:src|crates)/[^\s|)`]+)")
PRIVATE_IMPORT = re.compile(
    r"(?:^|::)(?:"
    r"lean[_-]?ctx[_-]?(?:enterprise|private)|"
    r"leanctx[_-]?(?:enterprise|private)|"
    r"private|proprietary|commercial|control_plane|strategic_data|"
    r"enterprise::(?:control_plane|scheduler|economics|knowledge_hub|fleet)"
    r")(?:$|::)",
    re.IGNORECASE,
)

BENCHMARK_CORPUS_PATH = re.compile(
    r"(?:^|/)(?:benchmark[_-]?(?:corpus|dataset)|"
    r"private[_-]?benchmark|customer[_-]?benchmark)(?:[._/-]|$)",
    re.IGNORECASE,
)
PROVIDER_RATE_PATH = re.compile(
    r"(?:^|/)(?:provider[_-]?(?:rate|rates|pricing|prices|costs)|"
    r"model[_-]?rates|rate[_-]?card|pricing[_-]?table)(?:[._/-]|$)",
    re.IGNORECASE,
)
BENCHMARK_CORPUS_DATA = re.compile(
    r"\b(?:private|customer|proprietary)\s+benchmark\s+(?:corpus|dataset)\b|"
    r"\bbenchmark[_-]?(?:corpus|dataset)\b",
    re.IGNORECASE,
)
PROVIDER_RATE_DATA = re.compile(
    r"(?:\"(?:provider|model)_(?:rate|rates|pricing|price|prices)\"\s*:|"
    r"\b(?:provider|model)_(?:rate|rates|pricing|price|prices)_micros\b|"
    r"\b(?:provider|model)_(?:rate|rates|pricing|price|prices)\s*=|"
    r"\b(?:input|output|cached|reasoning)_rate_micros\b)",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class ClassificationRule:
    pattern: str
    class_name: str
    line_number: int


def relative_path(path: Path) -> str:
    """Return a stable POSIX path relative to the repository root."""

    return path.relative_to(ROOT).as_posix()


def rust_modules() -> List[Path]:
    """Enumerate every Rust module in rust/src and rust/crates."""

    modules: List[Path] = []
    for root in (RUST_SOURCE_ROOT, RUST_CRATES_ROOT):
        if root.is_dir():
            modules.extend(path for path in root.rglob("*.rs") if path.is_file())
    return sorted(modules, key=relative_path)


def public_path(path: Path) -> bool:
    """Return whether a path is in the public Rust surface checked here."""

    name = relative_path(path)
    return name.startswith("rust/src/") or name.startswith("rust/crates/lean-ctx-")


def normalize_pattern(candidate: str) -> Optional[str]:
    """Normalize a classification path/glob into a repository-relative form."""

    value = candidate.strip().strip("`\"'")
    value = re.sub(r"^\./", "", value).replace("\\", "/")
    value = value.rstrip(",;:)")
    if not value:
        return None
    if value.startswith(str(ROOT).replace("\\", "/") + "/"):
        value = value[len(str(ROOT).replace("\\", "/")) + 1 :]
    if value.startswith("src/") or value.startswith("crates/"):
        value = "rust/" + value
    if not value.startswith("rust/"):
        return None
    return value


def path_candidates(line: str) -> List[str]:
    """Extract path/glob cells from a Markdown classification line."""

    candidates: List[str] = []
    candidates.extend(re.findall(r"`([^`]+)`", line))
    candidates.extend(INLINE_PATH.findall(line))
    if "|" in line:
        candidates.extend(cell.strip() for cell in line.split("|") if cell.strip())
    normalized: List[str] = []
    for candidate in candidates:
        match = re.search(r"(?:^|\s)(rust/(?:src|crates)/[^\s|)`]+)", candidate)
        value = match.group(1) if match else candidate
        normalized_value = normalize_pattern(value)
        if normalized_value and normalized_value not in normalized:
            normalized.append(normalized_value)
    return normalized


def class_from_line(line: str) -> Optional[str]:
    """Extract an explicit A-E class from a heading, list item, or table row."""

    match = CLASS_TOKEN.search(line)
    if match:
        return match.group(1).upper()
    heading = re.match(r"^\s*#{1,6}\s*(?:class\s*)?([A-E])(?:\s|[-—:]|$)", line, re.I)
    if heading:
        return heading.group(1).upper()
    bold = re.search(r"\*\*([A-E])\*\*", line, re.I)
    if bold:
        return bold.group(1).upper()
    if "|" in line:
        for cell in line.split("|"):
            if re.fullmatch(r"\s*([A-E])\s*", cell, re.IGNORECASE):
                return cell.strip().upper()
    return None


def parse_classification_document(
    path: Path,
) -> Tuple[List[ClassificationRule], List[str]]:
    """Parse both explicit rows and class-scoped path lists."""

    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        return [], ["cannot read classification document: %s" % error]

    rules: List[ClassificationRule] = []
    errors: List[str] = []
    current_class: Optional[str] = None
    for line_number, line in enumerate(lines, start=1):
        heading = re.match(r"^\s*#{1,6}\s+.*", line)
        explicit_class = class_from_line(line)
        if heading and explicit_class:
            current_class = explicit_class
        candidates = path_candidates(line)
        if not candidates:
            continue
        selected_class = explicit_class or current_class
        if selected_class is None:
            errors.append(
                "classification path without class at line %d: %s"
                % (line_number, line.strip())
            )
            continue
        for pattern in candidates:
            rules.append(ClassificationRule(pattern, selected_class, line_number))
    return rules, errors


def pattern_matches(pattern: str, path_name: str) -> bool:
    """Match exact paths, directory rules, and ordinary Markdown globs."""

    alternatives = (path_name, path_name.removeprefix("rust/"))
    for candidate in alternatives:
        if fnmatch.fnmatchcase(candidate, pattern):
            return True
        if candidate == pattern or candidate.startswith(pattern.rstrip("/") + "/"):
            return True
    return False


def best_classification(path: Path, rules: Sequence[ClassificationRule]) -> Optional[str]:
    """Choose the most-specific matching rule, preserving deterministic output."""

    name = relative_path(path)
    matches = [rule for rule in rules if pattern_matches(rule.pattern, name)]
    if not matches:
        return None
    matches.sort(key=lambda rule: (len(rule.pattern.replace("*", "")), rule.line_number), reverse=True)
    return matches[0].class_name


def check_classifications(modules: Sequence[Path]) -> List[str]:
    """Check module coverage and reject D/E assignments in public Rust paths."""

    if not CLASSIFICATION_PATH.exists():
        print(
            "INFO: %s is absent; skipping module classification checks"
            % relative_path(CLASSIFICATION_PATH)
        )
        return []

    rules, parse_errors = parse_classification_document(CLASSIFICATION_PATH)
    violations = ["[classification] %s" % error for error in parse_errors]
    if not rules:
        violations.append(
            "[classification] no usable path/class rules found in %s"
            % relative_path(CLASSIFICATION_PATH)
        )
    for module in modules:
        class_name = best_classification(module, rules)
        name = relative_path(module)
        if class_name is None:
            violations.append("[classification] %s has no A-E classification" % name)
        elif class_name in ("D", "E") and public_path(module):
            violations.append(
                "[public-private-boundary] %s is Class %s in a public path"
                % (name, class_name)
            )
    return violations


def check_private_imports(modules: Sequence[Path]) -> List[str]:
    """Reject imports from recognizable private namespaces in public modules."""

    violations: List[str] = []
    for module in modules:
        if not public_path(module):
            continue
        try:
            lines = module.read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeError) as error:
            violations.append("[read] %s: %s" % (relative_path(module), error))
            continue
        for line_number, line in enumerate(lines, start=1):
            match = re.match(r"^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+([^;]+);", line)
            if match and PRIVATE_IMPORT.search(match.group(1)):
                violations.append(
                    "[private-import] %s:%d imports private namespace: %s"
                    % (relative_path(module), line_number, match.group(1).strip())
                )
    return violations


def public_files() -> Iterable[Path]:
    """Yield files under the public source and public OCLA crate roots."""

    roots = [RUST_SOURCE_ROOT]
    if RUST_CRATES_ROOT.is_dir():
        roots.extend(sorted(RUST_CRATES_ROOT.glob("lean-ctx-*")))
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*"), key=relative_path):
            if path.is_file():
                yield path


def check_strategic_data() -> List[str]:
    """Reject obvious private benchmark corpora and provider-rate data."""

    violations: List[str] = []
    for path in public_files():
        name = relative_path(path)
        reason: Optional[str] = None
        if BENCHMARK_CORPUS_PATH.search(name):
            reason = "benchmark corpus/dataset path"
        elif PROVIDER_RATE_PATH.search(name):
            reason = "provider rate/pricing path"
        else:
            try:
                if path.stat().st_size > 4 * 1024 * 1024:
                    continue
                lines = path.read_text(encoding="utf-8").splitlines()
            except (OSError, UnicodeError):
                continue
            for line in lines:
                if BENCHMARK_CORPUS_DATA.search(line):
                    reason = "benchmark corpus/dataset content"
                    break
                if PROVIDER_RATE_DATA.search(line):
                    reason = "provider rate/pricing data"
                    break
        if reason:
            violations.append("[strategic-data] %s contains %s" % (name, reason))
    return violations


def main() -> int:
    modules = rust_modules()
    violations = []
    violations.extend(check_classifications(modules))
    violations.extend(check_private_imports(modules))
    violations.extend(check_strategic_data())
    if violations:
        print("Open-core boundary: FAIL")
        for violation in sorted(set(violations)):
            print("- %s" % violation)
        return 1
    print("Open-core boundary: PASS (%d Rust modules checked)" % len(modules))
    return 0


if __name__ == "__main__":
    sys.exit(main())
