# worldsim-testing — writing assertions that can fail

Carried over from V7's `evaluation_criteria.md` (the threshold rationale
types), with the V7 pipeline, scores and seed-42 fixtures removed and the
lessons in `../SKILL.md` folded in.

## 1. Every threshold has one rationale

Write the rationale type and the observed value in a comment above the
`assert!`. A threshold with no rationale is arbitrary; reconsider it.

| type | meaning | how to set it |
|---|---|---|
| Invariant | logically necessary; a violation is a bug (nobody on water, stock never negative, every member's settlement exists) | exact: `== 0`, always true/false |
| Rule-derived | follows from a `WorldRules` value (burn per hour × hours, cap per bush) | compute from the rule's fields in the test, not a literal |
| Measured baseline | an emergent value measured on the baseline | measure on tuning seeds, assert on held-out seeds with an explicit margin (lower bound at most ~30 % of observed unless argued; upper bound for runaway) |
| Regression guard | a known bug must not come back | cite the commit or lesson; tight enough to catch the old bug |
| Soft | expected emergent behaviour without a hard basis | last resort; say "soft" in the comment and print the value |

```rust
// Measured: lit share 0.93 / 0.91 / 0.95 on tuning seeds 3, 7, 11 (2026-10-21).
// Asserted on held-out seeds 19, 23 with threshold 0.60.
assert!(lit_share >= 0.60, "seed {seed}: winter hearths lit {lit_share:.2} < 0.60");
```

## 2. Recipe for a live-rule test

```rust
#[test]
fn settlement_<what_it_asserts>() {
    for seed in [19_u64, 23] {                        // held-out seeds, chosen before measuring
        let mut engine = sim_test::baseline_engine(seed);
        let mut events = 0_u32;
        for _ in 0..(40 * 1440) {                     // 40 game days
            engine.tick();
            // observe here if the quantity is per tick
        }
        // read engine.world / engine.resources
        assert!(events > 0, "seed {seed}: the event under test never happened");
        // the threshold on the measured invariant, with its rationale above
    }
}
```

Checklist before committing a test:
1. **Can it fail?** Break the code (delete the fix, flip the condition) and watch it fail. A test that passed before its change existed is vacuous.
2. **Which bug still passes?** Name one. If a plausible bug passes, tighten the assertion (exact value, ids, order, both bounds).
3. **Did the thing happen?** Assert the count of the checked event is > 0 before asserting about each event. Fail on a missing input file or scene; never skip silently.
4. **Is the expected value independent?** Derived from the rule, by hand, or from a separate re-implementation — never copied from the function's own output.
5. **Is the seed meaningful?** It must exercise the path (a seed with a second village for a relations test). Grep the tests for the seed range before using it; held-out seeds are looked at once.
6. **Censoring.** For "X happens within N days" or time-to-event: runs that end before the event are censored, not failures. Report the censored count, or restrict to events with at least the needed window left, or use a survival statistic decided before the run.
7. **Ticks.** 1,440 ticks = 1 game day; tick 0 is 08:00 of display day 1 (the clock offset is 480 ticks). Stepping to tick T runs systems for ticks 0..T-1, so an effect booked at T is visible only after tick T has run.
8. **Branches.** After tuning, count how often each intended branch's condition holds in the run; assert each one is reachable (Pass 20's exchange branch was unreachable for a whole pass).
9. **Order.** For an order-sensitive assertion, repeat over many fresh engines (32) so hash-order luck cannot pass it.
10. **Rule-independent fixtures** (`SimEngine::new` with `WorldRules::default()`) are fine for pure mechanics, but they switch the rules-gated systems off; they do not cover those systems.

## 3. Things that look like tests and are not

- Static text checks on `.gd`, `.tscn`, `.rs`, localization files (deleted in V8-0; they broke on every rewrite and proved nothing).
- A fixture that sets the value the code under test should set.
- `> 0` on a quantity observed in the hundreds.
- A determinism twin that does not compare the new state.
- A Godot headless run judged by exit code: Godot exits 0 on GDScript parse failure; require a completion sentinel and scan stderr for `SCRIPT ERROR` / `ERROR` (the gate's strict check and smoke do this).
- A check whose "PASS" text is printed before or without the comparison that should produce it.

## 4. Synthetic paths

When natural worlds never reach a write path (fission cancel, remnant merge,
porter loss), write a synthetic test that forces it and asserts the entries
and their order, and state "verified only synthetically" in the report.
