#!/usr/bin/env python3
"""Extract (file, lint_name) fingerprints from cargo clippy output logs.

Usage:
    extract_clippy_fingerprints.py LOG_FILE [LOG_FILE ...] > fingerprints.txt

Parses clippy output where each error block looks like::

    error: <message>
       --> path/to/file.rs:LINE:COL
        |
    LINE | <code>
        |
    note: <optional>
        = note: `-D clippy::<lint-name>` implied by `-D warnings`
        = help: ...

Clippy emits the `-D clippy::<lint>` note line ONLY ONCE per lint kind per
compilation run. Subsequent errors with the same `error:` message text omit
the note. We therefore build a map of `error_message -> lint_name` from the
first occurrence of each message and propagate the lint name to later blocks.

Outputs `file|lint_name` tuples (one per line, deduplicated, sorted).
The `file` path is normalized to use the form `crates/<crate>/...` so
fingerprints are stable across line/column shifts within a file.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ERROR_RE = re.compile(r"^error: (.+)$")
LOC_RE = re.compile(r"^\s*-->\s+(\S+):\d+:\d+\s*$")
DASH_D_RE = re.compile(r"`-D\s+(clippy::[a-z][a-z0-9_-]+)`")
# Rustc-level lints surface here too (e.g. `-D unused-variables`). Track them
# under a `rustc::` namespace so they are distinguishable from clippy lints.
DASH_D_RUSTC_RE = re.compile(r"`-D\s+([a-z][a-z0-9_-]+)`")
# clippy emits a help URL with the lint anchor on EVERY error block, e.g.:
#   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#manual_range_contains
# This is the most reliable lint identifier because it survives clippy's
# "suppress repeated -D notes" behavior across error variants of the same kind.
HELP_URL_RE = re.compile(r"rust-clippy/[^/]+/index\.html#([a-z][a-z0-9_]+)")


def parse_log(path: Path, msg_to_lint: dict[str, str]) -> list[tuple[str, str]]:
    """Parse one clippy log file. Returns list of (file, lint_name) tuples.

    Mutates msg_to_lint with newly-discovered (error_message -> lint) bindings.
    Errors with no resolvable lint are skipped (and reported on stderr).
    """
    fingerprints: list[tuple[str, str]] = []
    text = path.read_text(encoding="utf-8", errors="replace")

    # Split into blocks. A new block starts at every `error:` line.
    # We use a regex split that keeps the matched line attached to its body.
    block_starts = [m.start() for m in re.finditer(r"^error: ", text, re.MULTILINE)]
    if not block_starts:
        return fingerprints

    block_starts.append(len(text))
    for i in range(len(block_starts) - 1):
        block = text[block_starts[i] : block_starts[i + 1]]
        # Skip the "could not compile" terminal blocks
        first_line = block.split("\n", 1)[0]
        msg_match = ERROR_RE.match(first_line)
        if not msg_match:
            continue
        message = msg_match.group(1).strip()
        if message.startswith("could not compile"):
            continue
        if message.startswith("aborting due to"):
            continue

        # First file location in the block
        file_path: str | None = None
        for line in block.split("\n"):
            loc = LOC_RE.match(line)
            if loc:
                file_path = loc.group(1)
                break
        if file_path is None:
            continue

        # Lint identifier — prefer the help URL anchor (present on every block),
        # fall back to the `-D clippy::<lint>` note (only on first block of each
        # lint kind), then to the per-message memo from earlier blocks.
        lint: str | None = None
        url_match = HELP_URL_RE.search(block)
        if url_match:
            lint = f"clippy::{url_match.group(1).replace('_', '-')}"
        else:
            d_match = DASH_D_RE.search(block)
            if d_match:
                lint = d_match.group(1)
            else:
                rustc_match = DASH_D_RUSTC_RE.search(block)
                if rustc_match and "clippy::" not in rustc_match.group(1):
                    lint = f"rustc::{rustc_match.group(1)}"
                else:
                    lint = msg_to_lint.get(message)
        if lint is not None:
            msg_to_lint.setdefault(message, lint)

        if lint is None:
            print(
                f"WARN: unresolvable lint for error in {file_path}: {message[:80]!r}",
                file=sys.stderr,
            )
            continue

        fingerprints.append((file_path, lint))

    return fingerprints


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2

    msg_to_lint: dict[str, str] = {}
    all_fps: set[tuple[str, str]] = set()
    for arg in argv[1:]:
        path = Path(arg)
        if not path.exists():
            print(f"WARN: missing log {path}", file=sys.stderr)
            continue
        fps = parse_log(path, msg_to_lint)
        all_fps.update(fps)

    for file_path, lint in sorted(all_fps):
        print(f"{file_path}|{lint}")

    print(
        f"# {len(all_fps)} unique fingerprints across {len(argv) - 1} log file(s)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
