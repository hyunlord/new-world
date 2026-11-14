# ADR 0068: Correct set B deformation without regenerating accepted meshes

## Context

The M3D-2a planner accepted set B's game-size silhouette and texture direction.
The blocking defects are arm/shoulder collapse in work and unreadable resting
poses. New reconstruction would change an accepted surface without addressing
local bone-axis and skinning errors.

## Decision

Keep set B surface triangles, UVs and decoded atlas. Coincident seam welding
changes vertex indexing; roundtrip coordinate drift is allowed only below
1e-5 metres and measured independently against ordered triangle corners. Reweight the existing mesh with
seam-welded Blender automatic heat weights, anatomical locality and at most
three normalized influences, then reauthor the
two skeleton families' clips. Maintain existing exported bone and clip names. Fit one shared B rest template to the
measured chunky anatomy: the previous humanoid shoulder pivot was inside the
neck/upper torso, so weight filtering alone still produced armpit fans. Rebuild
inverse bind matrices while retaining the mesh and height-scaled family
proportions. This supersedes ADR0067's common A/B rest-proportion assumption for
B; it does not change set A. Add a shared `Tail` child of `Root` to both B
quadrupeds: hind-leg heat leakage and a rigid hanging tail otherwise support the
resting animal above the ground. The existing eleven bones retain their names
and hierarchy; the B quadruped family now has twelve bones. Keep set A
as the previous comparison arm and do not register trial content in the runtime.

Sample eight frames of every B clip at native 2x game projection and close up.
Inspect enlarged shoulder/elbow crops for work and keep before/after work/rest
sheets. Numerical weight and finite-coordinate checks supplement visual rejection;
they cannot certify the absence of visible pinching or self-intersection.

Preserve baseline files, failed attempts and full captures outside the repository.
The final Korean report records the acceptance result and any remaining limits.

## Alternatives considered

- Regenerate meshes on the DGX: rejected because the accepted style and topology
  are not the target of this correction, and reconstruction adds avoidable drift.
- Validate one representative frame: rejected because anticipation and
  follow-through can collapse even when the middle pose looks acceptable.
- Accept clean influence counts as sufficient: rejected because local axes,
  target pose and geometry determine visible deformation.

## Consequences

Shared procedural motions still require target-mesh resting alignment and
geometry-based hand/head clearance. The family rest skeleton is shared; final
curves are calibrated to each surface rather than promised byte-identical.
An oversized-head target needs a localized elbow core and lower work/sleep
hand targets to retain forearm volume and keep the face clear. Imported/exported
clips and engine imports must be checked, including hidden load scales. Existing
M3D-2a evidence remains historical rather than being overwritten.

## Date

2026-11-15

## Source

Owner-authorized `/Users/rexxa/work/3d-data/BRIEF_M3D2B.md` and
`/Users/rexxa/work/3d-data/m3d2-art/m3d2b/` evidence.
