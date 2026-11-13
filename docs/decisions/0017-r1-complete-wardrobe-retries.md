# R1 complete wardrobes through bounded, recorded retries

## Context

R1's first pass accepted 13 of 84 combinations. None of seven wardrobes had
all 12 age/sex/hair combinations. BRIEF4 authorizes six total attempts per
combination and requires whole wardrobes before importability.

## Decision

This extends ADR 0016 for the owner-authorized second pass: its first-pass
ban on recolouring is replaced by the recorded global correction below.

Rank wardrobes by existing accepted count and failed best-trial worst-ratio
deviation. Finish each bounded retry phase in order. Keep the five inclusive
0.9–1.1 ratios, frozen masks/probes, ten-frame structural checks and actual
1×/2× visual review unchanged. Count deterministic edits as attempts too.

Prefer the closest structurally valid prior sheet and a recorded spatially
uniform colour transform when geometry already passes. The transform preserves
alpha and geometry, leaves transparent/near-white pixels alone and records its
source hash, gains, optional value offset and colour-curve range. That range does not alter
measurement probes or their alpha thresholds. It changes skin/hair colour too, so fresh visual review
is mandatory. For geometry failures, edit the shipped or closest same-age/sex
sheet with explicit pose preservation rather than reusing a failed silhouette.

Only 12/12 body wardrobes enter the draft importable top list. Portrait work
starts only after that body milestone; readiness remains explicit until all
portraits and downstream importer/game checks pass. Partial assets and prior
portraits remain evidence, not fallback aliases. No runtime import is performed.

## Alternatives considered

- Relax the ratios or move masks: rejected because it changes the owner's bar.
- Fill missing looks with renamed existing art: rejected because it conceals gaps.
- Regenerate every colour failure: rejected because a deterministic recorded
  correction preserves valid poses and can be replayed exactly.
- Retry beyond six: rejected because the authorization is bounded.

## Consequences

A nearly complete wardrobe may remain non-importable after its budget is spent.
Raw sources, per-attempt approaches and reviews stay outside the repository;
adoption manifests bind their hashes. `art/batch_r1/retry.py` is a thin helper
using existing dependencies, not a new importer or rendering path.

## Date

2026-11-14

## Source

`/Users/rexxa/work/art-batch/BRIEF4.md`, section R1 pass 2;
`art/batch_r1/coverage.json`; `art/batch_r1/retry.py`;
`/Users/rexxa/work/art-batch/R1/pass2/PLAN.md`.
