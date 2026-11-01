#!/usr/bin/env python3
"""Compare summary JSON using explicit schema paths, preserving raw files.

Usage: read_normalise.py <base.json> <head.json>
READ_GATE_CONFIG may name a pinned gate configuration with base_commit. Only
fbc03bdd's documented colony schema asymmetry is eligible for that exclusion.
"""
import os
import sys

from read_summary_normalise import EXCLUDED_PATHS, first_difference, load_json, normalise_pair


def main():
    if len(sys.argv) == 1:
        print("Registered summary exclusions (no recursive key exclusions):")
        print("\n".join(EXCLUDED_PATHS))
        return 0
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        config_path = os.environ.get("READ_GATE_CONFIG")
        config = load_json(config_path) if config_path else {}
        a, b, masked = normalise_pair(load_json(sys.argv[1]), load_json(sys.argv[2]), config.get("base_commit"))
        difference = first_difference(a, b)
        print("NORMALISED MISMATCH: " + difference if difference else "NORMALISED MATCH")
        print("Applied exclusions: " + ", ".join(masked))
        return 1 if difference else 0
    except (ValueError, OSError, TypeError, AttributeError) as error:
        print(f"NORMALISED INVALID: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
