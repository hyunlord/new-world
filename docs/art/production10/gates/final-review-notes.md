# Final gate read-only review

- HEAD: 208eb522f736e710708585901a021c1761fc50b4
- Expected result rows: 82
- Root owns gate session 52711; no additional build or Godot launches by reviewer.
- Initial observation: results.tsv empty, gate is still in its first check. Status NOT_VERIFIED pending terminal results.

## First concrete failure

- cargo_test exited 101 after 142 seconds. `harness_revitalize_inspector_gather_food_is_not_social_target` failed at `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1154`: expected 8 target labels, actual 9.
- The panel `scripts/ui/panels/agent_inspector_panel.gd:39` now has the ninth label `INSPECTOR_TARGET_ACTIVITY_PLACE_FMT` at index 8. Existing bridge `rust/crates/sim-bridge/src/ffi/world_node.rs:1697` maps `TargetKind::Activity` to 8. Thus the eight-label static assertion is stale relative to the actual FFI target domain; keys 5/6/7 remain unchanged.
- Recommended bounded correction after root ends the source freeze: expect nine labels and explicitly assert index 8, preserving existing gather/social checks. No correction made by this reviewer.
- Initial succeeding rows: clippy, strict, bridge, smoke, smoke_errors, python; GDScript checks continue. This attempt cannot PASS even if all remaining checks pass.

## Terminal review

- Attempt 1 completed with gate terminal exit **1**; all **82/82 expected rows** present. **81 rows exited 0**, only `cargo_test` exited 101 from the stale target-label assertion described above.
- Source/HEAD/gate manifest stable: `source_stable=true`, `changed_source_paths=[]`.
- Bridge rebuild changed debug dylib SHA256 from `c757571424054d7f916103a87773b63c069c94a26779ff6bb22465d9fcf12e3f` to `85d2b0c3fc704d15f6c4d334c3c5d6736f025536103576f360723d85a0442a3c`; this is separately reported rebuild provenance, not source drift.
- `test_sim_reads_consumers_p9` passed W0/W500 with 600 frames each; equivalence test logged 497 assertions and 0 failures. No additional failed result rows.
- Attempt 1 evidence remains under `final-gate-supervisor-attempt1/` and repository `docs/art/production10/gates/final/` until root archives it. Root plans bounded static assertion correction followed by a fresh full gate; this reviewer performed no source edits or runtime launches.

## Attempt 2 terminal review

- Frozen HEAD `02f8721d04494dedc176a1ef34f1601338d89374` preserved before/after.
- Full gate terminal **0**, all **82/82 checks exit 0**; independent supervisor result PASS, no errors, source_stable=true, changed_source_paths=[].
- Rust cargo_test now passes (531 seconds), including the corrected target-label static assertion.
- Debug dylib rebuilt from `85d2b0c3fc704d15f6c4d334c3c5d6736f025536103576f360723d85a0442a3c` to `6b76bbc3bb25d4b9de088f191b8337efbb57e9b7a6bdd9b45e6b3fc5f7405c85`; binary provenance differs, source does not.
- Evidence: `final-gate-supervisor-attempt2/` and repository `docs/art/production10/gates/final-attempt2/`. This gate does not substitute for missing Part 4 final performance measurements.
- Reviewer performed read-only monitoring; no source edits, builds, or Godot launches.
