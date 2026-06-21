# Locale-infra prep — resync registry/compiled to source truth + de-treadmill the locale-lock class

## Section 1: Implementation Intent

Adding a single new localization key (Direction-2 slice 2-5a, currently stashed) exposed
real locale-infrastructure debt at HEAD `461fa166`:

1. **Stale compiled artifacts.** `localization/key_registry.json` and both
   `localization/compiled/{en,ko}.json` reported `active_key_count: 5116`, but the compiled
   `strings` maps already contained **5135** keys — 19 shipped UI keys (Phases 3-γ causal, 7-δ
   social, 8-δ memory, 9-δ combat) were added to the `localization/fluent/{en,ko}/messages.ftl`
   source but the registry + compiled-meta counts were never re-synced. The registry was
   internally consistent (5116 active + 5 removed = 5121 total) yet **stale vs the actual source**.

2. **A frozen-count lock that breaks on every legitimate key add.** The harness lock `a17`
   (`harness_p3_gamma_2_beta_key_registry_active_count_5116`) asserted the literal
   `"active_key_count": 5116`. Any correct recompile changes that number, so the lock punished
   legitimate additions — a per-slice lock-bumping treadmill.

3. **A permanent git-freeze on `localization/`.** The harness lock `a18`
   (`harness_p12_alpha_a18_shader_and_locale_unchanged`) froze `localization/` against ALL
   modification. That made sense when no planned stage touched locale; it is now false (2-5a and
   ongoing UI slices actively develop localization).

Approach: (A) recompile so registry + compiled reflect the true source keyset (5135);
(B) release a18's `localization/` freeze via its own documented precedent (it already released
`world_renderer.gd` and `shaders/palette_swap.gdshader`); (C) convert the frozen `== 5116` literal
in a17 into a **consistency invariant** — registry internal arithmetic + registry↔compiled
agreement — so a correct recompile always passes while real drift (the bug fixed here) still
fails. After this, 2-5a's key and every future locale slice ride clean.

This is a harness-infra + localization-data change. No production simulation logic changes.

## Section 2: What to Build

Exactly five files change. No others are authorized.

**Harness lock tests (Rust, sim-test crate — test code only, no production logic):**
- `rust/crates/sim-test/tests/harness_p3_gamma_2_beta_tile_click_chain.rs`
  — Convert assertion **A17** from the frozen `src.contains("\"active_key_count\": 5116")` literal
    into a consistency check (parse the three JSON files with `serde_json` and assert the
    invariants in Section 3). Rename the test fn
    `harness_p3_gamma_2_beta_key_registry_active_count_5116` →
    `harness_p3_gamma_2_beta_key_registry_active_count_consistent`. Keep the `A17 PASS:` print
    marker. Do NOT touch A14/A15/A16 (they assert 13 specific keys are present — addition-safe)
    or any other assertion in this file.
- `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs`
  — In assertion **A18** (`harness_p12_alpha_a18_shader_and_locale_unchanged`), remove
    `"localization/"` from `forbidden_prefixes` (set it to an empty `[&str; 0]`), with a
    documented release comment in the same style as the existing `world_renderer.gd` /
    `palette_swap.gdshader` releases. Keep `forbidden_exact = ["scripts/ui/panels/causal_panel.gd"]`
    unchanged (causal_panel stays frozen). Update the assert message + `A18` print marker to say
    the localization lock was released. Do NOT touch any other assertion.

**Localization compiled artifacts (data — recompiled by `tools/localization_compile.py`):**
- `localization/key_registry.json` — `active_key_count` 5116 → 5135, `key_count` 5121 → 5140.
- `localization/compiled/en.json` — `meta.active_key_count` 5116 → 5135.
- `localization/compiled/ko.json` — `meta.active_key_count` 5116 → 5135.

**Referenced but NOT modified** (these paths appear inside the a18 test's retained-freeze /
released-precedent comment block — the plan may mention them, but the change does NOT modify
them; listed here only so the scope guard recognises them as in-context):
`scripts/ui/panels/causal_panel.gd` (a18 keeps this frozen via `forbidden_exact`),
`scripts/ui/world_renderer.gd` and `shaders/palette_swap.gdshader` (prior precedented releases
cited in a18's comment).

**Locked scope boundary:** NO `rust/crates/sim-core|sim-systems|sim-engine|sim-bridge|sim-data`
change, NO `scripts/` change, NO new GDScript files, NO FFI change, NO localization SOURCE key
additions/removals (the 19 keys already exist in the fluent source — this is a resync, not an
authoring change). The fluent source `.ftl` files are NOT modified.

## Section 3: How to Implement

**Fix A — resync (already producible deterministically):**
```bash
python3 tools/localization_compile.py --project-root .
```
The tool reads `localization/fluent/{en,ko}/messages.ftl` (manifest `source_format: "fluent"`),
recomputes `active_keys = sorted(union of compiled strings keys)`, and rewrites the registry +
compiled files via `_write_json_if_changed`. Idempotent: re-running produces no further diff.
Verified: exit 0, `filled=0`, owner/duplicate guards clean, `active_key_count` → 5135.

**Fix B — a18 release:** set `let forbidden_prefixes: [&str; 0] = [];` and document the release.
The git-status mechanism is otherwise unchanged; it still freezes `causal_panel.gd`.

**Fix C — a17 consistency invariant.** Parse `key_registry.json`, `compiled/en.json`,
`compiled/ko.json` with `serde_json` (a sim-test dev-dependency; established `include_str!`
pattern in this file). Assert ALL of:
1. `registry.active_key_count + registry.removed_key_count == registry.key_count`
2. `registry.key_count == len(registry.key_to_id) == len(registry.keys)`
3. `registry.removed_key_count == len(registry.removed_keys)`
4. `registry.active_key_count == len(compiled/en strings) == len(compiled/ko strings)`
5. `compiled/{en,ko}.meta.active_key_count == len(their own strings map)`

Invariants 4 + 5 are the ones that catch the exact drift fixed here (compiled meta frozen at 5116
while strings advanced to 5135). A correct full recompile keeps all five true, so a legitimate
+1-key addition + recompile rides clean.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | `tools/localization_compile.py` recompile (data resync) | 🔴 DIRECT | — |
| T2 | a18 `localization/` lock release (test) | 🔴 DIRECT | — |
| T3 | a17 frozen-count → consistency (test) | 🔴 DIRECT | T1 (registry must be resynced first) |

All DIRECT: this is harness-lock + localization-data coordination, not feature logic. There is no
production simulation code to dispatch. Dispatch ratio N/A (infra-text + data maintenance).

## Section 5: Localization Checklist

**No new localization keys.** This is a registry/compiled **resync** of 19 pre-existing keys that
already exist in `localization/fluent/{en,ko}/messages.ftl` (verified present in both locales and
referenced in `scripts/ui/panels/causal_panel.gd` + the p7/p8/p9 harness tests). No source `.ftl`
edits; no `en/ko` symmetry change.

## Section 6: Verification & Notion

Gate:
```bash
cd rust && cargo test -p sim-test --test harness_p3_gamma_2_beta_tile_click_chain \
                                  --test harness_p12_alpha_camera_zoom -- --nocapture
# Expect: A14/A15/A16 PASS, A17 PASS (active=5135, total=5140, removed=5), A18 PASS (locks released)
cd rust && cargo test --workspace   # full gate; only the baseline {a15} determinism test fails
cd rust && cargo clippy --workspace --all-targets -- -D warnings   # clean
```
Regression: the deterministic Step-2.7 guard set-differences the gate result against
`.harness/baseline/known_failures.txt` (baseline = `{harness_scarcity_a15_determinism_of_scarcity_run}`).
Expected `regression_status: CLEAN` — a17/a18 are NOT in the baseline and must pass.

Scope: `git diff` touches only the five files listed in Section 2. No Notion page.

Sanity: adding one new locale key + recompile keeps a17 + all 13-key locks green (the 2-5a key
rides clean) — confirmed conceptually: invariants 1–5 hold for any self-consistent recompile.
