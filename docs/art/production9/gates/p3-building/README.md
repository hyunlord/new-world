# BuildingRenderer derived sleeper-count cache

Behavior-first test ran before the production edit: `before.log`, exit 0, 56 uncached reference comparisons. `after.log` repeats the same behavioral checks after the change. The final test also asserts Dictionary identity reuse for unchanged inputs.

The cached function is only `SHELTER_COLD.sleeper_counts`. Its key includes world identity, presence, Variant type, and exact values of agent xs/ys/states/target_kinds/colds and building xs/ys/kinds. Stored key arrays are deep duplicates; tests mutate each source packed column in place to exercise independence. `_reset_cold` clears the cache. All existing per-frame apply, frost fade, breath, tint and elapsed processing remains active.

Final runs completed 2026-10-15 12:11 KST before the W200 cooldown ended; no quiet lock was present at each launch:

- test_building_derived_cache_p9: exit 0, 56 behavioral comparisons plus unchanged-input cache reuse.
- test_shelter_cold: exit 0, failures=0.
- test_structure_states: exit 0.
- test_every_hearth: exit 0, failures=0.
- test_ysort_props: exit 0.
- gdscript_strict_check: exit 0, 42 files clean, FFI checks pass.
- git diff --check for changed scripts: exit 0.

Coverage includes repeated unchanged inputs with advancing delta and breath elapsed time; every relevant column changed in place, restored, absent, null and wrong type; building removal/insertion; reset; and replacement world followed by same-frame data mutation. Pixel identity and fresh ablation cost remain pending leader integration. No commit or full gate was run in this lane.
