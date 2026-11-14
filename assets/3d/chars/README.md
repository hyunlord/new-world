# M3D-2a character reconstruction trial

Set A uses the original semi-realistic concepts. Set B uses chunky stylized
concepts matched to the M3D-1 world reference. Inputs and turnarounds are in
`concepts/` and `concepts_b/`; turnarounds are references, never reconstruction
inputs. Final trial assets are in `A/<subject>/` and `B/<subject>/`.

Each final directory contains a textured GLB, one 512-pixel atlas, a four-view
and per-clip contact sheet, and a receipt. Human clips are idle, walk, work,
carry and sleep; quadrupeds use idle, walk, graze and lie_down. The human carry
package uses the same atlas and appears only in the carry animation. Diagnostic
thumbnail rows in contact sheets are labelled as thumbnails. Native game-size
A/B/M3D-1 comparisons are in `comparisons/<subject>_sheet.png` with matching receipts.

The two skeleton families share bone names, hierarchy and proportions. Rest
lengths scale with subject size and object transforms are applied; cross-height
motion transfer scales root displacement and recalibrates resting-pose grounding
to the target geometry. Front is Godot +Z (Blender -Y), units
are metres, and origin is at the ground. Human height is measured at the crown;
quadrupeds use the documented front-trunk withers landmark, excluding horns.

These are isolated trial assets, not registered runtime content. Primitive
motions, remeshed hands/feet and texture smoothing still need production review.
The Korean recommendation and all failed attempts, raw GLBs, full viewports and
logs remain under `/Users/rexxa/work/3d-data/m3d2-art/REPORT.md` and its evidence
directory. No credentials or raw reconstruction dumps belong in this tree.

Reproduction and limitations: `tools/3d/chars/dgx/README.md`,
`tools/3d/chars/blender/README.md`, ADR0066 and ADR0067.
