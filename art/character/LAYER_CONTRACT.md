# Layer contract v2 — pawn

TRIAL4 sidecars declare `template_version: 2`. Historical v1 sidecars and A/B
assets keep their original rules; the v1 document is preserved in git at
`7c767aae:art/character/LAYER_CONTRACT.md`.

Draw order: lab shadow, BODY[frame], TOP[idle,direction], HAIR[idle,direction],
ITEM[pose,direction]. No limb layers, hand anchors, or limb_ok for pawns.
Every layer uses the full template canvas and RGBA plus an equal-sized L mask:
0 empty,1 skin,2 hair,3 cloth primary,4 cloth secondary,5 item. Every nonzero
alpha pixel has a valid nonzero material mask; empty pixels have mask0.
Skin tint applies1, hair tint2, household/cloth tints3/4. Shadow is lab-only.

BODY includes head, neck connector, capsule and two foot wedges. Tops cover
its upper70%; hair uses the head box. Overlays are rasterized from idle only,
then registered to each body bob/lean/rotation. They must fit every frame.
ITEM grip is S(46,64), E(44,62), N(18,64). Equipped item alpha must overlap BODY
by at least2px; contact is pixel evidence, not a declared anchor. Carry logs
span shoulders at y56. Sleep rotates all layers and masks together head-left.

The optional `none` layer is explicitly empty, with an empty mask and its
absence declared by its variant; it is not evidence of a valid nonempty body.

Per-layer gates: exact canvas, complete mask, connectivity, enclosed holes
<=12px², upright BODY contact at opaque y86..88. Per-set: inventory closure,
registered fixed upper body, top coverage/occlusion, all frame fits, item
contact and no clipping. Shape/ink output is never repaired by hand.

Sidecars retain schema_version, id, kind, body_type, direction, frame, scale,
relative mask path and operations, with template_version2 and look metadata
(sex, age/body_type, skin_tone, hair_style, hair_color, top, top_palette, item),
anchors and spec hash. The generator preserves deterministic provenance.
Folders follow the existing BODY/HAIR/TOP/ITEM layout under route_p; ordinary
boards and the normal lab admit checker-passing assets only. Rejected images
remain in separately labelled diagnostics. Style scoring never replaces
structural or fresh-reviewer readability acceptance.
