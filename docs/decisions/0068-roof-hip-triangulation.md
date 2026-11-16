# 0068 — Preserve roof hips across lattice triangles

## Context
The ROOF2 reference captures show repeated triangular notches in both summer and winter roofs. Both seasons use the same CPU mesh; snow only changes the roof shader colour. Limiting hip rectangles did not prevent a lattice square from being split across its lower diagonal, cutting a repeated notch into an otherwise straight hip.

## Decision
Split each roof square along the diagonal with the greater sum of endpoint heights. Keep the existing at-most-two hip rectangles and flat caps. Rooms wholly enclosed by the same template site share one roof, including internal partition walls; map room selection to that roof so cutaway remains available. Keep independent buildings separate.

Test the actual generated triangle normals and ridge sections rather than only counting selected rectangles. Test summer/winter vertex equality and shared building ownership. Add these checks to the existing headless 3D gate.

The reported black-body/white-hood person is an adult with grey hair: rays to the body hit a roof, while rays to the head do not. Keep the painted atlas in the textured-person occlusion pass instead of replacing it with a dark silhouette; untextured fallback and visibility alpha remain unchanged.

## Alternatives considered
- Separate winter geometry: rejected because snow uses the same mesh.
- More per-room rectangles: rejected because it preserves separate room roofs and does not repair triangulation.
- Replace the roof renderer: rejected because the defect is local to triangulation and ownership.

## Consequences
Presentation only; simulation snapshots and commands remain unchanged. The roof of a multi-room building cuts away as a whole. Full publication gating and moving v8/main are outside this branch's scope.

## Date
2026-11-16

## Source
`/Users/rexxa/work/art-img-data/BRIEF_ROOF2.md`; `scripts/world3d/structures3d.gd`; `tools/v8/roof_geometry_test.gd`.
