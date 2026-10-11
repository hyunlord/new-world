"""M3: text world model (GOAL.md 11) — LoRA on a small Qwen checkpoint, one day per JSON line.

Each one-based day D's 06:00 row becomes a compact JSON line: the eight variables V (rounded; fire 0/1), the day,
the season and the interventions requested between that 06:00 and the next (none in A_obs). The task: given day
D's line, generate day D+1's line. LoRA (rank 16) on A train days, one epoch or at most 2 hours, whichever comes
first; evaluated on A test at the bench's T2 starts (06:00 of days 3, 10, 17, ...) at day granularity: JSON parse
rate and nMAE of V at 24 h (bench denominator) against daily naive (= persistence at whole days).

usage:
  python3 tools/worldmodel/wm1/m3_text.py train --out RUN_DIR [--base PATH]
  python3 tools/worldmodel/wm1/m3_text.py final --run RUN_DIR
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1 import common as c
from wm1 import evaluate as ev

KEYS = ("food", "wood", "fire", "cold", "alive", "dead", "bush", "work")
DEFAULT_BASE = ("/home/hyunlord/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/"
                "7ae557604adf67be50417f59c2c2f167def9a775")
DEFAULTS = dict(rank=16, alpha=32, lr=2e-4, batch=16, max_seconds=7200, threads=16, max_new_tokens=96, epochs=1)


def day_line(ep, day: int) -> str:
    """JSON line of one-based day `day` at 06:00."""
    r = ev.row_of(day, 360)
    v = ep.bench.v[r]
    rec = {"day": day, "season": int(ep.bench.season[r])}
    rec.update({k: int(round(float(x))) for k, x in zip(KEYS, v)})
    nxt = ev.row_of(day + 1, 360)
    acts = ep.bench.actions[r:nxt]
    rec["iv"] = [[b.KINDS[k], int(acts[i, k])] for i, k in zip(*np.nonzero(acts))]
    return json.dumps(rec, separators=(",", ":"))


def days_of(ep) -> list[int]:
    """Days D whose 06:00 rows of D and D+1 both exist (D >= 2: day 1 06:00 precedes tick 0)."""
    out, day = [], 2
    while ev.row_of(day + 1, 360) < ep.rows:
        out.append(day)
        day += 1
    return out


def parse(text: str) -> np.ndarray | None:
    try:
        rec = json.loads(text.strip().splitlines()[0])
        return np.array([float(rec[k]) for k in KEYS])
    except (ValueError, KeyError, IndexError, TypeError):
        return None


def train(args) -> int:
    import torch
    from peft import LoraConfig, get_peft_model
    from transformers import AutoModelForCausalLM, AutoTokenizer
    cfg = dict(DEFAULTS)
    cfg.update({k: v for k, v in vars(args).items() if k in DEFAULTS and v is not None})
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    torch.manual_seed(args.seed)
    np.random.seed(args.seed)
    torch.set_num_threads(cfg["threads"])
    t0 = time.time()
    g = c.load_groups(["A_train"], Path(args.root))
    samples = [(day_line(ep, D), day_line(ep, D + 1)) for ep in g["A_train"] for D in days_of(ep)]
    tok = AutoTokenizer.from_pretrained(args.base)
    tok.padding_side = "right"   # the prompt-length label mask assumes right padding
    model = AutoModelForCausalLM.from_pretrained(args.base, torch_dtype=torch.float32)
    lcfg = LoraConfig(r=cfg["rank"], lora_alpha=cfg["alpha"], lora_dropout=0.0, task_type="CAUSAL_LM",
                      target_modules=["q_proj", "k_proj", "v_proj", "o_proj"])
    model = get_peft_model(model, lcfg)
    opt = torch.optim.AdamW([p for p in model.parameters() if p.requires_grad], lr=cfg["lr"])
    rng = np.random.default_rng(args.seed)
    order = rng.permutation(len(samples))
    steps = len(order) // cfg["batch"]
    config = {"model": "m3_text", "base": args.base, "seed": args.seed, "hyper": cfg, "device": "cpu",
              "torch": torch.__version__, "code_commit": c.git_commit(), "manifest_sha256": c.manifest_sha(),
              "train_samples": len(samples), "steps_per_epoch": steps, "example": samples[0]}
    c.write_json(out / "config.json", config)
    log = (out / "train_log.jsonl").open("w")
    model.train()
    done, stop = 0, "epoch"
    for step in range(steps):
        idx = order[step * cfg["batch"]:(step + 1) * cfg["batch"]]
        prompts = [samples[i][0] + "\n" for i in idx]
        full = [samples[i][0] + "\n" + samples[i][1] + tok.eos_token for i in idx]
        enc = tok(full, return_tensors="pt", padding=True)
        labels = enc["input_ids"].clone()
        for j, p in enumerate(prompts):
            labels[j, : len(tok(p)["input_ids"])] = -100
        labels[enc["attention_mask"] == 0] = -100
        loss = model(**enc, labels=labels).loss
        opt.zero_grad()
        loss.backward()
        opt.step()
        done = step + 1
        if step % 5 == 0:
            log.write(json.dumps({"step": step, "loss": loss.item(), "t": time.time() - t0}) + "\n")
            log.flush()
            print(f"step {step}/{steps} loss {loss.item():.4f} {time.time() - t0:.0f}s", flush=True)
        if time.time() - t0 > cfg["max_seconds"]:
            stop = "time"
            break
    model.save_pretrained(out / "lora")
    config.update({"steps_done": done, "samples_seen": done * cfg["batch"], "epoch_fraction": done / max(steps, 1),
                   "stop": stop, "train_seconds": time.time() - t0})
    c.write_json(out / "config.json", config)
    print(f"done {done}/{steps} steps, stop {stop}")
    return 0


def final(args) -> int:
    import torch
    from peft import PeftModel
    from transformers import AutoModelForCausalLM, AutoTokenizer
    run = Path(args.run)
    if (run / "metrics.json").exists() and not args.force:
        raise SystemExit("test metrics are computed once per configuration")
    config = json.loads((run / "config.json").read_text())
    torch.set_num_threads(config["hyper"]["threads"])
    tok = AutoTokenizer.from_pretrained(config["base"])
    model = AutoModelForCausalLM.from_pretrained(config["base"], torch_dtype=torch.float32)
    model = PeftModel.from_pretrained(model, run / "lora")
    model.eval()
    g = c.load_groups(["A_test"], Path(args.root))
    eps = g["A_test"]
    std = b.split_std([ep.bench for ep in eps])
    preds, trues, naive, ok, examples = [], [], [], 0, []
    t0 = time.time()
    for ep in eps:
        for start in ev.bench_starts(ep, 24):
            day = (start * b.EVERY + b.OFFSET) // b.TPD + 1
            assert ev.row_of(day, 360) == start
            prompt = day_line(ep, day) + "\n"
            enc = tok(prompt, return_tensors="pt")
            with torch.no_grad():
                gen = model.generate(**enc, max_new_tokens=config["hyper"]["max_new_tokens"], do_sample=False,
                                     pad_token_id=tok.eos_token_id)
            text = tok.decode(gen[0, enc["input_ids"].shape[1]:], skip_special_tokens=True)
            v = parse(text)
            truth = ep.bench.v[start + 24]
            if v is not None:
                ok += 1
            else:
                v = ep.bench.v[start]   # an unparsable answer scores as persistence (reported separately)
            preds.append(v)
            trues.append(truth)
            naive.append(ep.bench.v[start])
            if len(examples) < 5:
                examples.append({"prompt": prompt.strip(), "output": text.strip()[:300]})
    p, t, n = np.array(preds), np.array(trues), np.array(naive)
    out = {"n": len(t), "parse_rate": ok / len(t), "m3": ev.summarize(b.nmae(p, t, std)),
           "daily_naive": ev.summarize(b.nmae(n, t, std)), "examples": examples, "seconds": time.time() - t0,
           "config": {k: config[k] for k in ("base", "seed", "code_commit", "steps_done", "epoch_fraction")},
           "eval_code_commit": c.git_commit(), "unparsed_scored_as": "persistence"}
    c.write_json(run / "metrics.json", out)
    print(json.dumps({k: out[k] for k in ("n", "parse_rate")}), out["m3"]["mean"], out["daily_naive"]["mean"])
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    tr = sub.add_parser("train")
    tr.add_argument("--out", required=True)
    tr.add_argument("--base", default=DEFAULT_BASE)
    tr.add_argument("--seed", type=int, default=0)
    tr.add_argument("--root", default=str(c.DEFAULT_ROOT))
    for k, v in DEFAULTS.items():
        tr.add_argument(f"--{k.replace('_', '-')}", dest=k, type=type(v), default=None)
    fi = sub.add_parser("final")
    fi.add_argument("--run", required=True)
    fi.add_argument("--root", default=str(c.DEFAULT_ROOT))
    fi.add_argument("--force", action="store_true")
    args = ap.parse_args(argv)
    return train(args) if args.cmd == "train" else final(args)


if __name__ == "__main__":
    raise SystemExit(main())
