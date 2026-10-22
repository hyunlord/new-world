# Standalone person composition contract

`art/proto/PersonComposer.gd` implements a Sprite2D cut-out rig under a real Skeleton2D/Bone2D hierarchy. It reads generated PNGs only; missing parts are explicit errors, never replaced by procedural body shapes. This rehearsal contains no simulation or game bridge.

## Inputs

Set exported `asset_root` before adding the composer to the tree, then call `compose(attrs: Dictionary)`. Defaults resolve to `art/style05/people`; external candidates may use `/Users/rexxa/work/arttrial2-data/people`. The loader accepts flat PNGs or `parts/` and `overlays/` subdirectories. Candidate availability is not style acceptance.

Required body PNGs: `head`, `torso`, `upper_arm_l/r`, `forearm_l/r`, `hand_l/r`, `upper_leg_l/r`, `lower_leg_l/r`, `foot_l/r`. Required overlays: `top_hide_wrap`, `top_fur_cloak`, `top_woven_tunic`, `hair_short`, `hair_long_tied`, `hair_braided`, `held_stone_axe`, `held_log_bundle`, `held_berry_basket`, `hat`, `stick`. Every basename ends in `.png`.

Sprites preserve source aspect ratio and fit their nominal body bounds. Normalized extraction pivots are explicit in `PIVOTS`: head `(0.5,0.65)`, torso `(0.5,0.15)`, upper arms `(0.35/0.65,0.12)`, forearms and upper legs `(0.4/0.6,0.09)`, hands `(0.43/0.57,0.1)`, lower legs and feet `(0.5,0.09)`. Image origin is top-left. Shoulder/hip anchors and child bone offsets are in working pixels; they must be inspected against the actual extracted sprites before visual acceptance. The design spans approximately 56 px crown to foot at adult scale; animations can widen or shorten the visible silhouette.

## Attribute mapping

| Attribute | Values and effect |
|---|---|
| `age` | adult scale1; child scale0.75 and head local scale0.9; elder torso angle−8° and generated stick visible |
| `skin` | HTML colour passed to the neutral-mask shader, matching the flat `#B0B0B0` body regions |
| `top` | `hide_wrap`, `fur_cloak`, `woven_tunic`; one generated overlay visible on torso |
| `top_tint` | Household HTML colour replaces near-neutral `#C0C0C0` regions only |
| `hair` | `short`, `long_tied`, `braided`; one head overlay visible |
| `held` | `none`, `stone_axe`, `log_bundle`, `berry_basket`; axe/basket follow hands, bundle follows shoulder |
| `hat` | Boolean; generated hat follows head |
| `state` | `idle`, `walk`, `chop`, `carry`, `sleep`, `knap` |
| `phase` | Initial animation time offset in seconds |
| `sex` | Original alternating female/male answer-key attribute retained with `sex_rendered=false`; single adult body does not encode sex, so the20/24 requirement remains and cannot be waived by relabelling |

The shader isolates near-neutral colour by channel spread and intensity distance, preserving dark ink and coloured material. It does not recolour whole sprites with a global skin `modulate`. Mask correctness requires image inspection; a parsed shader proves no visual result.

## Animation and review

AnimationPlayer owns all six loops. Walk/carry use 8 sampled intervals; chop uses12. Idle scales torso y for breathing; walk swings limbs; chop solves both8+7px arm chains to adjacent grips3px apart at all12keys; carry raises arms while legs walk; sleep rotates the torso subtree90° and is static; knap folds legs and moves hands. Elders retain the torso offset in every animation.

The `people` harness selects12 figures with exactly2 sleepers,2 children and1 elder on meadow or forest_floor. It writes the attribute answer key outside the repo; blind reviewers receive captures alone, ordered left-to-right then top-to-bottom. Do not send reviewers this spec or answer_key.json. Runtime animation samples log actual Bone2D rotations/scales and current player positions at four times. Those observations and visible captures, not the animation-name list, establish playback.

Two fresh reviewers completed all four image conditions; every readability bar failed. Verbatim answers and scores are in docs/v8/arttrial2/READABILITY.md and READABILITY.json. Preserve the original thresholds: ≥10/12 at2×, ≥8/12 at1× and ≥20/24 attributes. No programmatic identity or answer key can substitute for recognition.

Post-review correction: forearm sprites render above torso overlays so shared-grip motion is not occluded by clothing. `chop_grips.json` records both actual global hand positions at13sampled keys including the closing pose. Final matrices use separate `-v2` evidence paths; initial captures and answer keys remain preserved.
