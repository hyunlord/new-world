# ADR 0016: Reuse registered art operations for climate candidates

## Context

B2 supplies climate ground and wild flora art on `v8/art-batch-b2`. The game importer and simulation remain owned by C2/C3. The interrupted pass resumed under a three-attempt-per-asset budget and an instruction to skip optional variants.

## Decision

Reuse the existing flora crop, uniform reduction and anchor operations, canopy-only seasonal renderer, and deterministic world-coordinate ground edge recipes. Keep full, harvested and winter semantics explicit. A harvested evergreen fruit tree retains its leaves; a harvested wood tree uses its existing stump.

Preserve raw generations, exact prompts, reference and source hashes, rejected attempts and check output outside the repository. Commit selected candidates, sidecars, manifests and contact sheets. Record numeric failures separately from visual review; do not lower a frozen threshold to fill a slot. Three unsuccessful attempts leave an explicit gap.

Provide `art/climate/art_import.draft.json` and `bindings.draft.json` as additive proposals. Verified identities and proposed selectors remain distinct. The inspected runtime has no climate texture selector or olive wood material contract; candidate PNGs do not establish either. Do not apply the drafts during this art pass.

## Alternatives considered

- Generate every seasonal image independently: rejected because the existing renderer already preserves registered geometry and the planner requested economy.
- Add runtime climate/state selection here: rejected because this is an art-only pass and those consumers belong to C2/C3.
- Substitute a failed state with another state's image: rejected because it hides an integration gap and breaks state meaning.

## Consequences

Generated originals, normalized frames, reused assets and derived seasons/edges have separate counts. C2/C3 must merge metadata, confirm identifiers, connect selectors and warm-up, and verify real game rendering. Optional apple work and additional new silhouettes are outside the resumed completion scope.

## Date

2026-11-11.

## Source

- `/Users/rexxa/work/art-batch/BRIEF.md` and `BRIEF2.md`.
- Planner note, 2026-11-11 09:58 KST.
- `art/climate/COVERAGE.md`, `art/climate/STYLE_CONTRACT.md`.
- `art/flora/normalize.py`, `art/flora/seasons.py`, `art/compositor/edges.py`.
