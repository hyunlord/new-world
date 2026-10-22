# B model build — candidate 1

2026-10-22 owner amendment: no payment. Candidate 1 uses Kenney Mini Characters
`character-male-a.glb`, CC0 1.0; no login, generation, or Mixamo asset used.
Source: <https://kenney.nl/assets/mini-characters>.
License: <https://creativecommons.org/publicdomain/zero/1.0/>.
Archive `License.txt` SHA256:
`28358ae5accc85b572eb42507956afc8beae05acb4648bb9026a5714d421b785`.
Exact source GLB SHA and all original counts are in external
`/Users/rexxa/work/arttrial3-data/b/model/model_evidence.json`.

## Geometry and interface

The actual source body/head polygons and seven-bone rig are retained, not replaced
with a procedural body. Head dark hair and blue-grey glasses polygons are removed
by their original atlas colors; brown head skin polygons remain. The source body
surface is sculpted by anatomical weight group and recolored neutral grey.
This retains clothing-shaped source geometry; residual cuffs/seams are a known
visual limitation, not a claimed anatomical restoration. Newly authored meshes
are only loincloth, two tops, two hair variants, axe with grip empty, and logs.

Source height 0.671325; source head 0.328075 (48.87%). Adapted sole=0, top=1,
head height=0.30. Idle evaluated body bounds: x ±0.375217,
y −0.137813..0.129177, z 0..1. Forward −Y, up +Z; broad torso is about 0.50
world units wide. These are world dimensions; the renderer's actual pixel
measurements determine contact/limb acceptance.

`B_Rig` bones: root, leg-left, leg-right, torso, arm-left, arm-right, head.
Collections and object `layer` properties: BODY, TOP_hide_wrap, TOP_fur_cloak,
HAIR_short, HAIR_long_tied, ITEM_stone_axe, CARRY_log_bundle.
Materials carry pass_index: skin=1, hair=2, hide primary=3, fur secondary=4,
item=5. Loincloth belongs to BODY with region 5 as the layer contract requires.

Original mesh polygons are partitioned by source skin weights and source-space
wrist/ankle boundaries. `anatomical_id` maps 1=head, 2/3=left/right hands,
4/5=feet, 6/7=arms, 8/9=legs, 10=torso. Each partition records source mesh and
retained polygon count; no anatomy is painted into an output mask. Loincloth
is unmeasured torso region 10. All ten original partitions retain rig weights.

## Authored animation

| Action | Distinct key times | Render extraction |
|---|---|---|
| B_idle | 1, 13, 25 | idle=1 |
| B_walk | 1, 7, 13, 19, 25 | walk_a=1, walk_b=13 |
| B_two_hand_chop | 1, 7, 13, 19, 25 | work_a=1, work_b=13 |
| B_carry_walk | 1, 7, 13, 19, 25 | available separately |
| B_sleep | 1 | sleep=1 |

All are explicitly authored Blender pose keys, no external clips. Counts mean
**distinct timeline key times**, not one scalar F-curve point: every keyed pose
contains the bones' quaternion/location channels. The retained seven-bone rig
has no elbow or knee articulation. Sleep rotates the root so the head is left;
renderer owns framing and visibility. Animation readability remains a GIF gate.

## Files and checks

Build: Blender 4.5.14 LTS `-b --python art/pipeline3/build_b_model.py`.
The builder imports `b_model_source.py`, `b_model_wardrobe.py`, `b_model_clips.py`.
Outputs outside the repository: `b/model/model.blend`, `model.glb`,
`model_evidence.json`, `model_validation.json`.

Verified by actual Blender reload: seven bones, all ten anatomy partitions,
seven layer collections, five animation actions, and source mesh skin weights.
Exported GLB reimported successfully with one seven-bone rig and five actions.
Python compilation and repository Ruff checks pass. No still renders or look
setups were consumed by this builder; render/style/contact/limb/playback gates
are separate and have not been inferred from the mesh validation. Credits=0.
