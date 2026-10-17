# Source of the vendored modules

- **Repository:** https://github.com/Valentyn1997/CausalTransformer (the authors' implementation of the Causal
  Transformer, Melnychuk, Frauen and Feuerriegel, ICML 2022, which also contains their CRN implementation).
- **Commit:** `c49a24faa57af966501e241f57a26b528f874a53` (2024-08-13), cloned 2026-10-16 for reading.
- **License:** MIT (`LICENSE`, copied unchanged; sha256 `1b996b9d8fa573fb473dddefcf5a5ec87c551e9ba75f8114efe1dc5ed7a2cace`).

## Files taken

| here | upstream | sha256 of the upstream file | change |
|---|---|---|---|
| `utils_lstm.py` | `src/models/utils_lstm.py` | `3ed6f8808c60602b45cef2ab663e80b88e2c1f26a92845ccb7a81fb706c3e578` | none (copied unchanged) |
| `utils_transformer.py` | `src/models/utils_transformer.py` | `4028d4507c246d619a0645791c8933e919099ff3900cc014a34a0d35f60bbf2c` | none (copied unchanged) |
| `utils.py` | `src/models/utils.py` | `b6814c3acab6b26a1965328bfd6401d0f88258c1424dc4e150276c4681161fca` | only `grad_reverse`, `bce` and `BRTreatmentOutcomeHead` kept, their text unchanged; the file's PyTorch-Lightning / MLflow imports and the classes that need them (`FilteringMlFlowLogger`, `ROutcomeVitalsHead`, `AlphaRise`, `clip_normalize_stabilized_weights`) removed; a header comment added |
| `LICENSE` | `LICENSE` | (above) | none |
| `__init__.py` | — | — | empty, new (package marker) |

Nothing else of the repository is used: no Lightning, Hydra, MLflow, Ray or data code. How WM-4 uses these modules —
the 4-class treatment classifier that replaces `BRTreatmentOutcomeHead.linear3`, the `F.cross_entropy(...,
reduction="none")` used instead of `bce` (whose `reduce=False` argument is deprecated), the training loops, the EMA
helper and the rollouts — is in `tools/worldmodel/wm4/causal.py` and listed in `docs/worldmodel/wm4/METHODS.md`.
