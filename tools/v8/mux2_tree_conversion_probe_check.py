"""Validate exact ordered receipts from the real Godot tree-conversion probe."""
import argparse
import re
from pathlib import Path

ACTUAL = re.compile(r"MUX2_TREE_CONVERSION actual version=(0|[1-9][0-9]*) rows=(0|[1-9][0-9]*) unique=(0|[1-9][0-9]*) conversions=(0|[1-9][0-9]*) exact=(true|false)")
FIXTURE = re.compile(r"MUX2_TREE_CONVERSION fixture=(0|[1-9][0-9]*) rows=(0|[1-9][0-9]*) unique=(0|[1-9][0-9]*) conversions=(0|[1-9][0-9]*) exact=(true|false)")
FIXTURE_SHAPES = ((0, 0), (1, 1), (4096, 1), (4096, 3), (128, 128), (3, 2), (5, 4))
NEGATIVE = "MUX2_TREE_CONVERSION negative_order_detected=true"
DONE = "MUX2_TREE_BRIDGE_DONE checks=22 failures=0"


def validate_log(text: str, expect_unique: bool = False) -> list[str]:
    """Require four ordered actual/fixture/control blocks and all exact bridge checks."""
    errors: list[str] = []
    protocol: list[str] = []
    for line in text.splitlines():
        if "MUX2_TREE_" in line:
            protocol.append(line)
        if any(marker in line for marker in ("SCRIPT ERROR", "Parse Error", "panicked at")):
            errors.append("runtime/parser/panic error")
    expected: list[tuple[str, int]] = [
        ("PASS probe_explicitly_enabled", -1),
        ("PASS actual_content_bootstrap", -1),
        ("PASS actual_bridge_factory", -1),
    ]
    for call in range(4):
        if call == 2:
            expected.append(("PASS world_reset", -1))
        expected.append(("actual", call))
        expected.extend(("fixture", fixture) for fixture in range(7))
        expected.append(("negative", call))
        expected.extend((f"PASS call{call}_{label}", -1) for label in (
            "exact_schema", "aligned_columns", "original_tile_order", "actual_version"
        ))
        if call % 2:
            expected.append((f"PASS call{call}_repeated_exact_snapshot", -1))
    expected.append(("done", -1))
    if len(protocol) != len(expected):
        errors.append(f"protocol row count {len(protocol)} != {len(expected)}")
    for index, ((kind, identity), line) in enumerate(zip(expected, protocol)):
        if kind.startswith("PASS "):
            if line != "MUX2_TREE_BRIDGE_" + kind:
                errors.append(f"row{index} wrong/missing/failed bridge check")
            continue
        if kind in ("negative", "done"):
            if line != (NEGATIVE if kind == "negative" else DONE):
                errors.append(f"row{index} invalid {kind} receipt")
            continue
        match = (ACTUAL if kind == "actual" else FIXTURE).fullmatch(line)
        if match is None:
            errors.append(f"row{index} malformed or out-of-order {kind}")
            continue
        identifier, rows, unique, conversions = map(int, match.groups()[:4])
        if kind == "fixture" and (identifier != identity or (rows, unique) != FIXTURE_SHAPES[identity]):
            errors.append(f"row{index} wrong fixture identity/shape")
        if kind == "actual" and identifier > (1 << 64) - 1:
            errors.append(f"row{index} invalid version")
        if unique > rows or (rows == 0) != (unique == 0):
            errors.append(f"row{index} invalid cardinalities")
        if match.group(5) != "true":
            errors.append(f"row{index} frozen dictionary mismatch")
        wanted = unique if expect_unique else rows
        if conversions != wanted:
            errors.append(f"row{index} actual conversions={conversions}, expected={wanted}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("log", type=Path)
    parser.add_argument("--expect-unique", action="store_true")
    args = parser.parse_args()
    errors = validate_log(args.log.read_text(), args.expect_unique)
    print(f"TREE_CONVERSION_PROBE failures={len(errors)}")
    for error in errors:
        print(error)
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
