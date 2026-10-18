"""The paper's figures and quoted numbers (docs/worldmodel/wm5/GOAL.md 11.2), from the committed judge files of WM-2 …
WM-5 only.

**Numbers.** `docs/worldmodel/paper/paper_src.md` writes every quoted result as a placeholder that names its source:

    {{j5:v2/cell_averages/zero/7d#3}}      a value of WM-5's judge.json, 3 decimals
    {{j5:obs2/rules/R1/ct_obs2/ci_diff#ci3}}   a CI: "+0.123 [+0.100, +0.150]" (three numbers)
    {{j4:…}} {{j3:…}} {{j2:…}}             WM-4 / WM-3 / WM-2's judge.json
    {{p5:summary/all/casts#0}}             obs2's policy summary (judge.json `describe`)

Format suffixes: `#3` (decimals), `#3s` (signed), `#0` (integer with thousands separators), `#pct1` (a share as a
percentage with 1 decimal), `#ci3` (point [lo, hi], signed). `render` replaces every placeholder by its text, writes
`paper.md` and `paper_numbers.json` (one entry per number: the text, the source file, the key path, the format), and
refuses to leave a placeholder. `check` (a unit test calls it) re-reads every source and confirms each quoted text.

**Figures** (`figures` command; matplotlib, imported only there): the headroom ladder per split and horizon with every
method; per-cell errors; the obs2 confounding (act shares per food tercile); CRN / CT validation curves (WM-4 vs WM-5).
PNG and PDF under `docs/worldmodel/paper/figures/`.

usage:
  python3 tools/worldmodel/wm5/figures.py render      # paper_src.md -> paper.md + paper_numbers.json
  python3 tools/worldmodel/wm5/figures.py check       # every quoted number against its source
  python3 tools/worldmodel/wm5/figures.py figures     # PNG + PDF
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
PAPER = REPO / "docs/worldmodel/paper"
SOURCES = {
    "j5": "docs/worldmodel/wm5/runs/judge5/judge.json",
    "j4": "docs/worldmodel/wm4/runs/judge4/judge.json",
    "j3": "docs/worldmodel/wm3/runs/judge3/judge.json",
    "j2": "docs/worldmodel/wm2/runs/judge_wm2/judge.json",
}
PLACEHOLDER = re.compile(r"\{\{(j5|j4|j3|j2|p5):([^#}]+)#([a-z0-9]+)\}\}")
MINUS = "−"


def source_of(prefix: str) -> tuple[str, list[str]]:
    """(file, key prefix) of a placeholder prefix; `p5` is the obs2 policy summary inside WM-5's judge."""
    if prefix == "p5":
        return SOURCES["j5"], ["describe", "policy_summary"]
    return SOURCES[prefix], []


def lookup(obj, keys: list[str]):
    for k in keys:
        if isinstance(obj, list):
            obj = obj[int(k)]
        else:
            obj = obj[k]
    return obj


def fmt_value(x: float, spec: str) -> str:
    if spec.startswith("pct"):
        return f"{100 * float(x):.{int(spec[3:])}f} %"
    if spec == "0":
        return f"{int(round(float(x))):,}"
    signed = spec.endswith("s")
    d = int(spec.rstrip("s"))
    s = f"{float(x):+.{d}f}" if signed else f"{float(x):.{d}f}"
    return s.replace("-", MINUS) if signed or float(x) < 0 else s


def fmt_ci(ci: dict, d: int) -> str:
    f = lambda v: fmt_value(v, f"{d}s")
    return f"{f(ci['point'])} [{f(ci['lo'])}, {f(ci['hi'])}]"


_cache: dict = {}


def load_source(path: str) -> dict:
    if path not in _cache:
        _cache[path] = json.loads((REPO / path).read_text())
    return _cache[path]


def resolve(prefix: str, key: str, spec: str) -> tuple[str, dict]:
    path, kp = source_of(prefix)
    keys = kp + key.split("/")
    val = lookup(load_source(path), keys)
    if spec.startswith("ci"):
        text = fmt_ci(val, int(spec[2:]))
    else:
        text = fmt_value(val, spec)
    return text, {"text": text, "file": path, "key": keys, "format": spec}


def render(src: Path = PAPER / "paper_src.md", out: Path = PAPER / "paper.md",
           numbers: Path = PAPER / "paper_numbers.json") -> dict:
    text = src.read_text()
    entries = []

    def sub(m):
        t, e = resolve(m.group(1), m.group(2).strip(), m.group(3))
        e["placeholder"] = m.group(0)
        entries.append(e)
        return t

    body = PLACEHOLDER.sub(sub, text)
    left = re.findall(r"\{\{[^}]*\}\}", body)
    if left:
        raise SystemExit(f"unrendered placeholders: {left[:5]}")
    out.write_text(body)
    j = {"note": "every quoted result of paper.md: its text, source file and key path (rendered from paper_src.md by "
                 "tools/worldmodel/wm5/figures.py)", "count": len(entries), "numbers": entries}
    numbers.write_text(json.dumps(j, indent=1, ensure_ascii=False) + "\n")
    return j


def check(numbers: Path = PAPER / "paper_numbers.json", paper: Path = PAPER / "paper.md") -> list[str]:
    """Problems: an entry whose source value does not format to its text, or whose text is not in paper.md."""
    j = json.loads(numbers.read_text())
    body = paper.read_text()
    bad = []
    for e in j["numbers"]:
        val = lookup(load_source(e["file"]), e["key"])
        spec = e["format"]
        t = fmt_ci(val, int(spec[2:])) if spec.startswith("ci") else fmt_value(val, spec)
        if t != e["text"]:
            bad.append(f"{e['placeholder']}: source gives {t}, quoted {e['text']}")
        if e["text"] not in body:
            bad.append(f"{e['placeholder']}: {e['text']} not in paper.md")
    if "{{" in body:
        bad.append("paper.md has an unrendered placeholder")
    return bad


# ------------------------------------------------------------------ figures ----------------------------------------
def figures(outdir: Path = PAPER / "figures") -> list[str]:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    outdir.mkdir(parents=True, exist_ok=True)
    j5 = load_source(SOURCES["j5"])
    made = []

    def save(fig, name):
        for ext in ("png", "pdf"):
            fig.savefig(outdir / f"{name}.{ext}", dpi=160, bbox_inches="tight")
        plt.close(fig)
        made.append(name)

    # 1. headroom ladder with every method, per split and horizon
    groups = {
        "v2": [("floor", "floor"), ("ridge_priv*", "ridge_priv"), ("ridge_obs", "ridge_obs"), ("zero", "zero"),
               ("an5_arm", "an5_arm"), ("wm3_full", ("wm3_full_s0", "wm3_full_s1", "wm3_full_s2")),
               ("M1b (v2)", ("wm2_m1bact_v19_s0", "wm2_m1bact_v19_s1", "wm2_m1bact_v19_s2")),
               ("CRN-v2", ("crn_v2_s0", "crn_v2_s1")), ("CT-v2", ("ct_v2_s0", "ct_v2_s1"))],
        "obs2": [("floor", "floor"), ("ridge_priv*", "ridge_priv"), ("ridge_obs", "ridge_obs"), ("zero", "zero"),
                 ("naive", "naive"), ("ipw", "ipw"), ("match5", "match5"),
                 ("wm3_full", ("ref_wm3_full_s0", "ref_wm3_full_s1", "ref_wm3_full_s2")),
                 ("M1b-obs2", ("m1b_obs2_s0", "m1b_obs2_s1")), ("M1b-obs (WM-4)", ("ref_m1b_obs_s0", "ref_m1b_obs_s1")),
                 ("CRN-obs2", ("crn_obs2_s0", "crn_obs2_s1")), ("CT-obs2", ("ct_obs2_s0", "ct_obs2_s1")),
                 ("CT-obs (WM-4)", ("ref_ct_obs_s0", "ref_ct_obs_s1"))],
    }
    fig, axes = plt.subplots(2, 2, figsize=(11, 8), sharex=False)
    for i, split in enumerate(("v2", "obs2")):
        ca = j5.get(split, {}).get("cell_averages", {})
        for k, lag in enumerate(("7d", "30d")):
            ax = axes[i][k]
            labels, vals, cols = [], [], []
            for label, names in groups[split]:
                names = (names,) if isinstance(names, str) else names
                v = [ca[n][lag] for n in names if n in ca]
                if not v:
                    continue
                labels.append(label)
                vals.append(sum(v) / len(v))
                cols.append("#888888" if label in ("floor", "ridge_priv*") else "#4477aa" if label in ("ridge_obs", "zero")
                            else "#cc6677")
            y = range(len(labels))
            ax.barh(list(y), vals, color=cols)
            ax.set_yticks(list(y)); ax.set_yticklabels(labels, fontsize=8); ax.invert_yaxis()
            if "zero" in labels:
                ax.axvline(vals[labels.index("zero")], color="#4477aa", lw=0.8, ls="--")
            ax.set_title(f"{split} test, {lag.replace('d', ' days')}", fontsize=10)
            ax.set_xlabel("12-cell scaled effect MAE (lower is better)", fontsize=8)
    fig.suptitle("Headroom ladder and methods (* diagnostic: reads hidden state)", fontsize=11)
    fig.tight_layout()
    save(fig, "ladder")

    # 2. per-cell errors (obs2 and v2), a few methods
    import numpy as np
    for split, arms, methods in (
            ("obs2", ("gf240", "sp240", "rw120"), ("floor", "ridge_obs", "zero", "ipw", "m1b_obs2_s0", "crn_obs2_s0",
                                                   "ct_obs2_s0", "ref_wm3_full_s0")),
            ("v2", ("gf240E", "sp240L", "rw120E"), ("floor", "ridge_obs", "zero", "an5_arm", "wm3_full_s0", "crn_v2_s0",
                                                    "ct_v2_s0"))):
        tab = j5.get(split, {}).get("tables", {})
        methods = [m for m in methods if m in tab]
        if not methods:
            continue
        rows = [f"{a}|{o}|{lag}" for lag in ("7d", "30d") for a in arms
                for o in ("food_stock", "alive", "deaths", "settlements_alive")]
        M = np.array([[tab[m].get(r, {}).get("mae", np.nan) for m in methods] for r in rows])
        fig, ax = plt.subplots(figsize=(1.2 + 0.9 * len(methods), 8))
        im = ax.imshow(np.minimum(M, 3.0), aspect="auto", cmap="viridis_r")
        ax.set_xticks(range(len(methods))); ax.set_xticklabels(methods, rotation=60, ha="right", fontsize=7)
        ax.set_yticks(range(len(rows))); ax.set_yticklabels(rows, fontsize=6)
        for r_ in range(M.shape[0]):
            for c_ in range(M.shape[1]):
                if np.isfinite(M[r_, c_]):
                    ax.text(c_, r_, f"{M[r_, c_]:.2f}", ha="center", va="center", fontsize=5,
                            color="white" if M[r_, c_] > 1.2 else "black")
        fig.colorbar(im, ax=ax, label="scaled MAE (clipped at 3)")
        ax.set_title(f"{split} test: per-cell scaled effect MAE", fontsize=9)
        fig.tight_layout()
        save(fig, f"cells_{split}")

    # 3. obs2 confounding: act shares per food tercile
    pol = (j5.get("describe") or {}).get("policy_summary", {}).get("by_f_tercile")
    if pol:
        fig, ax = plt.subplots(figsize=(5, 3.2))
        names = ("low food", "middle", "high food")
        bottom = np.zeros(3)
        for arm, col in (("gf240", "#117733"), ("sp240", "#cc6677"), ("rw120", "#ddaa33")):
            v = np.array([pol[str(t)]["arm_shares"][arm] for t in range(3)])
            ax.bar(names, v, bottom=bottom, color=col, label=arm)
            bottom += v
        ax.set_ylabel("share of acts"); ax.legend(fontsize=8, loc="upper center", ncol=3)
        ax.set_title("obs2: act shares per food-per-person tercile", fontsize=9)
        save(fig, "obs2_confounding")

    # 4. CRN / CT validation curves: WM-4 (time-capped, nominal epochs) vs WM-5 (step plan)
    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4))
    for ax, (wm4run, wm5run, title) in zip(axes, (("ct_v2_s0", "ct_v2_s0", "CT on v2"),
                                                  ("crn_v2_s0", "crn_v2_s0", "CRN on v2 (decoder)"))):
        for root, run, lab in ((REPO / "docs/worldmodel/wm4/runs", wm4run, "WM-4"),
                               (REPO / "docs/worldmodel/wm5/runs", wm5run, "WM-5")):
            p = root / run / "train_log.jsonl"
            if not p.exists():
                continue
            recs = [json.loads(x) for x in p.read_text().splitlines() if x.strip()]
            recs = [r for r in recs if "val_roll" in r]
            if recs:
                ax.plot([r["steps"] for r in recs], [r["val_roll"] for r in recs], marker="o", ms=3, label=lab)
        ax.set_xlabel("optimizer steps"); ax.set_ylabel("val_roll (factual)"); ax.set_title(title, fontsize=9)
        ax.legend(fontsize=8)
    fig.tight_layout()
    save(fig, "training_curves")
    return made


def main(argv=None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    cmd = argv[0] if argv else "render"
    if cmd == "render":
        j = render()
        print(f"paper.md rendered: {j['count']} numbers")
        bad = check()
        print("\n".join(bad) or "check OK")
        return 1 if bad else 0
    if cmd == "check":
        bad = check()
        print("\n".join(bad) or "check OK")
        return 1 if bad else 0
    if cmd == "figures":
        print(figures())
        return 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    raise SystemExit(main())
