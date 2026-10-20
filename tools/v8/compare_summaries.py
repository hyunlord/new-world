#!/usr/bin/env python3
"""Compare the simulation content of two probe summary.json files.

Timing (wall seconds, ms per agent-tick) and the run label `git_sha` are
excluded. Keys present in only one summary are reported separately; V8-0
identity (tools/v8/identity.sh) requires zero differences AND zero one-sided
paths. Copied from tools/revitalize/compare_summaries.py (Pass 21) with the
`git_sha` label added to the excluded keys.

usage: compare_summaries.py REFERENCE.json CANDIDATE.json [--removed-keys FILE]
exit 0 when every simulation value is identical and no path is one-sided, 1 otherwise.

--removed-keys FILE lists keys a later commit removed on purpose (one per line,
`#` comments; tools/v8/identity_removed_keys.txt). A reference-only path whose
path has such a key as one of its components (the key itself, or a per-day
statistic below it such as `days.1.<key>.max`) is dropped ONLY if its reference
value is zero, false, null or empty; any other value is still a difference.
"""
import json
import sys

TIMING_KEYS = {"wall_seconds", "ms_per_agent_tick", "timing"}
LABEL_KEYS = {"git_sha"}
EXCLUDED_KEYS = TIMING_KEYS | LABEL_KEYS


def flatten(value, prefix=""):
    """Yield (dotted path, leaf) pairs; list items are indexed by position."""
    if isinstance(value, dict):
        for key, child in value.items():
            if key in EXCLUDED_KEYS:
                continue
            yield from flatten(child, f"{prefix}.{key}" if prefix else key)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from flatten(child, f"{prefix}[{index}]")
    else:
        yield prefix, value


def compare(reference, candidate):
    """Return (shared count, differences, reference-only paths, candidate-only paths)."""
    ref = dict(flatten(reference))
    cand = dict(flatten(candidate))
    shared = sorted(set(ref) & set(cand))
    diffs = [(path, ref[path], cand[path]) for path in shared if ref[path] != cand[path]]
    return len(shared), diffs, sorted(set(ref) - set(cand)), sorted(set(cand) - set(ref))


def removed_keys(path):
    keys = set()
    with open(path) as stream:
        for line in stream:
            key = line.split("#", 1)[0].strip()
            if key:
                keys.add(key)
    return keys


def components(path):
    return {part.split("[", 1)[0] for part in path.split(".")}


def main(argv):
    removed = set()
    if "--removed-keys" in argv:
        i = argv.index("--removed-keys")
        removed = removed_keys(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    if len(argv) != 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    with open(argv[1]) as stream:
        reference = json.load(stream)
    with open(argv[2]) as stream:
        candidate = json.load(stream)
    shared, diffs, ref_only, cand_only = compare(reference, candidate)
    if removed:
        ref_flat = dict(flatten(reference))
        allowed = [p for p in ref_only if components(p) & removed and ref_flat[p] in (0, 0.0, False, None, "", [], {})]
        ref_only = [p for p in ref_only if p not in allowed]
        print(f"removed-key paths dropped (reference value zero/empty): {len(allowed)}")
    print(f"shared simulation values: {shared}")
    print(f"differences: {len(diffs)}")
    for path, a, b in diffs[:40]:
        print(f"  {path}: reference={a!r} candidate={b!r}")
    print(f"reference-only paths: {len(ref_only)} {ref_only[:10]}")
    print(f"candidate-only paths: {len(cand_only)} {cand_only[:10]}")
    return 0 if not diffs and not ref_only and not cand_only else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
