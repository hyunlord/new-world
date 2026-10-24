# M-UX execution decisions

- D1 (2026-10-25): The attachment overrides checkout/branch instructions: only `/Users/rexxa/work/new-world-mux-dev`, branch `v8/mux`, based on fetched `origin/v8/main` `67a6232ab41a4bf5051b11303bcac97543b111f0`. No builds/runs/edits in pass4 or play2-dev; no process signals. First commit `ee10a4ea2` preserves the supplied goal verbatim.
- D2: Execute the owner's detailed A/B/C plan without an approval handoff. All unresolved choices are recorded here; no owner questions during execution. User scope takes precedence over workflow skill approval defaults. Native agents receive bounded ownership and cannot publish or recursively delegate.
- D3: Freeze a base release probe and colony fixture summaries outside Git before simulation edits. Colony summaries already exclude timing/Git metadata: compare them with literal `cmp`, not a new normalization filter. Existing baseline identity data remains `/Users/rexxa/work/m55-data/base/identity`.
- D4: Split behavior-changing commits from behavior-preserving refactors. Before each behavior change, record expected effects and the fixture comparison. Every refactor requires baseline identity plus soak/story/neighbours/F10/M55 seeds 0–7 × 48-day byte identity with steward off. No mixed commit and no relaxed gate.
- D5: Stage A implements camera/controls/help/alerts/learning/steward as required; optional drops follow A9, A6, B5, B4 only. UI styling derives from the existing Galmuri theme and panels. UI and Rust source ownership stay separate; one coordinator runs Godot and gates.
- D6: Every stage freezes a clean commit before the full gate, and publishes only that green commit to v8/wip then v8/main by fast-forward. No report-only ungated changes reach v8/main. A report records pre-gate evidence and points to the external exact-HEAD gate receipt; final delivery reports actual receipts after completion.
- D7: Raw builds, captures, calibration rounds, measurements and receipts live in `/Users/rexxa/work/mux-data/`. Preserve failed attempts and held-out results. Calibration is seeds 0–7 only, at most six rounds; held-out 1000–1031 is never used to tune rules.
- D8: Conservative hard-stop clock starts 2026-10-25 03:00 KST; deadline 2026-10-26 11:00 KST. At the limit finish the current step, gate, publish only green work, report partial requirements explicitly. Stage targets A 10 h, B 14 h, C 4 h do not reduce required scope.

## Execution ledger

| Step | Status | Evidence / next action |
|---|---|---|
| A0 isolated checkout, original goal, safety capture | IN_PROGRESS | First commit ee10a4ea2; base release build in progress |
| A1–A4 camera, controls, minimap, data-driven help | PENDING | UI integration map being collected |
| A5/A7 alerts and learning helper | PENDING | Bridge-read coverage map being collected |
| A8 steward, replay, save and calibration | PENDING | Command architecture map being collected |
| A6/A9 optional goals/settings and A10 rendering | PENDING | Preserve specified drop order |
| A11 UI/runtime/held-out evaluation, full gate, publish/report | PENDING | No completion claimed |
| B1 whole-tree review and prioritized cleanup plan | PENDING | Review precedes refactors |
| B2/B3 registries and data-only extension gate | PENDING | At least items/blueprints/recipes mandatory |
| B4/B5 measured behavior changes; B6 safe refactors | PENDING | Separate commits and fixtures |
| B7 full gate, publish/report | PENDING | No completion claimed |
| C1 proof-backed dead-code cleanup | PENDING | Preserve frozen baseline/research paths |
| C2 issue/history backfill and GitHub mirrors | PENDING | Initial registry started; auth check pending |
| C3 final gate, publish, clean removal and report | PENDING | Owner launcher checkout remains untouched |
