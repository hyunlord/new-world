"""M1b-obs (docs/worldmodel/wm4/GOAL.md 7.3): WM-2's M1b-act recipe trained from scratch on the observational split.

`register()` adds `obs_v19` to `wm1.common.DATASETS` (no edit of `wm1/common.py`): train = the observed train episodes,
val = the observed val episodes (selection on val factual T2 24 h, the observational practice), test = the test worlds'
factual episodes. Every process that loads an M1b-obs run calls `register()` first (`g.load_predictor` reads
`c.dataset(config["dataset"])`). `train` then calls `m1_tf.main` with WM-2's flags (`wm2/launch/queue.sh`, GOAL.md 7.3);
`final` calls `m1_gru.main(["final", ...])` (factual T2 on obs test).

Stall guard (WM-3's `finetune.py`, `cc18dc3b`): every thread's Python stack is dumped to stderr on SIGUSR1 and when one
validation evaluation (`m1_gru.val_score`) runs longer than 600 s.

usage (DGX):
  python3 tools/worldmodel/wm4/m1_obs.py train --out RUN --seed S --max-steps 8000 --max-seconds N
  python3 tools/worldmodel/wm4/m1_obs.py final --run RUN --out DIR
"""
from __future__ import annotations

import faulthandler
import signal
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import wm4  # noqa: F401
from wm1 import common as c

MANIFEST = c.REPO / "docs/worldmodel/wm4/obs_manifest.json"
FLAGS = ["--variant", "sev-blind_action-aware", "--window", "336", "--context", "168", "--warmup", "168",
         "--ramp-steps", "1500", "--eval-every", "150", "--patience", "12", "--batch", "32", "--clamp-nonneg", "1",
         "--k-max", "168", "--roll-min-context", "167", "--roll-starts", "1", "--tf-l1", "1.0", "--threads", "8",
         "--v-weight", "5.0", "--val-batch", "512"]
EVAL_STACK_DUMP = 600


def dataset() -> "c.Dataset":
    return c.Dataset("obs_v19", MANIFEST, {"train": ("obs_train", "train"), "val": ("obs_val", "val"),
                                           "test": ("obs_test", "test")},
                     ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True, None,
                     c.V17_EXTRA)


def register() -> None:
    if "obs_v19" not in c.DATASETS:
        c.DATASETS["obs_v19"] = dataset()


def guard() -> None:
    """SIGUSR1 and > 600 s validation evaluations dump every thread's stack (train.out)."""
    from wm1 import m1_gru as g
    faulthandler.register(signal.SIGUSR1, file=sys.stderr, all_threads=True)
    orig = g.val_score

    def guarded(*args, **kwargs):
        faulthandler.dump_traceback_later(EVAL_STACK_DUMP, exit=False, file=sys.stderr)
        try:
            return orig(*args, **kwargs)
        finally:
            faulthandler.cancel_dump_traceback_later()

    g.val_score = guarded


def main(argv=None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    register()
    if not argv or argv[0] not in ("train", "final"):
        raise SystemExit(__doc__)
    if argv[0] == "train":
        from wm1 import m1_tf
        guard()
        return m1_tf.main(["train", "--dataset", "obs_v19"] + FLAGS + argv[1:])
    from wm1 import m1_gru as g
    return g.main(["final"] + argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
