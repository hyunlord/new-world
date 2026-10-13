"""WM-1.3 Parts 5-6: the pre-registered Y verdicts, intervals and tables (GOAL_1_3.md 8; METRICS_1_3.md 5).

Reads only result JSON files (copies under docs/worldmodel/wm1/runs/), never data or checkpoints:

- the calendar family's validation selection `calendar_val_<ds>_<valsha>/calendar_val.json` and test numbers
  `calendar_<ds>_<testsha>/calendar.json` (members, persistence, ridge) for the four w13 datasets, and the legacy
  test `calendar_w13_v16_n400_on_p10_v16_<testsha>/calendar.json`;
- the models' `metrics.json` (`<run>/`, `<run>_short/` = the WM-1.2-rule checkpoint, `<run>_legacy/` = the p10_v16
  test), WM-1.2's runs on the new test (`m1tf_p10v16_r3_s<S>_on_w13v16/`), `val_metrics.json` / `config.json` for
  convergence, and the probe `probe_m1tf_w13v17_n400_s0/probe.json`.

Every interval is `wm1.bootstrap`'s world-level paired bootstrap (B = 2,000, `default_rng(0)`, 95 % percentile).
Writes `judge.json` and `judge.md`. A missing file leaves its row empty (`n/a`), never a guess.

usage: python3 tools/worldmodel/wm1/judge_1_3.py --runs docs/worldmodel/wm1/runs --val-sha SHA8 --test-sha SHA8 \
           --out docs/worldmodel/wm1/runs/judge_1_3
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
from wm1 import bootstrap as bs

VARS = ("food_stock", "wood_stock", "fire_lit", "cold_members", "alive", "deadwood_total", "food_on_bushes_total",
        "adults_working")
SKILL_VARS = {"food": "food_stock", "alive": "alive", "bushes": "food_on_bushes_total", "working adults": "adults_working"}
SKILL_VARS_V17 = SKILL_VARS | {"wood": "wood_stock"}
V16 = ("w13_v16_n80", "w13_v16_n200", "w13_v16_n400")
DATASETS = V16 + ("w13_v17_n400",)
N_OF = {"w13_v16_n80": 80, "w13_v16_n200": 200, "w13_v16_n400": 400, "w13_v17_n400": 400}
RUNS = {"w13_v16_n400": ("m1tf_w13v16_n400_s0", "m1tf_w13v16_n400_s1", "m1tf_w13v16_n400_s2"),
        "w13_v16_n80": ("m1tf_w13v16_n80_s0", "m1tf_w13v16_n80_s1"),
        "w13_v16_n200": ("m1tf_w13v16_n200_s0",),
        "w13_v17_n400": ("m1tf_w13v17_n400_s0", "m1tf_w13v17_n400_s1")}
WM12_RUNS = ("m1tf_p10v16_r3_s0", "m1tf_p10v16_r3_s1", "m1tf_p10v16_r3_s2")
PROBE_RUN = "m1tf_w13v17_n400_s0"
# (evaluate key, horizon) -> (raw selection task, bias-removed selection task)
CELLS = {("T2_any", "168"): ("T2any_168", "T2anybr_168"), ("T2_any", "24"): ("T2any_24", "T2anybr_24"),
         ("T2", "168"): ("T2_168", "T2br_168"), ("T2", "24"): ("T2_24", "T2br_24")}
REPORTED = ("cal_best", "cal_best_fair", "cal_best_anom", "an5", "apfit_abs", "persistence", "ridge")


def load(path: Path):
    return json.loads(path.read_text()) if path.exists() else None


def node(result: dict | None, key: str, h: str) -> dict | None:
    return (result or {}).get(key, {}).get(h, {}).get("test") or None


def mean_of(n: dict | None, br: bool) -> float | None:
    if not n:
        return None
    return n["bias_removed"]["mean"] if br else n["mean"]


def per_var(n: dict | None, br: bool) -> list | None:
    if not n:
        return None
    return n["bias_removed"]["per_var"] if br else n["per_var"]


def worlds(n: dict | None, br: bool, var: int | None = None) -> dict | None:
    if not n:
        return None
    return bs.world_scores(n, br) if var is None else bs.world_var_scores(n, var, br)


class Data:
    """Every result file of the run, by name."""

    def __init__(self, runs: Path, val_sha: str, test_sha: str):
        self.cal_val = {ds: load(runs / f"calendar_val_{ds}_{val_sha}" / "calendar_val.json") for ds in DATASETS}
        self.cal_test = {ds: load(runs / f"calendar_{ds}_{test_sha}" / "calendar.json") for ds in DATASETS}
        self.cal_legacy = load(runs / f"calendar_w13_v16_n400_on_p10_v16_{test_sha}" / "calendar.json")
        self.metrics = {r: load(runs / r / "metrics.json") for rs in RUNS.values() for r in rs}
        self.short = {r: load(runs / f"{r}_short" / "metrics.json") for rs in RUNS.values() for r in rs}
        self.legacy = {r: load(runs / f"{r}_legacy" / "metrics.json") for r in RUNS["w13_v16_n400"]}
        self.xworld = {r: load(runs / f"{r}_on_w13v16" / "metrics.json") for r in WM12_RUNS}
        self.val = {r: load(runs / r / "val_metrics.json") for rs in RUNS.values() for r in rs}
        self.config = {r: load(runs / r / "config.json") for rs in RUNS.values() for r in rs}
        self.probe = load(runs / f"probe_{PROBE_RUN}" / "probe.json")
        self.runs_dir = runs

    def member(self, ds: str, task: str, which: str = "cal_best") -> str | None:
        v = self.cal_val.get(ds)
        if not v:
            return None
        sel = v["selection"][task]
        return sel.get(which) if which != "cal_best_anom" else sel.get("cal_best_anom", sel["cal_best"])

    def cal_node(self, ds: str, name: str, key: str, h: str, legacy: bool = False) -> dict | None:
        """A calendar member, a selected one (`cal_best*`, by the val selection of `ds`'s matching task) or a
        reference on `ds`'s test (or the legacy test)."""
        res = self.cal_legacy if legacy else self.cal_test.get(ds)
        if not res:
            return None
        if name.startswith("cal_best"):
            br = name == "cal_best_anom"
            task = CELLS[(key, h)][1 if br else 0]
            name = self.member(ds, task, "cal_best_anom" if br else ("cal_best_fair" if name == "cal_best_fair" else "cal_best"))
        if name in res["test"]:
            return node(res["test"][name], key, h)
        return node(res["references"].get(name), key, h)


def model_scores(results: list[dict | None], key: str, h: str, br: bool, var: int | None = None) -> list[dict]:
    return [w for w in (worlds(node(r, key, h), br, var) for r in results) if w]


def rule(data: Data, ds: str, runs: tuple, key: str, h: str, br: bool, label: str, which: dict | None = None) -> dict:
    """The Y1 rule for the runs of `ds` against `cal_best` (raw) or `cal_best_anom` (bias-removed)."""
    results = [(which or data.metrics).get(r) for r in runs]
    seeds = model_scores(results, key, h, br)
    base_name = "cal_best_anom" if br else "cal_best"
    member = data.member(ds, CELLS[(key, h)][1 if br else 0], base_name)
    base = worlds(data.cal_node(ds, base_name, key, h), br)
    out = {"label": label, "dataset": ds, "against": f"{base_name} = {member}", "seeds_present": len(seeds),
           "seeds_expected": len(runs), "task": CELLS[(key, h)][1 if br else 0]}
    if not seeds or base is None:
        return out | {"verdict": "n/a"}
    return out | bs.verdict(seeds, base)


def y3(data: Data) -> dict:
    """Data vs steps (raw, 168 h every hour), intervals only."""
    key, h = "T2_any", "168"
    n80 = model_scores([data.metrics.get(r) for r in RUNS["w13_v16_n80"]], key, h, False)
    n80s = model_scores([data.short.get(r) for r in RUNS["w13_v16_n80"]], key, h, False)
    n400 = model_scores([data.metrics.get(r) for r in RUNS["w13_v16_n400"]], key, h, False)
    wm12 = model_scores([data.xworld.get(r) for r in WM12_RUNS], key, h, False)
    c80 = worlds(data.cal_node("w13_v16_n80", "cal_best", key, h), False)
    c400 = worlds(data.cal_node("w13_v16_n400", "cal_best", key, h), False)
    out = {"members": {"cal_best_n80": data.member("w13_v16_n80", "T2any_168"),
                       "cal_best_n400": data.member("w13_v16_n400", "T2any_168")}}
    if n80 and n80s and len(n80) == len(n80s):
        out["delta_steps"] = bs.interval(bs.diff(bs.average(n80), bs.average(n80s))) | {"seeds": len(n80)}
    if n80 and n400:
        out["delta_data"] = bs.interval(bs.diff(bs.average(n400), bs.average(n80))) | {"seeds": [len(n400), len(n80)]}
    if c80 and c400:
        out["delta_cal"] = bs.interval(bs.diff(c400, c80))
    if n80 and n400 and c80 and c400:
        dd = bs.diff(bs.diff(bs.average(n400), bs.average(n80)), bs.diff(c400, c80))
        out["delta_data_minus_delta_cal"] = bs.interval(dd)
        out["scale_helps_model_more"] = out["delta_data_minus_delta_cal"]["hi"] < 0
    if n80 and wm12:
        out["delta_steps_xworld"] = bs.interval(bs.diff(bs.average(n80), bs.average(wm12))) | {
            "seeds": [len(n80), len(wm12)], "confounded": "WM-1.2's runs: other training worlds (p10 seeds 20000-20079), "
                                                          "16 val worlds, 3,000-step cap"}
        out["wm12_runs_on_new_test"] = bs.interval(bs.average(wm12)) | {"seeds": [float(np.mean(list(w.values()))) for w in wm12]}
    return out


def y5(data: Data) -> dict:
    """Learning curve on the game_v16 test: M1b, cal_best, an5, apfit_abs at N = 80, 200, 400 (168 h and 24 h
    every hour, raw and bias-removed), with intervals and per-variable nMAE."""
    out = {}
    for h in ("168", "24"):
        for br in (False, True):
            tab = {}
            for ds in V16:
                row = {}
                seeds = model_scores([data.metrics.get(r) for r in RUNS[ds]], "T2_any", h, br)
                res = [data.metrics.get(r) for r in RUNS[ds]]
                if seeds:
                    pv = [per_var(node(r, "T2_any", h), br) for r in res if node(r, "T2_any", h)]
                    row["M1b"] = bs.interval(bs.average(seeds)) | {"seeds": len(seeds),
                                                                   "per_var": np.mean(pv, axis=0).tolist()}
                for name in ("cal_best_anom" if br else "cal_best", "an5", "apfit_abs"):
                    n = data.cal_node(ds, name, "T2_any", h)
                    if n:
                        row[name] = bs.interval(worlds(n, br)) | {"per_var": per_var(n, br)}
                        if name.startswith("cal_best"):
                            row[name]["member"] = data.member(ds, CELLS[("T2_any", h)][1 if br else 0], name)
                tab[str(N_OF[ds])] = row
            out[f"T2_any_{h}" + ("_br" if br else "")] = tab
    return out


def skills(data: Data, ds: str, variables: dict) -> dict:
    """Per-variable skill 1 - nMAE(model seed mean) / nMAE(baseline) at T2-any 168 h with world-level intervals:
    raw against `cal_best` and `apfit_abs`, bias-removed against `cal_best_anom` and `apfit_abs` (bias-removed)."""
    out = {}
    for br in (False, True):
        tab = {}
        for label, var in variables.items():
            j = VARS.index(var)
            seeds = model_scores([data.metrics.get(r) for r in RUNS[ds]], "T2_any", "168", br, j)
            row = {}
            for ref in ("cal_best_anom" if br else "cal_best", "apfit_abs"):
                base = worlds(data.cal_node(ds, ref, "T2_any", "168"), br, j)
                if seeds and base:
                    model = bs.average(seeds)
                    row[ref] = bs.skill_interval(model, base) | {
                        "model_nmae": float(np.mean(list(model.values()))), "base_nmae": float(np.mean(list(base.values())))}
            tab[label] = row
        out["bias_removed" if br else "raw"] = tab
    return out


def tables(data: Data, ds: str) -> dict:
    """Every reported predictor's raw and bias-removed T2 scores on `ds`'s test (per variable, mean, interval of the
    mean over worlds); the models' seeds and seed mean."""
    out = {}
    runs = RUNS[ds]
    for (key, h) in CELLS:
        for br in (False, True):
            rows = {}
            for name in REPORTED:
                n = data.cal_node(ds, name, key, h)
                if n:
                    rows[name] = {"mean": mean_of(n, br), "per_var": per_var(n, br), "ci": bs.interval(worlds(n, br))}
            have = []
            for r in runs:
                n = node(data.metrics.get(r), key, h)
                if n:
                    rows[r] = {"mean": mean_of(n, br), "per_var": per_var(n, br), "ci": bs.interval(worlds(n, br))}
                    have.append(n)
            if have:
                rows["M1b_mean"] = {"mean": float(np.mean([mean_of(n, br) for n in have])),
                                    "per_var": np.mean([per_var(n, br) for n in have], axis=0).tolist(),
                                    "ci": bs.interval(bs.average([worlds(n, br) for n in have])), "seeds": len(have)}
            sel = {w: data.member(ds, CELLS[(key, h)][1 if w == "cal_best_anom" else 0], w)
                   for w in ("cal_best", "cal_best_fair", "cal_best_anom")}
            out[f"{key}_{h}" + ("_br" if br else "")] = {"selection": sel, "rows": rows}
    return out


def legacy(data: Data) -> dict:
    """The N = 400 models and the N = 400 calendar family on WM-1.2's 16 test worlds of p10_v16 (report only)."""
    out = {}
    for key, h in (("T2_any", "168"), ("T2_any", "24")):
        for br in (False, True):
            seeds = model_scores([data.legacy.get(r) for r in RUNS["w13_v16_n400"]], key, h, br)
            row = {}
            if seeds:
                row["M1b_n400"] = bs.interval(bs.average(seeds)) | {"seeds": [float(np.mean(list(s.values()))) for s in seeds]}
            name = "cal_best_anom" if br else "cal_best"
            n = data.cal_node("w13_v16_n400", name, key, h, legacy=True)
            if n:
                row[name] = bs.interval(worlds(n, br)) | {
                    "member": data.member("w13_v16_n400", CELLS[(key, h)][1 if br else 0], name)}
                if seeds:
                    row["diff"] = bs.paired(bs.average(seeds), worlds(n, br))
            for ref in ("an5", "apfit_abs", "persistence", "ridge"):
                m = data.cal_node("w13_v16_n400", ref, key, h, legacy=True)
                if m:
                    row[ref] = bs.interval(worlds(m, br))
            out[f"{key}_{h}" + ("_br" if br else "")] = row
    return out


def convergence(data: Data) -> dict:
    """Stop reason, best step, last step, seconds and WM-1.2-rule checkpoint of every run; "not converged" when the
    run ended on `max_steps` or `time` with its best step in the last 10 %."""
    out = {}
    for ds, runs in RUNS.items():
        for r in runs:
            v = data.val.get(r)
            if not v:
                out[r] = {"present": False}
                continue
            last, best = v["last_step"], v["best_step"]
            out[r] = {"present": True, "dataset": ds, "stop": v["stop"], "best_step": best, "last_step": last,
                      "seconds": v["seconds"], "best_val_t2_24": v["best_val_t2_24"], "round_counts": v["round_counts"],
                      "converged": not (v["stop"] in ("max_steps", "time") and best >= 0.9 * last),
                      "wm12rule": v.get("wm12rule"), "train_episodes": (data.config.get(r) or {}).get("train_episodes")}
    return out


def y6(data: Data) -> dict:
    p = data.probe
    if not p:
        return {"verdict": "n/a (dropped or missing)"}
    metric = lambda v: v.get("r2_test", v.get("accuracy_test"))
    out = {"model": {k: metric(v) for k, v in p["model"].items()},
           "reference_raw_row": {k: metric(v) for k, v in p["reference_raw_row"].items()},
           "reference_columns": len(p["reference_columns"]), "row": p["row"]}
    if "model_observable" in p:
        mo, ro = p["model_observable"], p["reference_observable"]
        out["observable"] = {
            k: {"model": metric(mo[k]) if "skipped" not in mo[k] else None,
                "reference": metric(ro[k]) if "skipped" not in ro[k] else None,
                "majority": mo[k].get("majority_train_class_accuracy_test"), "detail": mo[k]} for k in mo}
    return out


def run(runs: Path, val_sha: str, test_sha: str) -> dict:
    data = Data(runs, val_sha, test_sha)
    y = {"Y1": rule(data, "w13_v16_n400", RUNS["w13_v16_n400"], "T2_any", "168", False, "v16 N=400 raw T2 168 h every-hour"),
         "Y1b": rule(data, "w13_v16_n400", RUNS["w13_v16_n400"], "T2_any", "24", False, "v16 N=400 raw T2 24 h every-hour"),
         "Y2": rule(data, "w13_v16_n400", RUNS["w13_v16_n400"], "T2_any", "168", True, "v16 N=400 bias-removed T2 168 h every-hour"),
         "Y3": y3(data),
         "Y4": rule(data, "w13_v17_n400", RUNS["w13_v17_n400"], "T2_any", "168", False, "v17 N=400 raw T2 168 h every-hour"),
         "Y4br": rule(data, "w13_v17_n400", RUNS["w13_v17_n400"], "T2_any", "168", True, "v17 N=400 bias-removed T2 168 h every-hour"),
         "Y5": y5(data), "Y6": y6(data)}
    # Reported beside Y1: the same rule at N = 80 and N = 200 (each against its own N's cal_best).
    y["Y1_n80"] = rule(data, "w13_v16_n80", RUNS["w13_v16_n80"], "T2_any", "168", False, "v16 N=80 raw T2 168 h every-hour")
    y["Y1_n200"] = rule(data, "w13_v16_n200", RUNS["w13_v16_n200"], "T2_any", "168", False, "v16 N=200 raw T2 168 h every-hour")
    return {"Y": y, "skills": {"w13_v16_n400": skills(data, "w13_v16_n400", SKILL_VARS),
                               "w13_v17_n400": skills(data, "w13_v17_n400", SKILL_VARS_V17)},
            "tables": {ds: tables(data, ds) for ds in DATASETS}, "legacy": legacy(data), "convergence": convergence(data),
            "val_sha": val_sha, "test_sha": test_sha}


def fmt(x, digits: int = 3) -> str:
    return "—" if x is None or (isinstance(x, float) and not np.isfinite(x)) else f"{x:.{digits}f}"


def ci(x: dict | None, digits: int = 3) -> str:
    return "—" if not x else f"{fmt(x['point'], digits)} [{fmt(x['lo'], digits)}, {fmt(x['hi'], digits)}]"


def markdown(r: dict) -> str:
    y = r["Y"]
    lines = ["# WM-1.3 judge output", "", "Intervals: world-level paired bootstrap, B = 2,000, 95 % percentile.", "",
             "## Y table", "", "| # | comparison | M1b seed mean (seeds) | baseline | model − baseline [95 % CI] | verdict |",
             "|---|---|---|---|---|---|"]
    for k in ("Y1", "Y1b", "Y2", "Y4", "Y4br", "Y1_n80", "Y1_n200"):
        x = y[k]
        if x.get("verdict") == "n/a" or "ci_diff" not in x:
            lines.append(f"| {k} | {x['label']} | n/a | {x['against']} | — | n/a |")
            continue
        seeds = ", ".join(fmt(s) for s in x["seeds"])
        lines.append(f"| {k} | {x['label']} | {fmt(x['model'])} ({seeds}) | {x['against']} {fmt(x['baseline'])} | "
                     f"{ci(x['ci_diff'])} | **{x['verdict']}** |")
    t = y["Y3"]
    lines += ["", "### Y3 data vs steps (v16 test, raw, T2 168 h every hour; no verdict)", "",
              "| quantity | point [95 % CI] |", "|---|---|"]
    for k in ("delta_steps", "delta_data", "delta_cal", "delta_data_minus_delta_cal", "delta_steps_xworld",
              "wm12_runs_on_new_test"):
        lines.append(f"| {k} | {ci(t.get(k), 4)} |")
    lines.append(f"| scale helps the model more than the calendar (CI of Δ_data − Δ_cal below 0) | "
                 f"{t.get('scale_helps_model_more', 'n/a')} |")
    for name, tab in y["Y5"].items():
        lines += ["", f"### Y5 learning curve — {name}", "", "| N | M1b | cal_best (member) | an5 | apfit_abs |", "|---|---|---|---|---|"]
        for n, row in tab.items():
            cb = row.get("cal_best_anom") or row.get("cal_best")
            lines.append(f"| {n} | {ci(row.get('M1b'))} | {ci(cb)} ({(cb or {}).get('member', '—')}) | {ci(row.get('an5'))} | "
                         f"{ci(row.get('apfit_abs'))} |")
    for ds, sk in r["skills"].items():
        for which, tab in sk.items():
            lines += ["", f"### per-variable skill, {ds}, {which} (1 − nMAE / baseline, T2-any 168 h)", "",
                      "| variable | baseline | model nMAE | baseline nMAE | skill [95 % CI] |", "|---|---|---|---|---|"]
            for label, row in tab.items():
                for ref, v in row.items():
                    lines.append(f"| {label} | {ref} | {fmt(v['model_nmae'])} | {fmt(v['base_nmae'])} | {ci(v)} |")
    z = y["Y6"]
    if "model" in z:
        lines += ["", f"### Y6 probe ({PROBE_RUN}, test; R², classification: accuracy)", "", "| label | model | raw row "
                  f"({z['reference_columns']} columns) |", "|---|---:|---:|"]
        for k, v in z["model"].items():
            lines.append(f"| {k} | {fmt(v)} | {fmt(z['reference_raw_row'].get(k))} |")
        for k, v in z.get("observable", {}).items():
            extra = f" (majority {fmt(v['majority'])})" if v.get("majority") is not None else ""
            lines.append(f"| {k}{extra} | {fmt(v['model'])} | {fmt(v['reference'])} |")
    lines += ["", "## Convergence", "", "| run | stop | best step | last step | seconds | converged | WM-1.2-rule best step / stop |",
              "|---|---|---:|---:|---:|---|---|"]
    for run_name, v in r["convergence"].items():
        if not v.get("present"):
            lines.append(f"| {run_name} | missing | | | | | |")
            continue
        w = v.get("wm12rule") or {}
        lines.append(f"| {run_name} | {v['stop']} | {v['best_step']} | {v['last_step']} | {v['seconds']:.0f} | "
                     f"{'yes' if v['converged'] else 'not converged'} | {w.get('best_step', '—')} / {w.get('stop', '—')} |")
    lines += ["", "## Legacy test (p10_v16, 16 worlds; report only)", "", "| cell | M1b N=400 | cal_best (member) | model − cal [CI] | an5 | apfit_abs |",
              "|---|---|---|---|---|---|"]
    for cell, row in r["legacy"].items():
        cb = row.get("cal_best_anom") or row.get("cal_best")
        lines.append(f"| {cell} | {ci(row.get('M1b_n400'))} | {ci(cb)} ({(cb or {}).get('member', '—')}) | {ci(row.get('diff'))} | "
                     f"{ci(row.get('an5'))} | {ci(row.get('apfit_abs'))} |")
    for ds, tabs in r["tables"].items():
        for name, t in tabs.items():
            sel = t["selection"]
            lines += ["", f"## {ds} — {name} (test; cal_best `{sel['cal_best']}`, cal_best_fair `{sel['cal_best_fair']}`, "
                      f"cal_best_anom `{sel['cal_best_anom']}`)", "", "| predictor | " + " | ".join(VARS) + " | mean [95 % CI] |",
                      "|---|" + "---:|" * (len(VARS) + 1)]
            for n, v in t["rows"].items():
                lines.append(f"| {n} | " + " | ".join(fmt(x) for x in v["per_var"]) + f" | {ci(v['ci'])} |")
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--runs", default="docs/worldmodel/wm1/runs")
    ap.add_argument("--val-sha", required=True)
    ap.add_argument("--test-sha", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv)
    r = run(Path(args.runs), args.val_sha, args.test_sha)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "judge.json").write_text(json.dumps(r, indent=1, sort_keys=True, default=float) + "\n")
    (out / "judge.md").write_text(markdown(r))
    print(json.dumps({k: v.get("verdict") for k, v in r["Y"].items() if isinstance(v, dict) and "verdict" in v}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
