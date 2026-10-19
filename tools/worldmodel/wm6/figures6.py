"""WM-6's paper figure (GOAL.md 11.2): the learning curves of the direct effect models from the committed judge
(`docs/worldmodel/wm6/runs/judge6/judge.json`; N on a linear axis, models offset by a few worlds so that coincident
markers stay visible): per horizon, the v2 test 12-cell error against N (top) and the closure
(`zero` − model) / (`zero` − floor) with its 95 % bootstrap interval (bottom). Observable models solid with circles,
privileged (diagnostic) models dashed with squares; one hue per model family in a fixed order (validated palette:
dataviz `validate_palette.js`, light mode, all checks pass; the contrast warning is relieved by the legend and the
paper's closure table). `zero`, `an5_arm` and the floor are grey reference lines, labelled directly. matplotlib only here.

usage: python3 tools/worldmodel/wm6/figures6.py   → docs/worldmodel/paper/figures/learning_curve.{png,pdf}
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
JUDGE = REPO / "docs/worldmodel/wm6/runs/judge6/judge.json"
OUT = REPO / "docs/worldmodel/paper/figures"
FAMILY_COLOR = {"ridge": "#2a78d6", "mlp": "#eb6834", "seq": "#1baf7a", "gbm": "#eda100"}   # fixed order, slots 1–4
NAMES = {"ridge": "ridge", "mlp": "MLP", "seq": "GRU", "gbm": "GBM"}
NS = (400, 800, 1600, 2000)
INK, MUTED, GRID = "#1f1f1e", "#6b6a63", "#e6e5df"


def main() -> int:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    j = json.loads(JUDGE.read_text())
    cur, ref = j["curves"], j["references"]
    fig, axes = plt.subplots(2, 2, figsize=(10, 7), sharex=True)
    for col, lag in enumerate(("7d", "30d")):
        ax_e, ax_c = axes[0, col], axes[1, col]
        for fi, fam in enumerate(("ridge", "mlp", "seq", "gbm")):
            for feat, ls, mk in (("obs", "-", "o"), ("priv", "--", "s")):
                m = f"{fam}_{feat}"
                if m not in cur:
                    continue
                ns = [n for n in NS if str(n) in cur[m]]
                xs = [n + (fi - 1.5) * 18 for n in ns]           # a small offset so coincident markers stay visible
                e = [cur[m][str(n)][lag]["value"] for n in ns]
                cl = [cur[m][str(n)][lag]["closure"] for n in ns]
                lab = f"{NAMES[fam]} {'observable' if feat == 'obs' else 'privileged (diag.)'}"
                kw = dict(color=FAMILY_COLOR[fam], ls=ls, lw=2, marker=mk, ms=6, mec="white", mew=1, label=lab)
                ax_e.plot(xs, e, **kw)
                ax_c.errorbar(xs, [c["point"] for c in cl],
                              yerr=[[c["point"] - c["lo"] for c in cl], [c["hi"] - c["point"] for c in cl]],
                              capsize=3, elinewidth=1, **kw)
        for name, y_ in (("zero", ref["zero"][lag]["value"]), ("an5_arm", ref["an5_arm"][lag]["value"]),
                         ("floor", ref["floor"][lag]["value"])):
            ax_e.axhline(y_, color=MUTED, lw=1, ls=":" if name == "an5_arm" else "-", zorder=0)
            ax_e.annotate(name, (2150, y_), xytext=(4, 0), textcoords="offset points", va="center", fontsize=8,
                          color=MUTED, annotation_clip=False)
        for y_, name in ((0.0, "zero"), (1.0, "floor")):
            ax_c.axhline(y_, color=MUTED, lw=1, zorder=0)
            ax_c.annotate(name, (2150, y_), xytext=(4, 0), textcoords="offset points", va="center", fontsize=8,
                          color=MUTED, annotation_clip=False)
        ax_c.axhline(ref["an5_arm"][lag]["closure"], color=MUTED, lw=1, ls=":", zorder=0)
        ax_c.annotate("an5_arm", (2150, ref["an5_arm"][lag]["closure"]), xytext=(4, 0), textcoords="offset points",
                      va="center", fontsize=8, color=MUTED, annotation_clip=False)
        ax_e.set_title(f"v2 test, {'one week' if lag == '7d' else 'one month'}", color=INK, fontsize=11)
        ax_e.set_ylabel("12-cell scaled effect MAE (lower is better)", color=INK, fontsize=9)
        ax_c.set_ylabel("closure: (zero − model) / (zero − floor)", color=INK, fontsize=9)
        ax_c.set_xlabel("training worlds N (replicate 0)", color=INK, fontsize=9)
        for ax in (ax_e, ax_c):
            ax.set_xlim(250, 2150)
            ax.set_xticks(NS)
            ax.set_xticklabels([f"{n:,}" for n in NS])
            ax.grid(True, color=GRID, lw=0.8)
            ax.tick_params(colors=MUTED, labelsize=8)
            for s in ("top", "right"):
                ax.spines[s].set_visible(False)
            for s in ("left", "bottom"):
                ax.spines[s].set_color(GRID)
    h, lb = axes[0, 0].get_legend_handles_labels()
    fig.legend(h, lb, loc="lower center", ncol=4, frameon=False, fontsize=8, bbox_to_anchor=(0.5, -0.02))
    fig.tight_layout(rect=(0, 0.06, 0.97, 1))
    OUT.mkdir(parents=True, exist_ok=True)
    for ext in ("png", "pdf"):
        fig.savefig(OUT / f"learning_curve.{ext}", dpi=160, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {OUT / 'learning_curve.png'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
