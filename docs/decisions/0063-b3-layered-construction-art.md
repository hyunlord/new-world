# B3 layered construction art

## Context

The round-two art brief requests blueprint, foundation, half-built, built,
damaged and ruined states for every A2 building. A2 rooms already use the
material compositor with independently sorted floors, walls and props. The
B3 branch must provide art and import proposals without changing simulation
or game rendering code.

## Decision

Keep room lifecycle states as explicit layered recipes in
`art/batch_b3/rooms/recipes.json`. Reuse the existing material resolver and
wall topology. A foundation exposes the original 16px top plus 8px facade;
a half-built room mixes that course with original 60px wall cells. Damage
removes a visible non-door wall cell and places unchanged adopted debris in
the breach. Ruins retain sparse short courses and remove furniture.

Preserve built previews byte-for-byte against the selected A2 recipes.
Blueprints reuse the existing runtime ghost operation: 25% alpha wash and a
one-pixel pale outline on the transparent side of the silhouette. Recipe
outputs are counted separately from generated originals. Rejected or
unavailable source art stays an explicit gap.

C2/C3 must resolve real blueprint/building codes, lifecycle selection and
independent layer sorting. Draft files do not introduce runtime enums or
claim that the importer already accepts these recipes. Round-three fixes
name the exact round-one assets they replace; they do not overwrite other
art worktrees.

## Alternatives considered

- Flatten rooms into one sprite: rejected because it loses per-layer depth
  sorting and duplicates adopted material pixels.
- Generate a separate ghost image for every building: rejected because the
  runtime already derives this display state from accepted built art.
- Treat failed numeric checks as visual exceptions: rejected; frozen family
  checks remain mandatory for new standalone generations.

## Consequences

Construction stages can be reproduced and reviewed without new runtime
code. The room helper has regression tests for cropped-course anchors,
missing dependencies, ghost edges and the protected-door damage defect.
Integration remains a separate C2/C3 task; a candidate passing art review is
not proof of a working game state selector.

## Date

2026-11-11

## Source

- Owner briefs: `/Users/rexxa/work/art-batch/BRIEF.md` and `BRIEF2.md`.
- A2 asset source: `227529c26307bc3a854eb7bf819e88d02af39046`.
- `art/batch_b3/PLAN.md`, `art/batch_b3/rooms/review.json`.
- Runtime ghost source: `4a44336e7782f2b27615d942b5cd54dbfee89406:scripts/world/art/site_ghost.gd`.
