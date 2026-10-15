# Art Pass 9 independent review — round 1, preparation

Status: SOURCE PREPARATION AND PIXEL IDENTITY REVIEW COMPLETE; FINAL REVIEW PENDING. No confirmed production defect found in the reviewed source. Six-framing pixel identity passes. Performance and publication approval remain pending.

## Independence and scope

Reviewed working source based on HEAD `034b8b1300efd344390a4187c7a4aa3842d71f78`, including uncommitted Part 1 implementation. Did not read GOAL sections 3–7, the original attachment, MIGRATION_PLAN, planner rationale, or parent history. Read READ_AUDIT and FFI_INVENTORY as permitted evidence, then checked relevant source independently. No builds, Godot sessions, foreign process operations, or code edits were performed. This document is the reviewer's only write path.

One graft repo-map call was used before source lookup; it reported approximately 1,857,527 tokens saved. GDScript inspection used direct source reads because the returned graph covered Rust/Python.

## Source findings

No prioritized production correction is requested at this checkpoint.

- Merge `49bc0b7c87159340b8024d6b9327bc394dc6daa0` has parents `fe185a23` and `da8e45c7`. Its scripts, scenes, and project.godot match original art `56059e0f`; the Rust delta against `da8e45c7` is confined to world_node.rs and static-source tests. The bridge delta is 404 added lines, zero deleted lines, preserving upstream implementation while adding art presentation getters.
- Current changes relative to the merge remain within the permitted source areas; unrelated .gitignore/.ignore changes are excluded. Rust implementation changes are confined to the causal getter/helper/tests in world_node.rs. `git diff --check` completed without errors.
- `scripts/ui/sim_reads.gd:22–40,65–81,103–109` validates sibling ownership and preserves uncached, mutable fake-world pass-through. `:83–125` caches ordinary getters by process frame and terrain by world lifetime. `:154–165` invalidates after mutations and immediately clears terrain for both reset methods; the connected deferred signal independently clears it again.
- `scenes/main.tscn` adds SimReads immediately after WorldSim, preserving reset connection order. No production protected direct getter/mutator call was found by the inspected searches; the static boundary test also scans game/test/support scripts, including dynamic dispatch.
- `sim_reads.gd:180–188` freezes nested dictionaries and ordinary arrays. Packed arrays remain governed by caller ownership. The HUD duplicates its compatibility dictionary (`hud_status_panel.gd:115`), and GodHandEffects replaces retained state on reset (`god_hand_effects.gd:65`). Source audit and the 600-frame consumer evidence did not expose packed-array caller writes.
- Terrain lifetime proof agrees with independent field-use search: writes occur in initialization/worldgen, including terrain.rs:127's swap, and fixtures. Runtime settlement/influence paths read terrain; no runtime terrain writer was found.
- `world_node.rs:1277` onward collects events in tile-input/ring order with first-seen ID deduplication. AgentRenderer preserves per-kind ingestion order and adds reset/overflow rearming (`agent_renderer.gd:326–401`). Synthetic tests exercise both cue types, ordering, reset, and overflow.

## Verified evidence and resolved limitations

- `gates/p1/reader-attempt1.log`: zero failures.
- `gates/p1/consumers-attempt1.log`: W0 and W500 each exercised for 600 frames, both zooms, open inspector/GodHand, enabled layers, and reset after retained cache; both report pass.
- `gates/p1/causal-attempt1.log`: 600 ticks, 8 recall ingestions, 0 combat ingestions, zero failures. Natural combat equivalence was not observed in that run; combat coverage is synthetic, not a claimed natural event sample.
- The earlier owner warnings in `gates/p1/equivalence-attempt2.log` are historical and **resolved**, not an open finding. Independently read `gates/p1-full/test_sim_reads_equivalence_p9.log`: 461 assertions, zero failures, no warnings. Its results.tsv row records exit 0 and 65 seconds. Current source unsets/restores owner around reader removal/reinsertion.
- At this checkpoint, `gates/p1-full-attempt2/results.tsv` contained 61 completed rows and provenance still said `running`. It is not classified as full PASS. Final exact-source/final-publication evidence remains separate.

## Independent A/A′ control recomputation

Read only `/Users/rexxa/work/art-production9-evidence/identity/attempt2/{A,Aprime}`. Used an independent Pillow RGBA difference, maximum over all four channels, and histogram count of nonzero pixels. No masking or generated comparison artifacts were used. All three pairs are 1920×1080, and their JSON sidecars match exactly.

| Framing | Differing pixels | Maximum channel difference | Identical A/A′ PNG SHA-256 |
|---|---:|---:|---|
| P1 | 0 | 0 | `9a75583e2c099bc5085dafd97fae1a1d9d2ced33673b6fd2fdc3df90a1a215e6` |
| P2 | 0 | 0 | `b97eca15b04dd38be8057ebfe2f964250625f006a7434a621f05db31d8c21dfb` |
| P3 | 0 | 0 | `69b4ab9ca4364196b274f8aaf7f688ffe0e0047f8e1f07f71710b77aa8a21459` |

These establish only the available baseline controls. They do not establish A/B equivalence or visual approval.

## Pending before final verdict

Independent block-median recomputation for each performance phase; F1–F7 verdict audit; quiet-lock/hold provenance; completed final publication gates bound to actual source; final allowed-path audit. These remain round 1 work when artifacts arrive. No complete independent review is claimed yet.

## Round 1 artifact continuation — six-framing identity

The implementation owner supplied source commit `a779e9270a54579e1e0c30c46fa3913100a7f784` and the completed `identity/attempt2/{A,Aprime,B,comparison}` evidence under `/Users/rexxa/work/art-production9-evidence`. The prior running-gate observation above is historical: the reviewer subsequently read all 70 results.tsv rows in p1-full-attempt2 and found zero nonzero exit statuses. This integration gate is separate from final publication checks.

Independently opened the full A/P1.png and B/P1.png pair, then the full A/P4.png and B/P4.png pair. P1 preserves the close village scene, river edge, residents, shelters/hearth/storage, top HUD, lower-left feed, and lower-right GodHand controls. P4 preserves the wide view with three settlement labels, terrain footprint and grey outside-map area, distributed residents/vegetation, and HUD/panels. No visible A/B change was observed in either pair. This is preservation approval for the optimization; pre-existing crowding and grey exterior in P4 are present in both images and are not newly introduced defects.

Read comparison/comparison.json and independently recomputed all six pairs using RGBA differences, including alpha, with no masks. Every image is 1920×1080. P1, P2, P3, P4, P5, and P6 each have A/A′ = 0 differing pixels / 0 maximum channel difference and A/B = 0 differing pixels / 0 maximum channel difference. These independent results agree with all six official comparison rows. F5 pixel identity is supported as PASS for these captured states. Timing/performance conclusions are not inferred from image equality.

No additional graft calls were needed; cumulative call count remains one and reported savings remain approximately 1,857,527 tokens.
