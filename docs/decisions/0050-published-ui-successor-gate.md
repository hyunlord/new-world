# ADR 0050 — Validate UI successors without extending M-CORE exceptions

## Context

M-UX2.1 resumes on published M-CORE3814828c. Its Rust tree is identical to measured826a7ecd, but the original P3d guard rejects any added verifier/tool files. The original CORE performance checker fixes its comparison BASE at9855bd. P7's side64 exception applies only to M-CORE. A UI pass cannot remove those guards or silently reuse that exception.

## Decision

Use an explicit, hash-pinned `published_ui_successor` adapter. Require published BASE ancestry, exact committed and live Rust identity, unchanged Cargo.lock, a complete committed delta limited to UI/rendering/tool/documentation paths, and exact live source/tool identity. Root project/bridge configuration is included in live identity. Existing historical tools and their dependencies remain byte-identical to published BASE, with only the explicitly listed gate integration and UI test files allowed to change. Seal the config only after the final commit.

Bind the original accepted45-step gate and successful launcher terminal, evaluation and shadow configs by independently embedded hashes. Validate their frozen references and retain original P3d chain metadata. The historical chain is inherited from that accepted gate: its old multi-worktree Python verification is explicitly NOT_RERUN, so this pass executes no code in another session's checkout. Fresh seed0 OFF/ON shadow fixtures still run; outcome, why, story and historical raw performance checks are recomputed through the existing gate. Old failure labels remain byte-identical, distinctly labeled historical.

Require a separate fresh, interleaved published381 BASE versus candidate performance comparison with the ordinary standing1.10 and C3 limits, per-repeat load admission and original raw inputs. No P7 waiver applies. Identical runtime binaries may be used only with explicit provenance that both runtime trees are identical; this is not represented as independent builds.

## Alternatives considered

Changing old receipt source labels, dropping P3d to select the wrong seed protocol, weakening its tools guard, enabling M-CORE-only P7 exceptions, and treating old performance as fresh were rejected. A full relocated historical verifier projection was considered; the chain embeds absolute, independently pinned historical views, so the adapter instead declares accepted-gate inheritance while retaining the chain and recomputing raw outputs.

## Consequences

A final HEAD/config change requires resealing. Runtime, lock, missing evidence, undeclared files or failed fresh comparisons fail closed. Historical side64 failures remain visible; no claim is made that they have been repaired. Gate logs distinguish inherited behavioral acceptance, fresh fixture execution and fresh performance. Adapter approval does not itself grant publication or satisfy the FPS/GPU checks.

## Date

2026-10-28.

## Source

Owner M-UX2.1 resume note and explicit approval to include the adapter; `tools/v8/core_ui_successor.py`, `core_ui_successor_perf.py`, `core_gate_checks.sh`; published M-CORE acceptance `cb162f3c638ab9569dd7127e5dee7d38c5d3927f`; `docs/v8/mux2_1/MUX21_DECISIONS.md`.
