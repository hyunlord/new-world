# C3A Pass4: ship stone through exact log-alpha stencils

## Context

BRIEF6 rejects Pass3: filling side quadrants through the full tile removes
small room interiors. Solid coverage alone was the wrong acceptance invariant.
This decision supersedes ADR0018's full connected-quadrant geometry.

## Decision

Use the pinned shipped log piece alpha byte for byte for all16 topology masks,
end/corner aliases and four wooden doors. Fill only wall-body RGB with the
Pass2 native course field, sampled in world coordinates, with restrained
edge ink and top-left shading. Preserve shipped contact/shadow pixels and
wooden doors. Never alter alpha to conceal inherited reference geometry.

Keep the existing wall-piece export entry point and additive import drafts.
Test every delivered piece's complete alpha, compare composed room coverage,
and inspect5x4,L,T+X rooms at1x/2x plus all26 joins at4x. Evidence is external.
No runtime import, generated geometry, rescaling or new dependencies.

The pinned NS doors are64x60 at64px tile stride and the shipped renderer uses
the same tile origin as walls. Their inherited4px bottom gap cannot be removed
while preserving exact alpha. Hold overall adoption rather than label the
literal no-background-at-any-join criterion satisfied. Wall-only joins and
reference-identical room floors are independently verifiable.

## Alternatives considered

- Keep Pass3 solid quadrants: rejected because small room interiors disappear.
- Reconstruct a similar thin polygon: rejected because exact alpha equality is
  simpler and prevents another topology interpretation from changing the room.
- Repair inherited log alpha holes: incompatible with the explicit exact-alpha
  requirement; report any reference limitation rather than silently modifying it.

## Consequences

Stone and log interiors have identical geometry, including antialias/contact
coverage. The unchanged native course remains visibly repetitive. Runtime
selection, mixed materials and construction states need later integration checks.

## Date

2026-11-14

## Source

BRIEF6.md; `art/batch_c3a/walls_pass2/`; shipped log pieces at
`d086d7c54d1b29421c9f7540f3a9358f0c054314`.
