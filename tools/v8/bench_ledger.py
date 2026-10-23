#!/usr/bin/env python3
"""The benchmark ledger (M1.5 goal 6.1, ADR 0016 C3): docs/perf/PERF_LEDGER.md.

usage: bench_ledger.py rows   BENCH_JSON --pass NAME [--note TEXT] [--date YYYY-MM-DD]
       bench_ledger.py append BENCH_JSON LEDGER --pass NAME (--gate-log GATE_TXT | --re-measure TEXT)
                              [--note TEXT] [--date YYYY-MM-DD]
       bench_ledger.py check  BENCH_JSON LEDGER --pass NAME [--base COMMIT]
                              [--justified TEXT] [--changelog CHANGELOG.md]

`worldsim_perf bench` writes BENCH_JSON. `rows` prints the ledger rows of a
run, `append` adds them to LEDGER. `check` compares the run with the ledger's
latest quiet pass: any scene whose tick mean or p99, or the run's memory per
agent, is more than 10 % above that pass fails, unless `--justified TEXT` is
given and TEXT appears verbatim in the CHANGELOG (the pass's justification,
quoted). Since M2 the C3 ceilings are hard: a MISS (or a run without
ceilings) fails, and no justification excuses it. Exit 0 = pass, 1 =
regression or ceiling miss, 2 = usage or a refused append.

M4 (goal 3.6, the ledger rule): a run with any repeat started at a 1-minute
load of 8 or more is contended. `append` refuses it unless `--note` explains
it, and then marks every row's note "contended, not a base: …"; such rows stay
in the ledger as a record and are never a comparison base for `check`.

M5 (goal §5.3, the ledger rule, second fix; M4's gate compared M4 with M4
because M4 had appended its own rows before its gate):
- `check --pass NAME` names the candidate's pass. The base is the rows that
  share the `pass` value of the last row that is quiet (not contended), whose
  `pass` differs from NAME and, with `--base COMMIT`, whose commit is not a
  strict descendant of COMMIT (`git merge-base --is-ancestor`; a commit that
  git cannot resolve is kept, with a note).
- Memory: from M5 on the rule is on `alloc/agent (B)` (the allocator counter
  of `worldsim_perf`, `alloc_per_agent` in BENCH_JSON) when both the base rows
  and the run have it; RSS/agent is then printed as information and never
  fails. When the base rows have no alloc number (every row before M5), the
  old RSS/agent rule decides. The output names the deciding metric.
- `append` is refused unless `--gate-log GATE_TXT` names the gate summary of
  the run's commit (`<log_dir>/gate.txt` of `tools/v8/gate.sh`: first line
  `head <sha>`, last line `finished … status 0`; the head and the run's
  `commit` match as prefixes either way). The gate runs `check` first; the
  pass appends its rows only in its report step, after the gate is green.
- Escape hatch: `--re-measure TEXT` instead of `--gate-log` records a quiet
  re-measure of an older pass that already passed its gate (a base line for
  a new pass, a corrected number); TEXT becomes the rows' note and must start
  with "re-measure:". It cannot be combined with `--note`.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

COLUMNS = ["date", "commit", "pass", "N", "scene", "tick mean (µs)", "p99 (µs)", "max (µs)",
           "RSS/agent (B)", "alloc/agent (B)", "RSS/tile (B)", "load (1/5/15 min)", "note"]
ALLOC_COLUMN = "alloc/agent (B)"
# The header before M5 (no allocator column); its rows parse with alloc empty.
OLD_COLUMNS = [c for c in COLUMNS if c != ALLOC_COLUMN]
LIMIT = 1.10
# M4 ledger rule: rows whose note starts with this are never a comparison base.
CONTENDED_MARK = "contended, not a base"
# A repeat that started at this 1-minute load or above makes the run contended (M2 D23).
MAX_QUIET_LOAD = 8.0
# M5: the note of a row appended with --re-measure starts with this.
RE_MEASURE_MARK = "re-measure:"
# The repository whose history `--base` is resolved in (tests point it elsewhere).
GIT_DIR = Path(__file__).resolve().parents[2]


def run_rows(bench, pass_name, note, date):
    """Ledger rows (dicts keyed by COLUMNS) of one bench run."""
    rows = []
    per_agent = bench.get("bytes_per_agent")
    alloc_agent = bench.get("alloc_per_agent")
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
            ALLOC_COLUMN: "" if alloc_agent is None else f"{alloc_agent:.0f}",
            "RSS/tile (B)": "" if per_tile is None else f"{per_tile:.1f}",
            "load (1/5/15 min)": "; ".join(loads),
            "note": note,
        })
    return rows


def format_row(row):
    return "| " + " | ".join(row[c].replace("|", "/") for c in COLUMNS) + " |"


def parse_ledger(text):
    """Every table row under a header equal to COLUMNS or OLD_COLUMNS, as dicts keyed by COLUMNS
    (rows under the old header get an empty alloc/agent)."""
    rows, header = [], None
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells in (COLUMNS, OLD_COLUMNS):
            header = cells
            continue
        if header is None:
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        if len(cells) == len(header):
            row = dict(zip(header, cells))
            row.setdefault(ALLOC_COLUMN, "")
            rows.append(row)
    return rows


def number(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def ratio_line(label, before, now, unit, deciding):
    """(line, failed) for one memory metric; information-only metrics never fail."""
    ratio = now / before
    failed = deciding and ratio > LIMIT
    verdict = ("FAIL" if failed else "ok") if deciding else "information only"
    return f"{label}: {before:.0f} -> {now:.0f} {unit} (x{ratio:.3f}) {verdict}", failed


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

    def first(column):
        return next((number(r.get(column)) for r in previous if number(r.get(column))), None)

    rss_before, rss_now = first("RSS/agent (B)"), bench.get("bytes_per_agent")
    alloc_before, alloc_now = first(ALLOC_COLUMN), bench.get("alloc_per_agent")
    by_alloc = bool(alloc_before) and alloc_now is not None
    if by_alloc:
        lines.append("memory metric: alloc/agent decides (the allocator counter; RSS/agent is information)")
        line, failed = ratio_line("alloc/agent", alloc_before, alloc_now, "B", True)
        lines.append(line)
        if failed:
            failures.append("alloc/agent")
    else:
        why = "the base rows have no alloc/agent" if not alloc_before else "the run has no alloc_per_agent"
        lines.append(f"memory metric: RSS/agent decides ({why})")
    if rss_before and rss_now is not None:
        line, failed = ratio_line("RSS/agent", rss_before, rss_now, "B", not by_alloc)
        lines.append(line)
        if failed:
            failures.append("RSS/agent")
    return failures, lines


def is_base(row):
    """Whether a ledger row may be a comparison base (not marked contended)."""
    return not row.get("note", "").lower().startswith(CONTENDED_MARK)


def contended_loads(bench):
    """The 1-minute loads at or above MAX_QUIET_LOAD that any repeat of the run started at."""
    high = []
    for scene in bench.get("scenes", []):
        for repeat in scene.get("per_repeat", []):
            first = str(repeat.get("load_before", "")).split()
            if first and number(first[0]) is not None and number(first[0]) >= MAX_QUIET_LOAD:
                high.append(number(first[0]))
    return high


def git(*args):
    """(returncode, stdout) of a git command in GIT_DIR; (None, "") when git cannot run."""
    try:
        done = subprocess.run(["git", "-C", str(GIT_DIR), *args], capture_output=True, text=True,
                              timeout=30, check=False)
    except (OSError, subprocess.SubprocessError):
        return None, ""
    return done.returncode, done.stdout.strip()


def resolve(commit):
    """The full sha of `commit` in GIT_DIR, or None."""
    code, out = git("rev-parse", "--verify", "--quiet", f"{commit}^{{commit}}")
    return out if code == 0 and out else None


def descends_from(base_sha, commit, notes, cache):
    """Whether `commit` is a strict descendant of `base_sha`; None (kept, noted) when unknown."""
    if commit in cache:
        return cache[commit]
    full = resolve(commit)
    if full is None:
        notes.append(f"note: commit {commit} cannot be resolved by git; its rows are not excluded by --base")
        result = None
    elif full == base_sha:
        result = False
    else:
        code, _ = git("merge-base", "--is-ancestor", base_sha, full)
        if code not in (0, 1):
            notes.append(f"note: git merge-base failed for {commit}; its rows are not excluded by --base")
        result = code == 0
    cache[commit] = result
    return result


def latest_pass(rows, candidate=None, base=None, notes=None):
    """The rows of the latest quiet pass whose label differs from `candidate` and whose commit
    does not descend from `base` (M5 ledger rule; see the module header)."""
    notes = [] if notes is None else notes
    rows = [r for r in rows if is_base(r) and (candidate is None or r["pass"] != candidate)]
    if base:
        base_sha = resolve(base)
        if base_sha is None:
            notes.append(f"note: --base {base} cannot be resolved by git (or git is unavailable); "
                         "no row is excluded by it")
        else:
            cache = {}
            rows = [r for r in rows if not descends_from(base_sha, r["commit"], notes, cache)]
    if not rows:
        return []
    name = rows[-1]["pass"]
    return [r for r in rows if r["pass"] == name]


def gate_log_problem(path, commit):
    """None when `path` is a passing gate summary of `commit`, else the reason it is not."""
    try:
        lines = [line.strip() for line in Path(path).read_text().splitlines() if line.strip()]
    except OSError as error:
        return f"cannot read the gate log {path}: {error}"
    if not lines or not lines[0].startswith("head "):
        return f"{path}: not a gate summary (the first line is not 'head <sha>')"
    head = lines[0].split()[1] if len(lines[0].split()) > 1 else ""
    commit = str(commit or "").strip()
    if not head or not commit or commit == "unknown" or not (head.startswith(commit) or commit.startswith(head)):
        return f"{path}: the gate ran on {head or '?'}, the bench run is of {commit or '?'}"
    match = re.match(r"^finished .* status (\d+)$", lines[-1])
    if match is None:
        return f"{path}: the gate did not finish (no 'finished … status N' last line)"
    if match.group(1) != "0":
        return f"{path}: the gate failed (status {match.group(1)})"
    return None


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
    opts = {"--pass": None, "--note": "", "--date": "", "--justified": None, "--changelog": "CHANGELOG.md",
            "--base": None, "--gate-log": None, "--re-measure": None}
    while rest:
        if rest[0] not in opts or len(rest) < 2:
            print(f"bad argument {rest[0]}", file=sys.stderr)
            return 2
        opts[rest[0]] = rest[1]
        rest = rest[2:]
    if not opts["--pass"]:
        print("--pass is required", file=sys.stderr)
        return 2
    bench = json.loads(bench_path.read_text())
    if mode in ("rows", "append"):
        note = opts["--note"]
        if mode == "append":
            re_measure = opts["--re-measure"]
            if re_measure is not None:
                if note:
                    print("--re-measure TEXT is the rows' note; do not combine it with --note", file=sys.stderr)
                    return 2
                if not re_measure.lower().startswith(RE_MEASURE_MARK):
                    print(f"--re-measure TEXT must start with {RE_MEASURE_MARK!r}", file=sys.stderr)
                    return 2
                note = re_measure
            elif opts["--gate-log"] is None:
                print("append refused: --gate-log <log_dir>/gate.txt of a passing gate on this commit is "
                      "required (or --re-measure 're-measure: …' for an older pass that passed its gate)",
                      file=sys.stderr)
                return 2
            else:
                problem = gate_log_problem(opts["--gate-log"], bench.get("commit"))
                if problem:
                    print(f"append refused: {problem}", file=sys.stderr)
                    return 2
        high = contended_loads(bench)
        if high:
            if not note:
                print(f"contended run (1-min load up to {max(high):.2f} >= {MAX_QUIET_LOAD:g}): "
                      "--note must say why it is recorded; its rows are never a base", file=sys.stderr)
                return 2
            if not note.lower().startswith(CONTENDED_MARK):
                note = f"{CONTENDED_MARK}: {note}"
        rows = [format_row(r) for r in run_rows(bench, opts["--pass"], note, opts["--date"])]
        if mode == "rows":
            print("\n".join(rows))
        else:
            text = ledger.read_text()
            if not text.endswith("\n"):
                text += "\n"
            ledger.write_text(text + "\n".join(rows) + "\n")
            print(f"appended {len(rows)} rows to {ledger}")
        return 0
    notes = []
    previous = latest_pass(parse_ledger(ledger.read_text()), opts["--pass"], opts["--base"], notes)
    for line in notes:
        print(line)
    if not previous:
        print(f"{ledger}: no ledger rows of another quiet pass than {opts['--pass']!r} "
              "(absence is a failure)")
        return 1
    failures, lines = compare(bench, previous)
    print(f"candidate pass {opts['--pass']!r}; compared with pass {previous[0]['pass']} ({previous[0]['commit']})")
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
