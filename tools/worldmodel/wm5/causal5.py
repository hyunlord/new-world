"""CRN and the Causal Transformer as their authors train them (docs/worldmodel/wm5/GOAL.md 7.4; changes against WM-4 in
docs/worldmodel/wm5/METHODS.md D16 onward). Wraps `wm4.causal` (models, data, rollouts, `val_roll`) without editing it.

- **Step plan, not a time cap.** Each stage trains for its planned number of optimizer steps = the authors' (CRN encoder
  15,700, CRN decoder 53,800, CT 11,850), reduced only when the 300-s cost check on the actual dataset (`cost`) predicts
  more than the stage's budget: planned = floor((budget − evaluation allowance) / measured s per step) (both CRN stages
  scaled by the same factor). The time cap is a safety net of 1.25 × budget; a run stopped by it reports `stop` "time".
- **α on real progress.** α = `alpha_at(step / planned_steps, α_max)` (the authors' `AlphaRise` "exp" rule), set before
  every optimizer step, so a run that completes its plan ends at α ≈ α_max (0.99991 α_max at the last step).
- **Checkpoints.** Primary = the authors' practice, the **last** step (`model.pt`): CT's EMA weights (β 0.99) at the last
  step; CRN's last encoder and last decoder. Secondary = the lowest-`val_roll` evaluation (`model_best.pt`; WM-4's rule):
  CT's EMA weights at that evaluation; CRN's last encoder with the decoder of its lowest `val_roll` (the decoder is
  trained on the last encoder; the encoder's lowest one-step loss is logged only).
- **Evaluations**: at most 15 per stage (14 evenly spaced in the plan, and the end of the stage); `val_roll` on the whole
  val set unless one evaluation takes longer than 60 s, then on a fixed sample of 120 val episodes (recorded).
- **Rounding at readout** (`predict`): M1's rule (`wm3.effects3._chunk_h` L149–150: the V features, cumulative cold
  hours and cumulative deaths when `round_counts`) applied to the rolled-out food stock and alive (V features) and the
  cumulative deaths (the true deaths at c plus the summed predicted daily deaths) at the readout tokens, never inside the
  feedback loop; settlements alive unrounded (M1 reads it from an extra channel it does not round). Every evaluation
  writes the rounded (primary) and unrounded predictions.

usage (DGX):
  causal5.py cost --method crn|ct --dataset v2_v19|obs2_v19 --seconds 300 --budget B --out DIR
  causal5.py train --method crn|ct --dataset v2_v19|obs2_v19 --seed S --cost COST.json --budget B --max-seconds N --out RUN
  causal5.py effects --run RUN --which last|best --split v2|obs|obs2 --cases JSON --out DIR [--name NAME]
"""
from __future__ import annotations

import argparse
import copy
import faulthandler
import json
import math
import signal
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm5  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as g
from wm2 import effects as ef
from wm2 import v2
from wm4 import causal as c4
from wm4 import tokens as tk

torch = g.torch

AUTHORS_STEPS = {"crn_enc": 15_700, "crn_dec": 53_800, "ct": 11_850}
BUDGET = {"crn": 5_400, "ct": 14_400}             # GOAL.md 9.1: CRN <= 1.5 h per run (both stages), CT <= 4 h
SAFETY = 1.25
MAX_EVALS = 15
EVAL_SAMPLE_LIMIT_S = 60.0
EVAL_SAMPLE = 120
ENC_TIME_SHARE = 0.5                               # the encoder's share of the safety net (WM-4 D15)
# WM-4's grid choices (METHODS.md, GOAL.md 3.1): CRN configuration 3, CT configuration 2; no new grid.
CONFIG = {"crn": dict(c4.GRID["crn"][3]), "ct": dict(c4.GRID["ct"][2])}
OUTCOME_COL = {"food_stock": "s.food_stock", "alive": "g.alive", "settlements_alive": "g.settlements_alive"}
ROUNDED = ("food_stock", "alive", "deaths")


def alpha_at(step: int, planned: int, alpha_max: float) -> float:
    """α at optimizer step `step` (0-based) of a stage planned for `planned` steps."""
    return c4.alpha_at(min(1.0, step / max(1, planned)), alpha_max)


def eval_points(planned: int, n: int = MAX_EVALS) -> set[int]:
    """Steps after which an evaluation runs: n − 1 evenly spaced points inside the plan (the n-th is the end)."""
    return {int(round(planned * i / n)) for i in range(1, n)} - {0, planned}


def scaled_plan(cost: dict | None, method: str, budget: float) -> dict:
    """Planned steps per stage (GOAL.md 7.4): the authors' unless the cost check predicts more than the budget, then
    floor((budget − evaluation allowance) / s per step), both CRN stages scaled by one factor."""
    stages = ["crn_enc", "crn_dec"] if method == "crn" else ["ct"]
    plan = {s: AUTHORS_STEPS[s] for s in stages}
    if cost is None:
        return {"planned": plan, "reduced": False, "predicted_seconds": None, "factor": 1.0}
    allowance = sum(MAX_EVALS * cost["stages"][s]["eval_seconds"] for s in stages)
    train_s = sum(plan[s] * cost["stages"][s]["s_per_step"] for s in stages)
    pred = train_s + allowance
    if pred <= budget:
        return {"planned": plan, "reduced": False, "predicted_seconds": pred, "factor": 1.0, "allowance": allowance}
    f = max(0.0, (budget - allowance) / train_s)
    return {"planned": {s: int(math.floor(plan[s] * f)) for s in stages}, "reduced": True, "predicted_seconds": pred,
            "factor": f, "allowance": allowance}


# ------------------------------------------------------------------ data -------------------------------------------
def load_data(dataset: str, root: Path):
    """(train Data, val Data, val phases, phase mode, variant): `v2_v19` as WM-4; `obs2_v19` — the observed train / val
    episodes, phase 06:00 (GOAL.md 7.4)."""
    if dataset != "obs2_v19":
        return c4.load_data(dataset, root)
    from wm5 import m1_obs2
    m1_obs2.register()
    gr = c.load_groups(["train", "val"], root, "obs2_v19")
    variant = g.dataset_variant(c.dataset(dataset), "sev-blind_action-aware")
    return c4.Data(gr["train"], variant), c4.Data(gr["val"], variant), [tk.PHASE_06], "fixed_06", variant


class Val:
    """`val_roll` on the val set or, when one evaluation takes longer than 60 s, on a fixed sample of 120 episodes (the
    first 120 in seed order; recorded)."""

    def __init__(self, seqs: list, n_episodes: int, phases: list[int]):
        self.seqs, self.n, self.phases, self.sampled = seqs, n_episodes, phases, False
        self.jobs = c4.val_jobs(seqs)

    def __call__(self, model: dict, device) -> tuple[float, float]:
        t = time.time()
        v = c4.roll_loss(model, self.jobs, device)
        dt = time.time() - t
        if dt > EVAL_SAMPLE_LIMIT_S and not self.sampled and self.n > EVAL_SAMPLE:
            keep = [s for i, s in enumerate(self.seqs) if i % self.n < EVAL_SAMPLE]   # same episodes in every phase
            self.seqs, self.jobs, self.sampled = keep, c4.val_jobs(keep), True
        return v, dt

    def info(self) -> dict:
        return {"episodes": EVAL_SAMPLE if self.sampled else self.n, "sampled": self.sampled, "jobs": len(self.jobs)}


# ------------------------------------------------------------------ training ---------------------------------------
def train_crn(hp: dict, tr, val: Val, mode: str, norm, seed: int, device, plan: dict, cap: float, log, t0: float):
    rng = np.random.default_rng([seed, 0])
    nt = c4.norm_tensors(norm, device)
    enc = c4.CRNEncoder(hp).to(device)
    opt = torch.optim.Adam(enc.parameters(), lr=hp["enc_lr"])
    vb = c4.stack(val.seqs, device)
    P = plan["crn_enc"]
    pts, steps, evals, stop = eval_points(P), 0, 0, "planned"
    best1, best1_step = float("inf"), -1
    enc_cap = t0 + cap * ENC_TIME_SHARE
    losses = []

    def enc_eval():
        nonlocal best1, best1_step
        enc.eval()
        with torch.no_grad():
            _, opred, _ = enc(vb)
            v1 = float(((opred[:, :-1] - c4.targets(vb, nt)) ** 2).mean())
        log({"stage": "encoder", "steps": steps, "planned": P, "alpha": enc.head.alpha,
             "mse": float(np.mean([x[0] for x in losses])) if losses else None,
             "ce": float(np.mean([x[1] for x in losses])) if losses else None, "val_one_step": v1, "t": time.time() - t0})
        losses.clear()
        if v1 < best1:
            best1, best1_step = v1, steps
        enc.train()

    epochs = 0
    while steps < P:
        seqs = tr.seqs(c4.train_phases(tr, mode, rng), norm)
        enc.train()
        epochs += 1
        for idx in c4.batches(len(seqs), hp["enc_batch"], rng):
            enc.head.alpha = alpha_at(steps, P, hp["alpha"])
            bt = c4.stack([seqs[i] for i in idx], device)
            tpred, opred, _ = enc(bt)
            mse = ((opred[:, :-1] - c4.targets(bt, nt)) ** 2).mean()
            cls = c4.ce(tpred[:, :-1], c4.classes(bt["a"][:, :-1])).mean()
            loss = mse + cls
            opt.zero_grad(); loss.backward(); opt.step()
            losses.append((mse.item(), cls.item()))
            steps += 1
            if steps in pts:
                enc_eval(); evals += 1
            if steps >= P or time.time() > enc_cap:
                break
        if time.time() > enc_cap and steps < P:
            stop = "time"
            break
    enc_eval(); evals += 1
    enc_alpha = enc.head.alpha
    enc.eval()
    for p in enc.parameters():
        p.requires_grad_(False)
    info = {"encoder": {"planned": P, "steps": steps, "epochs": epochs, "stop": stop, "evals": evals,
                        "alpha_last": enc_alpha, "alpha_max": hp["alpha"], "best_one_step": best1,
                        "best_one_step_step": best1_step}}
    dec = c4.CRNDecoder(hp).to(device)
    opt = torch.optim.Adam(dec.parameters(), lr=hp["dec_lr"])
    model = {"method": "crn", "enc": enc, "dec": dec, "norm": norm, "norm_t": nt, "names": tr.names}
    P = plan["crn_dec"]
    pts, steps, evals, stop = eval_points(P), 0, 0, "planned"
    best, best_step, best_state = float("inf"), -1, None
    K = len(tr.seqs([tk.PHASE_06], norm)[0].rows)
    starts = K - 2 - c4.TAU
    last_val = None

    def dec_eval():
        nonlocal best, best_step, best_state, last_val
        v, dt = val(model, device)
        last_val = v
        log({"stage": "decoder", "steps": steps, "planned": P, "alpha": dec.head.alpha, "eval_s": dt,
             "mse": float(np.mean([x[0] for x in losses])) if losses else None,
             "ce": float(np.mean([x[1] for x in losses])) if losses else None, "val_roll": v, "t": time.time() - t0})
        losses.clear()
        if v < best:
            best, best_step, best_state = v, steps, copy.deepcopy(dec.state_dict())

    epochs = 0
    while steps < P:
        seqs = tr.seqs(c4.train_phases(tr, mode, rng), norm)
        full = c4.stack(seqs, device)
        with torch.no_grad():
            enc_br = enc.br(full)
        win = np.array([(i, k) for i in range(len(seqs)) for k in range(starts + 1)])
        win = win[rng.permutation(len(win))]
        tgt_all = c4.targets(full, nt)
        epochs += 1
        for i in range(0, len(win) - hp["dec_batch"] + 1, hp["dec_batch"]):
            dec.train()
            dec.head.alpha = alpha_at(steps, P, hp["alpha"])
            w = win[i: i + hp["dec_batch"]]
            si, ki = torch.tensor(w[:, 0], device=device), torch.tensor(w[:, 1], device=device)
            pos = ki[:, None] + 1 + torch.arange(c4.TAU, device=device)[None]
            init = enc_br[si, ki]
            tpred, opred, _ = dec(full["a"][si[:, None], pos - 1], full["y"][si[:, None], pos],
                                  full["a"][si[:, None], pos], init)
            mse = ((opred - tgt_all[si[:, None], pos]) ** 2).mean()
            cls = c4.ce(tpred, c4.classes(full["a"][si[:, None], pos])).mean()
            loss = mse + cls
            opt.zero_grad(); loss.backward(); opt.step()
            losses.append((mse.item(), cls.item()))
            steps += 1
            if steps in pts:
                dec_eval(); evals += 1
            if steps >= P or time.time() - t0 > cap:
                break
        if time.time() - t0 > cap and steps < P:
            stop = "time"
            break
    dec_eval(); evals += 1
    last_state = copy.deepcopy(dec.state_dict())
    info["decoder"] = {"planned": P, "steps": steps, "epochs": epochs, "stop": stop, "evals": evals,
                       "alpha_last": dec.head.alpha, "alpha_max": hp["alpha"], "val_roll_last": last_val,
                       "best_val_roll": best, "best_step": best_step,
                       "alpha_best": alpha_at(max(0, best_step - 1), P, hp["alpha"])}
    best_model = {"enc": copy.deepcopy(enc.state_dict()), "dec": best_state}
    last_model = {"enc": copy.deepcopy(enc.state_dict()), "dec": last_state}
    return model, last_model, best_model, info


def train_ct(hp: dict, tr, val: Val, mode: str, norm, seed: int, device, plan: dict, cap: float, log, t0: float):
    rng = np.random.default_rng([seed, 0])
    nt = c4.norm_tensors(norm, device)
    net = c4.CT(hp).to(device)
    tnames = ("head.linear2", "head.linear3")
    t_params = [p for n, p in net.named_parameters() if n.startswith(tnames)]
    o_params = [p for n, p in net.named_parameters() if not n.startswith(tnames)]
    opt0 = torch.optim.Adam(o_params, lr=hp["lr"])
    opt1 = torch.optim.Adam(t_params, lr=hp["lr"])
    ema_t, ema_o = c4.EMA(t_params, hp["beta"]), c4.EMA(o_params, hp["beta"])
    model = {"method": "ct", "net": net, "norm": norm, "norm_t": nt, "names": tr.names}
    P = plan["ct"]
    pts, steps, evals, stop = eval_points(P), 0, 0, "planned"
    best, best_step, best_state, last_val = float("inf"), -1, None, None
    losses = []
    alpha = alpha_at(0, P, hp["alpha"])

    def ct_eval():
        nonlocal best, best_step, best_state, last_val
        ema_t.swap(); ema_o.swap()
        v, dt = val(model, device)
        state = copy.deepcopy(net.state_dict())          # the EMA weights
        ema_t.swap(); ema_o.swap()
        last_val = v
        log({"stage": "ct", "steps": steps, "planned": P, "alpha": alpha, "eval_s": dt,
             "mse": float(np.mean([x[0] for x in losses])) if losses else None,
             "conf": float(np.mean([x[1] for x in losses])) if losses else None,
             "cls": float(np.mean([x[2] for x in losses])) if losses else None, "val_roll": v, "t": time.time() - t0})
        losses.clear()
        if v < best:
            best, best_step, best_state = v, steps, state
        return state

    epochs, last_state = 0, None
    while steps < P:
        seqs = tr.seqs(c4.train_phases(tr, mode, rng), norm)
        epochs += 1
        for idx in c4.batches(len(seqs), hp["batch"], rng):
            net.train()
            alpha = alpha_at(steps, P, hp["alpha"])
            bt = c4.stack([seqs[i] for i in idx], device)
            K = bt["y"].shape[1]
            split = torch.cat([torch.full((len(idx),), K, device=device),
                               torch.tensor(rng.integers(0, K + 1, len(idx)), device=device)])
            bt2 = {k: torch.cat([v, v], 0) for k, v in bt.items()}
            tgt, cls_t = c4.targets(bt2, nt), c4.classes(bt2["a"][:, :-1])
            tpred, opred, _ = net(bt2, split)
            mse = ((opred[:, :-1] - tgt) ** 2).mean()
            uniform = torch.full_like(tpred[:, :-1], 1.0 / tk.N_CLASSES)
            conf = alpha * c4.ce(tpred[:, :-1], uniform).mean()
            opt0.zero_grad(); opt1.zero_grad(); (mse + conf).backward(); opt0.step(); ema_o.update()
            tpred, _, _ = net(bt2, split, detach_treatment=True)
            loss1 = alpha * c4.ce(tpred[:, :-1], cls_t).mean()
            opt0.zero_grad(); opt1.zero_grad(); loss1.backward(); opt1.step(); ema_t.update()
            losses.append((mse.item(), conf.item(), loss1.item()))
            steps += 1
            if steps in pts:
                ct_eval(); evals += 1
            if steps >= P or time.time() - t0 > cap:
                break
        if time.time() - t0 > cap and steps < P:
            stop = "time"
            break
    last_state = ct_eval(); evals += 1
    info = {"ct": {"planned": P, "steps": steps, "epochs": epochs, "stop": stop, "evals": evals, "alpha_last": alpha,
                   "alpha_max": hp["alpha"], "val_roll_last": last_val, "best_val_roll": best, "best_step": best_step,
                   "alpha_best": alpha_at(max(0, best_step - 1), P, hp["alpha"])}}
    net.load_state_dict(last_state)
    return model, last_state, best_state, info


def _phases_note(mode: str, vphases: list[int]) -> dict:
    return {"phase_mode": mode, "val_phases": vphases}


def setup(method: str, dataset: str, seed: int, root: Path, device_name: str):
    device = g.pick_device(device_name)
    torch.manual_seed(seed)
    tr, va, vphases, mode, variant = load_data(dataset, root)
    norm = tk.Norm.fit(tr.raws)
    val = Val(c4.val_seqs(va, vphases, norm), len(va.raws), vphases)
    hp = dict(c4.DEFAULTS[method]) | CONFIG[method]
    return device, tr, va, vphases, mode, variant, norm, val, hp


def run_train(a) -> int:
    t0 = time.time()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    device, tr, va, vphases, mode, variant, norm, val, hp = setup(a.method, a.dataset, a.seed, Path(a.root), a.device)
    cost = json.loads(Path(a.cost).read_text()) if a.cost else None
    budget = min(float(a.budget), float(a.max_seconds))
    plan = scaled_plan(cost, a.method, float(a.budget))
    cap = SAFETY * budget
    logf = (out / "train_log.jsonl").open("w")

    def log(rec):
        logf.write(json.dumps(rec) + "\n"); logf.flush()
        print(json.dumps(rec), flush=True)

    cfg = {"method": a.method, "dataset": a.dataset, "seed": a.seed, "hyper": hp, "config_overrides": CONFIG[a.method],
           "budget": float(a.budget), "max_seconds": float(a.max_seconds), "safety_cap_seconds": cap,
           "plan": plan, "authors_steps": AUTHORS_STEPS, "cost_file": a.cost, "cost": cost,
           "norm": norm.to_json(), "state_names": list(variant.state_names),
           "input_names": list(variant.input_names()), "train_episodes": len(tr.raws), "val_episodes": len(va.raws),
           "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(a.dataset), "device": device,
           "torch": torch.__version__, "numpy": np.__version__, "argv": sys.argv} | _phases_note(mode, vphases)
    c.write_json(out / "config.json", cfg)
    fn = train_ct if a.method == "ct" else train_crn
    model, last, best, info = fn(hp, tr, val, mode, norm, a.seed, device, plan["planned"], cap, log, t0)
    logf.close()
    torch.save(last, out / "model.pt")
    torch.save(best, out / "model_best.pt")
    info["val"] = val.info()
    info["seconds"] = time.time() - t0
    stages = [info[s] for s in (["encoder", "decoder"] if a.method == "crn" else ["ct"])]
    info["reached"] = {"steps_fraction_min": min(s["steps"] / max(1, s["planned"]) for s in stages),
                       "alpha_fraction_min": min(s["alpha_last"] / s["alpha_max"] for s in stages),
                       "stop": [s["stop"] for s in stages], "planned_equal_authors": not plan["reduced"]}
    info["val_roll"] = stages[-1]["val_roll_last"]
    cfg["result"] = info
    c.write_json(out / "config.json", cfg)
    c.write_json(out / "val_metrics.json", info)
    print(f"done {a.method} {a.dataset} s{a.seed}: reached {info['reached']} {info['seconds']:.0f}s", flush=True)
    return 0


def run_cost(a) -> int:
    """The 300-s cost check (GOAL.md 9.1) on the actual dataset (train and val only): s per step of each stage (CRN: half
    the seconds each), one evaluation's wall time, the peak GPU memory, and the plan it implies for the budget."""
    t0 = time.time()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    device, tr, va, vphases, mode, variant, norm, val, hp = setup(a.method, a.dataset, 0, Path(a.root), a.device)
    t_load = time.time() - t0
    if device.startswith("cuda"):
        torch.cuda.reset_peak_memory_stats()
    recs = []

    def log(rec):
        recs.append(rec); print(json.dumps(rec), flush=True)

    seconds = float(a.seconds)
    t1 = time.time()
    fn = train_ct if a.method == "ct" else train_crn
    # a plan the time cannot finish: the stage stops on its cap (CRN: half of `seconds` for the encoder)
    plan = {s: AUTHORS_STEPS[s] for s in (["crn_enc", "crn_dec"] if a.method == "crn" else ["ct"])}
    _, _, _, info = fn(hp, tr, val, mode, norm, 0, device, plan, seconds, log, t1)
    stages = {}
    for key, stage in ((("crn_enc", "encoder"), ("crn_dec", "decoder")) if a.method == "crn" else (("ct", "ct"),)):
        rs = [r for r in recs if r["stage"] == stage]
        t_first, t_last = rs[0]["t"], rs[-1]["t"]
        ev = [r.get("eval_s", 0.0) for r in rs]
        n = info[stage]["steps"]
        # wall time of the stage's optimizer steps: its span minus the evaluations inside it
        start = 0.0 if stage in ("encoder", "ct") else max(r["t"] for r in recs if r["stage"] == "encoder")
        span = t_last - start - sum(ev)
        stages[key] = {"steps": n, "seconds": span, "s_per_step": span / max(1, n),
                       "eval_seconds": float(np.max(ev)) if any(ev) else 0.05, "evals": len(rs)}
    cost = {"method": a.method, "dataset": a.dataset, "seconds": seconds, "load_seconds": t_load, "stages": stages,
            "peak_gpu_mib": (torch.cuda.max_memory_allocated() / 2 ** 20) if device.startswith("cuda") else None,
            "device": device, "val": val.info(), "code_commit": c.git_commit(), "argv": sys.argv,
            "budget": float(a.budget), "train_episodes": len(tr.raws), "val_episodes": len(va.raws),
            "wall_seconds": time.time() - t0}
    cost["plan"] = scaled_plan(cost, a.method, float(a.budget))
    c.write_json(out / "cost.json", cost)
    print(json.dumps({k: v for k, v in cost.items() if k != "argv"}), flush=True)
    return 0


# ------------------------------------------------------------------ loading / prediction ---------------------------
def load(run: Path, device, which: str = "last") -> tuple[dict, dict]:
    """A WM-5 run's primary (`last`: model.pt) or secondary (`best`: model_best.pt) checkpoint; a WM-4 run (no
    `plan` in config.json) loads its kept model.pt whichever is asked."""
    cfg = json.loads((Path(run) / "config.json").read_text())
    if "plan" not in cfg:
        return c4.load(Path(run), device)
    hp, norm = cfg["hyper"], tk.Norm.from_json(cfg["norm"])
    sd = torch.load(Path(run) / ("model.pt" if which == "last" else "model_best.pt"), map_location=device)
    if cfg["method"] == "ct":
        net = c4.CT(hp).to(device); net.load_state_dict(sd); net.eval()
        m = {"method": "ct", "net": net}
    else:
        enc, dec = c4.CRNEncoder(hp).to(device), c4.CRNDecoder(hp).to(device)
        enc.load_state_dict(sd["enc"]); dec.load_state_dict(sd["dec"]); enc.eval(); dec.eval()
        m = {"method": "crn", "enc": enc, "dec": dec}
    m |= {"norm": norm, "norm_t": c4.norm_tensors(norm, device), "names": tuple(cfg["state_names"])}
    return m, cfg


def readout(ar: np.ndarray, fa: np.ndarray, deaths_c: float, names: tuple, truth: dict, rounded: bool) -> dict:
    """Effects at the case's lags from the arm / none rollouts (raw daily states of tokens k + 1 …): levels at token
    n − 1 (n = lag / 24) and cumulative deaths = deaths at c + the summed predicted daily deaths; `rounded` applies M1's
    readout rounding to food stock, alive and cumulative deaths (never to settlements alive)."""
    didx = [names.index(n) for n in tk.DEATHS]
    rnd = np.round if rounded else (lambda x: x)
    eff = {}
    for o in v2.OUTCOMES:
        eff[o] = {}
        for lag in truth[o]:
            n = v2.LAGS[lag] // tk.TOKEN_ROWS
            if o == "deaths":
                eff[o][lag] = float(rnd(deaths_c + ar[:n, didx].sum()) - rnd(deaths_c + fa[:n, didx].sum()))
            else:
                j = names.index(OUTCOME_COL[o])
                f = rnd if o in ROUNDED else (lambda x: x)
                eff[o][lag] = float(f(ar[n - 1, j]) - f(fa[n - 1, j]))
    return eff


def predict(model: dict, fams: list, cases: list, device) -> tuple[dict, dict]:
    """(rounded, unrounded) predictions per case key: rollout with the arm at the cast token (none after) − rollout with
    none, both from the case's `pre` member history (`fams`: `wm3.effects3.FamilyData`, `factual[rep]` = that member) at
    the cast row's phase (WM-4's prediction contract, D13, plus rounding)."""
    names = model["names"]
    by = {f.seed: f for f in fams}
    jobs = []
    for x in cases:
        ep = by[x.seed].factual[x.rep]
        if tuple(ep.names) != tuple(names):
            raise ValueError(f"{x.key}: state columns differ from the model's")
        raw = tk.raw_of(ep, d.variant_for("sev-blind_action-aware", ep.names))
        seq = tk.tokenize(raw, tk.phase_of_row(x.cast_row), model["norm"])
        k = tk.token_of_row(x.cast_row)
        assert seq.rows[k] == x.cast_row
        kind, amount = v2.ARM_SPEC[x.arm]
        arm_plan = np.vstack([tk.arm_treatment(kind, amount)] + [tk.NONE] * (c4.HORIZON - 1))
        jobs += [c4.Job(seq, k, arm_plan), c4.Job(seq, k, np.vstack([tk.NONE] * c4.HORIZON))]
    ros = c4.rollout(model, jobs, device)
    rounded, raw_out = {}, {}
    for i, x in enumerate(cases):
        ar, fa = ros[2 * i], ros[2 * i + 1]
        dc = float(by[x.seed].truth[x.rep]["deaths"][x.cast_row])
        for out, rd in ((rounded, True), (raw_out, False)):
            out[x.key] = {"effects": readout(ar, fa, dc, names, x.truth, rd), "shift": 0.0, "shift_predicted": False}
    return rounded, raw_out


def cmd_effects(a) -> int:
    """One run's effects on a split's cases (test, once per run / checkpoint): `obs2` — the r0 cases of the obs2 test
    families (`effects5.load_obs2_cases`); `obs` — WM-4's obs test cases (`obs_base.json`); `v2` — WM-2's test cases
    (`effects_base.json`). Rounded (primary) and unrounded predictions."""
    from wm3 import effects3 as e3
    from wm4 import effects4 as e4
    from wm4 import m1_obs
    from wm5 import effects5 as e5
    from wm5 import m1_obs2
    m1_obs.register(); m1_obs2.register()
    t0 = time.time()
    name = a.name or f"effects_{a.which}"
    target = Path(a.out) / f"{name}.json"
    if target.exists():
        raise SystemExit(f"{target} exists: computed once per run")
    device = g.pick_device(a.device)
    model, cfg = load(Path(a.run), device, a.which)
    root = Path(a.root)
    if a.split == "obs2":
        cases, fams = e5.obs2_eval_inputs(root, a.cases, "test", replicate=0)
    elif a.split == "obs":
        _, cases, _, _ = e4.load_obs_base(Path(a.cases))
        fams = e3.load_families(e4.family_dirs(root, "test"), "test")
    else:
        cases = [ef.case_from(x) for x in json.loads(Path(a.cases).read_text())["cases"]["test"]]
        fams = e3.load_families(ef.family_dirs(root, "test"), "test")
    rounded, unrounded = predict(model, fams, cases, device)
    out = {"run": a.run, "which": a.which, "method": cfg["method"], "dataset": cfg["dataset"], "seed": cfg["seed"],
           "split": a.split, "code_commit": c.git_commit(), "train_code_commit": cfg.get("code_commit"),
           "argv": sys.argv, "predictions": rounded, "predictions_unrounded": unrounded, "rounded": True,
           "cases_sha256": ef._sha(Path(a.cases)), "n_cases": len(cases), "seconds": time.time() - t0}
    if target.exists():
        raise SystemExit(f"{target} exists")
    c.write_json(target, out)
    print(f"effects {cfg['method']} {a.split} {a.which} done {time.time() - t0:.0f}s")
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("cost")
    p.add_argument("--method", required=True, choices=["crn", "ct"])
    p.add_argument("--dataset", required=True, choices=["v2_v19", "obs2_v19"])
    p.add_argument("--seconds", type=float, default=300)
    p.add_argument("--budget", type=float, required=True)
    p = sub.add_parser("train")
    p.add_argument("--method", required=True, choices=["crn", "ct"])
    p.add_argument("--dataset", required=True, choices=["v2_v19", "obs2_v19"])
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--cost", default=None, help="the kind's cost.json (its plan; omitted: the authors' steps)")
    p.add_argument("--budget", type=float, required=True)
    p.add_argument("--max-seconds", dest="max_seconds", type=float, required=True)
    p = sub.add_parser("effects")
    p.add_argument("--run", required=True)
    p.add_argument("--which", default="last", choices=["last", "best"])
    p.add_argument("--split", required=True, choices=["v2", "obs", "obs2"])
    p.add_argument("--cases", required=True, help="effects_base.json (v2), obs_base.json (obs), obs2_base.json (obs2)")
    p.add_argument("--name", default=None)
    for p in sub.choices.values():
        p.add_argument("--root", default=str(c.DEFAULT_ROOT))
        p.add_argument("--device", default="auto")
        p.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    if torch is None:
        raise SystemExit("torch is required")
    faulthandler.register(signal.SIGUSR1, file=sys.stderr, all_threads=True)
    return {"cost": run_cost, "train": run_train, "effects": cmd_effects}[a.cmd](a)


if __name__ == "__main__":
    raise SystemExit(main())
