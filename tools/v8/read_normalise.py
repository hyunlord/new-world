#!/usr/bin/env python3
"""Normalising script for M-READ identity comparison.

Lists and excludes the new keys added by M-READ that are presentation-only
(not saved, not hashed). Colony gameplay stays byte-identical when these
keys are removed from the comparison.

Usage:
    python3 tools/v8/read_normalise.py <summary_a.json> <summary_b.json>

Exits 0 when the files are identical after removing the excluded keys;
exits 1 with a diff when they differ.
"""
import json
import sys

# Exactly the new keys M-READ adds to the why-not records in MindState.
# These are #[serde(skip)] presentation-only fields.
EXCLUDED_KEYS = [
    "why_not",          # MindState.why_not (per-person why-not ring)
    "diagnosis_cache",  # SimResources.diagnosis_cache
    "requests_cache",   # SimResources.requests_cache
]


def strip_keys(obj, keys_to_remove):
    """Recursively remove keys from nested dicts."""
    if isinstance(obj, dict):
        return {
            k: strip_keys(v, keys_to_remove)
            for k, v in obj.items()
            if k not in keys_to_remove
        }
    if isinstance(obj, list):
        return [strip_keys(item, keys_to_remove) for item in obj]
    return obj


def main():
    if len(sys.argv) == 1:
        # Just print the excluded keys.
        print("M-READ excluded keys for normalised comparison:")
        for key in EXCLUDED_KEYS:
            print(f"  {key}")
        return 0

    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <summary_a.json> <summary_b.json>", file=sys.stderr)
        return 2

    path_a, path_b = sys.argv[1], sys.argv[2]
    with open(path_a) as f:
        a = json.load(f)
    with open(path_b) as f:
        b = json.load(f)

    a_clean = strip_keys(a, EXCLUDED_KEYS)
    b_clean = strip_keys(b, EXCLUDED_KEYS)

    if a_clean == b_clean:
        print(f"NORMALISED MATCH: {path_a} == {path_b} (excluded {len(EXCLUDED_KEYS)} keys)")
        return 0

    # Find differences.
    print(f"NORMALISED MISMATCH: {path_a} != {path_b}")
    a_str = json.dumps(a_clean, sort_keys=True, indent=2)
    b_str = json.dumps(b_clean, sort_keys=True, indent=2)
    a_lines = a_str.splitlines()
    b_lines = b_str.splitlines()
    diffs = 0
    for i, (la, lb) in enumerate(zip(a_lines, b_lines)):
        if la != lb:
            diffs += 1
            if diffs <= 20:
                print(f"  line {i+1}:")
                print(f"    A: {la}")
                print(f"    B: {lb}")
    if len(a_lines) != len(b_lines):
        print(f"  line count: A={len(a_lines)} B={len(b_lines)}")
    if diffs > 20:
        print(f"  ... and {diffs - 20} more differences")
    return 1


if __name__ == "__main__":
    sys.exit(main())
