# Revitalize metrics

## Measurement contract

All measurements use the dedicated DGX clone, release probe, 1440 ticks/day,
120-tick regular samples, exact day 1/3/5/10 checkpoints and independently
constructed determinism twins. JSONL has 34 stable schema-version-1 fields.
Distributions use linear quantiles. T6 is cumulative sampled idle need
breaches divided by sampled living agents (including tick zero). Performance
is primary-engine tick wall time divided by exact pre-tick agent counts;
wall-clock run duration also includes output and twin verification.

## Part 1 legacy reference

Four identical repeats, seeds 0..4 (seed ignored by the fixture), 10 days:
DETERMINISM PASS. Day 10: 44 alive, 213 births, 233 dehydration deaths,
zero starvation/combat deaths. Final fingerprint `b9ce9cb344a98de5`.
The exact 2880-tick regression pin is frozen in
`harness_revitalize_legacy_pin.rs`; it must never be updated to accommodate
new behavior. Detailed distributions: `probe/p1_legacy/summary.md` and JSON.

## Game tuning

Pending Part 2 untuned reference and Part 3 bounded tuning rounds.
