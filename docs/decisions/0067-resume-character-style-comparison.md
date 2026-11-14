# ADR 0067 — Resume the character trial with a licensed RGBA path

## Context

The first M3D-2a run stopped before reconstruction. The planner subsequently
confirmed that the owner's existing Hugging Face account already has DINOv3
access and explicitly authorized a single token-file copy into the DGX trial
workspace. The reference world also requires a chunkier style than set A.

## Decision

Keep ADR0066's isolated trial and safety boundaries. The resume brief permits
one credential copy from the owner's existing cache into the dedicated
workspace, mode 0600. Read it silently inside the container; never put its value
in command arguments, configuration, logs, reports or git.

Use MIT BiRefNet to produce nonopaque RGBA inputs. Configure TRELLIS with no
background-remover instance and minimally guard its constructor; verify its
RGBA preprocessing bypass. Do not download or use RMBG-2.0. Pin model revisions.
Keep set A and add chunky low-poly set B for the same five subjects.

Skip UniRig under the resume instruction and use Blender automatic weights.
Supersede ADR0066's normalized object-scale detail: apply armature transforms,
keep common bone names, hierarchy and proportions, and scale translation curves
when transferring a clip between different heights. Resting poses also require
target-mesh grounding; a real adult-to-child transfer exposed a 4.33cm drift
that height scaling alone does not correct. Compare actual reconstructed
A/B meshes against read-only M3D-1 placeholders using its 30-degree FOV,
50-degree pitch and native 64/128-pixel ground-metre projection at 1x/2x.

Actual imported seam topology must be repaired before automatic weights. Weld
coincident geometry while preserving loop UVs and remove loose geometry. If
nonmanifold geometry prevents decimation from meeting the requested budget,
use the brief-permitted remesh and bake from a preserved source surface. A
count-passing decimation can still collapse the silhouette; explicitly force the
same remesh path after comparing raw and reduced views. Narrow the withers
selection to the body centreline when swept horns cross the broader band, and
inspect the actual surface marker in side view. Record
cleanup retries independently from reconstruction attempts. The human carry load
shares the atlas; retain constant object animation channels and verify binary
scale values for visibility in each exported clip.

## Alternatives considered

- Use RMBG-2.0: rejected by the owner-approved resume brief and its licence.
- Substitute DINO conditioning: rejected; authorized original weights exist.
- Continue solving aarch64 bpy for UniRig: rejected by the explicit Mac fallback.
- Retain a height scale on armatures: rejected to meet applied transforms.
- Compare resized thumbnails: rejected because native game-size readability is
  the acceptance question.

## Consequences

Authentication is no longer a reported licence/access blocker. Generation,
cleanup and deformation still require actual per-subject evidence; a successful
sample or synthetic test cannot stand in for those outputs. Failed attempts
remain in the external evidence directory. Runtime integration remains separate.

## Date

2026-11-15.

## Source

Owner-approved `/Users/rexxa/work/3d-data/BRIEF_M3D2_CHARS_RESUME.md`;
`tools/3d/chars/dgx/prepare_pipeline.py`;
`tools/3d/chars/dgx/prepare_rgba.py`;
`tools/3d/chars/blender/compare.py`;
external `/Users/rexxa/work/3d-data/m3d2-art/REPORT.md`.

Godot import exposed singular bases from a zero-scale hidden carry attachment.
Use scale 1e-6 for default/non-carry states and 1 for carry; assert exported samples
and require a clean second import after the fresh-tree font import.
