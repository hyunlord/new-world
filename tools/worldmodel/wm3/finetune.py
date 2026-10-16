"""WM-3 fine-tuning loop (docs/worldmodel/wm3/GOAL.md 7.4, 7.5): continue a WM-2 M1b-act run as an `M1TFH` on plain
and cast windows with WM-2's losses plus the pair and hazard losses, selecting checkpoints by their effect error on
validation. `m1_gru.train` is not called or changed.

- Init: the init run's `config.json` and `best.pt` (strict load except the new hazard layer, seeded by `--seed`); the
  normalizer is refit on the same train groups and must equal the init's stored one to 1e-12.
- Batches (`wm3.sampling`): `--n-plain` plain windows (`default_rng([seed, 0])`), `--n-cast` cast windows
  (`default_rng([seed, 1])`) and, with `--partners 1`, their partners; rollout positions of the plain windows from
  `default_rng([seed, 2])` (WM-2's `roll_positions`). AdamW (`--lr`, weight decay and grad clip as the init), rollout
  horizon fixed at k = 168. Every other hyper-parameter is the init run's.
- Evaluation every `--eval-every` steps (and at step 0, the init): `val_t2` = WM-2's validation T2 24 h
  (`m1_gru.val_score`, batched 512) and `val_eff` = the 24-cell mean scaled effect MAE on the val cases of replicate 0
  (`wm3.effects3.val_eff`, 720-row rollouts), both with `round_counts = False`. Eligible: `val_t2` <= 1.05 x the
  step-0 `val_t2`; the kept checkpoint (`best.pt`) is the eligible one with the lowest `val_eff` (the init included);
  patience counts evaluations without a new best eligible `val_eff`. The step-0 `val_t2` must equal the init's
  `best_val_t2_24` within 1e-3 (else stop).
- End: the output rounding of counts by `m1_gru.choose_rounding` on the kept checkpoint (val, batched 512);
  `config.json` (model `m1_tfh`), `train_log.jsonl`, `val_metrics.json`.

usage (DGX): python3 tools/worldmodel/wm3/finetune.py --init RUN --out DIR --config full|ctrl|os --seed S
             --n-plain 16 --n-cast 16 --partners 1 --pair 1.0 --hazard 10.0 --lr 1e-4 --max-steps 4000 --eval-every 250
             --patience 6 --max-seconds N --base effects_base.json
"""
from __future__ import annotations

import argparse
import faulthandler
import hashlib
import json
import signal
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm3  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm1 import m1_gru as g
from wm1 import m1_tf
from wm2 import effects as ef
from wm2 import v2
from wm3 import effects3 as e3
from wm3 import losses as ls
from wm3 import model as wm
from wm3 import sampling as sp

torch = g.torch
K = 168
ELIGIBLE = 1.05
T2_TOL = 1e-3
NORM_TOL = 1e-12
# The init's hyper-parameters a fine-tuning run keeps (everything of `m1_tf.DEFAULTS` the loop reads).
KEEP = ("d_model", "layers", "heads", "context", "warmup", "window", "weight_decay", "grad_clip", "threads",
        "roll_min_context", "roll_starts", "roll_weight", "tf_l1", "clamp_nonneg", "v_weight", "val_batch", "k_max")


def sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def norm_close(a: d.Normalizer, b: dict, tol: float = NORM_TOL) -> bool:
    """The refit normalizer equals the stored one (names exactly, every array within `tol`)."""
    bb = d.Normalizer.from_json(b)
    if a.names != bb.names or a.continuous != bb.continuous or a.fit_rows != bb.fit_rows:
        return False
    return all(np.allclose(getattr(a, k), getattr(bb, k), rtol=0, atol=tol) for k in ("mean", "std", "dstd", "lo", "hi"))


def val_cases(base: dict) -> list:
    """The `val_eff` cases: the val cases of replicate 0 of the W3 arms (`effects_base.json`)."""
    return [ef.case_from(j) for j in base["cases"]["val"] if j["rep"] == 0 and j["arm"] in v2.W3_ARMS]


EVAL_STACK_DUMP = 600     # seconds: an evaluation slower than this dumps every thread's stack to stderr (train.out)


def evaluate(pred, val: dict, stds_t2: dict, fams: list, cases: list, stds: dict, vb: int, step: int = 0) -> dict:
    pred.round_counts = False
    t = time.time()
    print(f"eval {step} start", flush=True)
    faulthandler.dump_traceback_later(EVAL_STACK_DUMP, repeat=True)
    t2, per = g.val_score(pred, val, stds_t2, None, vb)
    t_t2 = time.time() - t
    print(f"eval {step} val_t2 {t2:.4f} {t_t2:.1f}s", flush=True)
    eff, cells = e3.val_eff(pred, fams, cases, stds)
    faulthandler.cancel_dump_traceback_later()
    return {"val_t2": t2, "val_t2_per": per, "val_eff": eff, "val_eff_cells": cells, "t2_s": t_t2,
            "eff_s": time.time() - t - t_t2}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--init", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--config", required=True, help="WM-3 configuration name (full, ctrl, os)")
    ap.add_argument("--seed", type=int, required=True)
    ap.add_argument("--n-plain", type=int, required=True)
    ap.add_argument("--n-cast", type=int, required=True)
    ap.add_argument("--partners", type=int, default=0)
    ap.add_argument("--pair", type=float, default=0.0)
    ap.add_argument("--hazard", type=float, default=0.0)
    ap.add_argument("--lr", type=float, default=1e-4)
    ap.add_argument("--max-steps", type=int, default=4000)
    ap.add_argument("--eval-every", type=int, default=250)
    ap.add_argument("--patience", type=int, default=6)
    ap.add_argument("--max-seconds", type=float, required=True)
    ap.add_argument("--base", required=True, help="effects_base.json (the val cases and the effect scales)")
    ap.add_argument("--root", default=str(c.DEFAULT_ROOT))
    ap.add_argument("--device", default="auto")
    ap.add_argument("--log-every", type=int, default=50)
    a = ap.parse_args(argv)
    t0 = time.time()                      # the clock starts before data loading (as WM-2)
    faulthandler.register(signal.SIGUSR1, all_threads=True)   # `kill -USR1 PID` prints every thread's stack
    if torch is None:
        raise SystemExit("torch is required")
    if a.pair and not a.partners:
        raise SystemExit("the pair loss needs --partners 1")
    init = Path(a.init)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    icfg = json.loads((init / "config.json").read_text())
    ival = json.loads((init / "val_metrics.json").read_text())
    if icfg["model"] != "m1_tf" or icfg["dataset"] != "v2_v19":
        raise SystemExit(f"{init}: not a WM-2 M1b-act run")
    ds = c.dataset("v2_v19")
    variant = g.dataset_variant(ds, icfg["variant"])
    if list(variant.input_names()) != icfg["input_names"]:
        raise SystemExit("input list differs from the init's")
    cfg = {k: icfg["hyper"][k] for k in KEEP}
    cfg.update({"lr": a.lr, "max_steps": a.max_steps, "eval_every": a.eval_every, "patience": a.patience,
                "max_seconds": a.max_seconds, "n_plain": a.n_plain, "n_cast": a.n_cast, "partners": int(a.partners),
                "pair": a.pair, "hazard": a.hazard, "k": K, "haz_seed": a.seed, "eligible_factor": ELIGIBLE,
                "val_eff_horizon": e3.VAL_EFF_HORIZON, "cast_pos": sp.CAST_POS, "batch": 0,
                "plain_stream": [a.seed, 0], "cast_stream": [a.seed, 1], "roll_stream": [a.seed, 2]})
    device = g.pick_device(a.device)
    g.set_seed(a.seed, cfg["threads"])
    groups = c.load_groups(["train", "val"], Path(a.root), ds.name)
    train_eps, val = groups["train"], {"val": groups["val"]}
    if any(ep.names != variant.state_names for ep in train_eps + groups["val"]):
        raise SystemExit("an episode's feature table differs from the dataset's column list")
    stds_t2 = c.split_stds(val)
    norm = d.Normalizer.fit(train_eps, variant)
    if not norm_close(norm, icfg["normalizer"]):
        raise SystemExit("the refit normalizer differs from the init's stored normalizer (> 1e-12): stop")
    lay = d.Layout(variant)
    preps = [d.prepare(ep, variant, norm) for ep in train_eps]
    cases = sp.train_cases(train_eps)
    sampler = sp.Sampler(preps, cases, a.seed, cfg["window"], a.n_plain, a.n_cast, bool(a.partners))
    cfg["batch"] = sampler.batch
    base = json.loads(Path(a.base).read_text())
    stds = ef.stds_from(base["stds"])
    vcases = val_cases(base)
    fams = e3.load_families([p for p in ef.family_dirs(Path(a.root), "val")], "val")
    print(f"data: train {len(train_eps)} episodes, {len(cases)} cases; val {len(groups['val'])} episodes, "
          f"{len(vcases)} val_eff cases, {len(fams)} families; {time.time() - t0:.0f}s", flush=True)
    model = wm.build(lay, cfg)
    state = torch.load(init / "best.pt", map_location="cpu")
    wm.load_init(model, state)
    model = model.to(device)
    opt = torch.optim.AdamW(model.parameters(), lr=a.lr, weight_decay=cfg["weight_decay"])
    stepper = g.Stepper(lay, norm, device, bool(cfg["clamp_nonneg"]))
    pred = m1_tf._predictor(model, lay, norm, variant, device, cfg)
    rng_roll = np.random.default_rng([a.seed, 2])
    vb = int(cfg["val_batch"])
    config = {"model": wm.KIND, "dataset": ds.name, "variant": variant.name, "wm3_config": a.config, "seed": a.seed,
              "hyper": cfg, "device": device, "torch": torch.__version__, "numpy": np.__version__,
              "python": sys.version.split()[0], "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(ds.name),
              "init_run": str(init), "init_checkpoint_sha256": sha256(init / "best.pt"),
              "init_best_val_t2_24": ival["best_val_t2_24"], "init_round_counts": icfg.get("round_counts"),
              "base_sha256": sha256(Path(a.base)), "train_groups": ["train"], "val_groups": ["val"],
              "train_episodes": len(train_eps), "train_cases": len(cases), "val_episodes": len(groups["val"]),
              "val_eff_cases": len(vcases), "input_names": list(variant.input_names()), "normalizer": norm.to_json(),
              "data_root": str(a.root), "threads": torch.get_num_threads(), "argv": sys.argv}
    c.write_json(out / "config.json", config)
    log = (out / "train_log.jsonl").open("w")
    ev0 = evaluate(pred, val, stds_t2, fams, vcases, stds, vb, 0)
    t2_ref = ev0["val_t2"]
    log.write(json.dumps({"step": 0, "eval": True, **ev0, "eligible": True, "best": ev0["val_eff"],
                          "t": time.time() - t0}) + "\n")
    log.flush()
    print(f"step 0 val_t2 {ev0['val_t2']:.4f} (init {ival['best_val_t2_24']:.4f}) val_eff {ev0['val_eff']:.4f} "
          f"eval {ev0['t2_s']:.1f}+{ev0['eff_s']:.1f}s {time.time() - t0:.0f}s", flush=True)
    if abs(t2_ref - ival["best_val_t2_24"]) > T2_TOL:
        config["stop"] = "init_val_t2_mismatch"
        c.write_json(out / "config.json", config)
        raise SystemExit(f"step-0 val_t2 {t2_ref} differs from the init's {ival['best_val_t2_24']} by > {T2_TOL}: stop")
    torch.save(model.state_dict(), out / "best.pt")
    best, best_step, bad, step, stop = ev0["val_eff"], 0, 0, 0, "max_steps"
    trajectory = [{"step": 0, "val_t2": ev0["val_t2"], "val_eff": ev0["val_eff"], "eligible": True}]
    for step in range(1, a.max_steps + 1):
        model.train()
        win, _ = sampler.draw()
        bt = g.batch_tensors(win, device)
        tf, parts, roll, pair, haz = ls.wm3_losses(model, stepper, bt, K, cfg, rng_roll, a.n_plain, a.n_cast,
                                                   bool(a.partners))
        loss = tf + cfg["roll_weight"] * roll + a.pair * pair + a.hazard * haz
        opt.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), cfg["grad_clip"])
        opt.step()
        if step % a.log_every == 0:
            log.write(json.dumps({"step": step, "k": K, "loss": loss.item(), **parts, "t": time.time() - t0}) + "\n")
            log.flush()
        if step % a.eval_every == 0:
            ev = evaluate(pred, val, stds_t2, fams, vcases, stds, vb, step)
            eligible = ev["val_t2"] <= ELIGIBLE * t2_ref
            if eligible and ev["val_eff"] < best - 1e-6:
                best, best_step, bad = ev["val_eff"], step, 0
                torch.save(model.state_dict(), out / "best.pt")
            else:
                bad += 1
            trajectory.append({"step": step, "val_t2": ev["val_t2"], "val_eff": ev["val_eff"], "eligible": eligible})
            log.write(json.dumps({"step": step, "eval": True, **ev, "eligible": eligible, "best": best,
                                  "best_step": best_step, "k": K, "t": time.time() - t0}) + "\n")
            log.flush()
            print(f"step {step} loss {loss.item():.4f} val_t2 {ev['val_t2']:.4f} val_eff {ev['val_eff']:.4f} "
                  f"{'eligible' if eligible else 'not eligible'} best {best:.4f}@{best_step} {time.time() - t0:.0f}s",
                  flush=True)
            if bad >= a.patience:
                stop = "patience"
                break
        if time.time() - t0 > a.max_seconds:
            stop = "time"
            break
    log.close()
    model.load_state_dict(torch.load(out / "best.pt", map_location=device))
    round_counts, choice = g.choose_rounding(pred, val, stds_t2, None, vb)
    selected = "init" if best_step == 0 else f"step {best_step}"
    summary = {"best_val_eff": best, "best_step": best_step, "last_step": step, "stop": stop, "selected": selected,
               "seconds": time.time() - t0, "round_counts": round_counts, "rounding_val": choice,
               "init_val_t2": t2_ref, "init_val_eff": ev0["val_eff"], "eligible_threshold": ELIGIBLE * t2_ref,
               "not_converged": best_step >= 0.9 * step and stop != "patience", "trajectory": trajectory}
    c.write_json(out / "val_metrics.json", summary)
    config.update({"round_counts": round_counts, "best_step": best_step, "stop": stop, "selected": selected})
    c.write_json(out / "config.json", config)
    print(f"done: best val_eff {best:.4f} at {best_step} ({selected}), stop {stop}, round {round_counts}, "
          f"{time.time() - t0:.0f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
