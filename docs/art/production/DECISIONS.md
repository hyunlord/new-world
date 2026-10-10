# Production decisions

- Start UTC:2026-10-10T11:06:37Z; twelve-hour cutoff:2026-10-10T23:06:37Z.
- PROD_BASE_SHA:0941e1b76ee250a017295fe923f7c0841dafaaa7. Direction merge:b5ead380c3eed047f6689224763d3713f7e5b02c, clean merge.
- Only the fresh /Users/rexxa/work/new-world-art-prod clone is modified. WIP gets each Part; green only advances after full gate. No simulation/bridge changes, no harness hooks.
- D3 throughout; muted D3 ground only. No D1/D2 assets installed.
- Image budget60 including failures; checkpoints20/40. No calls yet. Windowed runs always use shared lock; headless checks do not.
- Explicit task approval supersedes skill approval gates and generic lead/main reset workflow. Work follows the supplied Parts, preserving failed checks honestly.

## Part 0 baseline

Full baseline gate passed: cargo test 322s, Clippy14s, strict GDScript4s, bridge2s, smoke7s, smoke error scan0, Python1s. Results and logs: `gates/baseline/`. Baseline has no art scene tests yet. All renderer edits began only after this gate finished.

## Production rendering decisions

Under 200 villagers and 1,500 trees use pooled Sprite2D on one y-sorted layer. A y-sorted MultiMesh is reserved for a later 10k-agent renderer. Camera tween samples are snapped to the integer ladder, preserving nearest texels during the0.12s transition. The explicit art bible and BRIEF are the design system; web-specific Lighthouse/React rules do not apply to this native Godot surface.

## Generation observations

Calls1–4 requested1024×1024, but the built-in tool returned1254×1254. Raw files are preserved without resampling, and the converter measures their actual native grid. V_a1 andV_b1 accepted0/16 each; size and fidelity thresholds remain unchanged. After the first oversize sheet, prompts request less than half-cell figureheight and fewer flat colours; this is ordinary per-sheet size guidance, not the20-call checkpoint revision.

## Part2 gate and first look

Full gatepassed all13rows (cargo315s,Clippy1s,strict4s,bridge1s,smoke7s,Python2s,all6headlessarttests). Real finalplan10captures saved underp2, rawlosslessoutsidecommitteddocs. Firstlookandvisualverdictstored. The treepropertytest uses a directiontree through theproductionatlashelper; actualforestplacementverification remainsPart4/7. Reviewerrequested actualagentfeet andno-tint checks, added andpassed.
