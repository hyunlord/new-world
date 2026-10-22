#!/usr/bin/env python3
"""The benchmark ledger (M1.5 goal 6.1, ADR 0016 C3): docs/perf/PERF_LEDGER.md.

usage: bench_ledger.py rows   BENCH_JSON --pass NAME [--note TEXT] [--date YYYY-MM-DD]
       bench_ledger.py append BENCH_JSON LEDGER --pass NAME [--note TEXT] [--date YYYY-MM-DD]
       bench_ledger.py check  BENCH_JSON LEDGER [--justified TEXT] [--changelog CHANGELOG.md]

`worldsim_perf bench` writes BENCH_JSON. `rows` prints the ledger rows of a
run, `append` adds them to LEDGER. `check` compares the run with the ledger's
latest pass (the rows that share the last row's `pass` value): any scene whose
tick mean or p99, or the run's RSS per agent, is more than 10 % above that
pass fails, unless `--justified TEXT` is given and TEXT appears verbatim in
the CHANGELOG (the pass's justification, quoted). Since M2 the C3 ceilings are
hard: a MISS (or a run without ceilings) fails, and no justification excuses
it. Exit 0 = pass, 1 = regression or ceiling miss, 2 = usage.
"""
import json
import sys
from pathlib import Path

COLUMNS = ["date", "commit", "pass", "N", "scene", "tick mean (µs)", "p99 (µs)", "max (µs)",
           "RSS/agent (B)", "RSS/tile (B)", "load (1/5/15 min)", "note"]
LIMIT = 1.10


def run_rows(bench, pass_name, note, date):
    """Ledger rows (dicts keyed by COLUMNS) of one bench run."""
    rows = []
    per_agent = bench.get("bytes_per_agent")
    per_tile = bench.get("bytes_per_tile")
    for scene in bench["scenes"]:
        loads = [r.get("load_before", "") for r in scene.get("per_repeat", [])]
        rows.append({
            "date": date,
            "commit": str(bench.get("commit", "unknown"))[:8],
            "pass": pass_name,
            "N": str(scene["agents"]),
            "scene": scene["label"],
            "tick mean (µs)": f"{scene['tick_mean_us']:.1f}",
            "p99 (µs)": f"{scene['tick_p99_us']:.0f}",
            "max (µs)": f"{scene['tick_max_us']:.0f}",
            "RSS/agent (B)": "" if per_agent is None else f"{per_agent:.0f}",
            "RSS/tile (B)": "" if per_tile is None else f"{per_tile:.1f}",
            "load (1/5/15 min)": "; ".join(loads),
            "note": note,
        })
    return rows


def format_row(row):
    return "| " + " | ".join(row[c].replace("|", "/") for c in COLUMNS) + " |"


def parse_ledger(text):
    """Every table row of the ledger whose header is COLUMNS, as dicts."""
    rows, header = [], None
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if header is None:
            if cells == COLUMNS:
                header = cells
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        if len(cells) == len(header):
            rows.append(dict(zip(header, cells)))
    return rows


def number(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def compare(bench, previous):
    """(failures, lines): the run against the rows of the previous pass."""
    failures, lines = [], []
    by_scene = {row["scene"]: row for row in previous}
    for scene in bench["scenes"]:
        old = by_scene.get(scene["label"])
        if old is None:
            lines.append(f"{scene['label']}: no previous row")
            continue
        for key, column in (("tick_mean_us", "tick mean (µs)"), ("tick_p99_us", "p99 (µs)")):
            before, now = number(old[column]), float(scene[key])
            if before is None or before <= 0:
                continue
            ratio = now / before
            verdict = "FAIL" if ratio > LIMIT else "ok"
            lines.append(f"{scene['label']} {column}: {before:.1f} -> {now:.1f} (x{ratio:.3f}) {verdict}")
            if ratio > LIMIT:
                failures.append(f"{scene['label']} {column}")
    before = next((number(r["RSS/agent (B)"]) for r in previous if number(r["RSS/agent (B)"])), None)
    now = bench.get("bytes_per_agent")
    if before and now is not None:
        ratio = now / before
        verdict = "FAIL" if ratio > LIMIT else "ok"
        lines.append(f"RSS/agent: {before:.0f} -> {now:.0f} B (x{ratio:.3f}) {verdict}")
        if ratio > LIMIT:
            failures.append("RSS/agent")
    return failures, lines


def latest_pass(rows):
    if not rows:
        return []
    name = rows[-1]["pass"]
    return [r for r in rows if r["pass"] == name]


def main(argv):
    if len(argv) < 2 or argv[0] not in ("rows", "append", "check"):
        print(__doc__, file=sys.stderr)
        return 2
    mode, bench_path = argv[0], Path(argv[1])
    rest = argv[2:]
    ledger = None
    if mode in ("append", "check"):
        if not rest:
            print(__doc__, file=sys.stderr)
            return 2
        ledger, rest = Path(rest[0]), rest[1:]
    opts = {"--pass": None, "--note": "", "--date": "", "--justified": None, "--changelog": "CHANGELOG.md"}
    while rest:
        if rest[0] not in opts or len(rest) < 2:
            print(f"bad argument {rest[0]}", file=sys.stderr)
            return 2
        opts[rest[0]] = rest[1]
        rest = rest[2:]
    bench = json.loads(bench_path.read_text())
    if mode in ("rows", "append"):
        if not opts["--pass"]:
            print("--pass is required", file=sys.stderr)
            return 2
        rows = [format_row(r) for r in run_rows(bench, opts["--pass"], opts["--note"], opts["--date"])]
        if mode == "rows":
            print("\n".join(rows))
        else:
            text = ledger.read_text()
            if not text.endswith("\n"):
                text += "\n"
            ledger.write_text(text + "\n".join(rows) + "\n")
            print(f"appended {len(rows)} rows to {ledger}")
        return 0
    previous = latest_pass(parse_ledger(ledger.read_text()))
    if not previous:
        print(f"{ledger}: no ledger rows (absence is a failure)")
        return 1
    failures, lines = compare(bench, previous)
    print(f"compared with pass {previous[0]['pass']} ({previous[0]['commit']})")
    print("\n".join(lines))
    ceilings = sorted(bench.get("ceilings", {}).items())
    missed = [name for name, verdict in ceilings if verdict != "PASS"]
    for name, verdict in ceilings:
        print(f"ceiling {name}: {verdict}")
    if not ceilings:
        print("bench: FAIL (the run reports no C3 ceilings; absence is a failure)")
        return 1
    if missed:
        print(f"bench: FAIL (C3 ceiling missed: {', '.join(missed)})")
        return 1
    if failures:
        quote = opts["--justified"]
        if quote and quote in Path(opts["--changelog"]).read_text():
            print(f"JUSTIFIED by the CHANGELOG: {quote!r} ({', '.join(failures)})")
            return 0
        print(f"bench: FAIL (> +10 %: {', '.join(failures)})")
        return 1
    print("bench: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
