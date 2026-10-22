# Layer contract v1

Draw order: shadow, body[frame], top[direction], hair[direction], beard,
headgear, item[frame, hand], carry[direction, shoulder]. Optional outline is
not used in Trial3: forced outlines and quantization are forbidden.

Each PNG uses the full template canvas and RGBA. Each has an equal-sized L-mode
mask: 0 empty, 1 skin, 2 hair, 3 cloth primary, 4 cloth secondary, 5 item.
Every nontransparent pixel requires a nonzero mask. Skin applies only to exposed
body; cloth region IDs only to tops. Base loincloth uses untinted region5 as a
worn item; skin region1 never covers the loincloth. Log the hand-coded mask.
Household tint affects 3, agent hair tint 2, skin tint 1.

An overlay is authored as the overlay itself, never obtained by subtracting
independently generated images. It is opaque throughout its interior. Explicit
mask-box extraction of a dressed figure and interior hole fill is permitted by
goal §3.4 only with the exact box and operation logged. Hole fill may never
repair missing limbs or join disconnected body pieces.

Top alpha is restricted to torso +/-4px, plus declared collar <=4px into head
and/or cloak hem <=12px below hip. Coverage >=95% of the torso. Composition
must reveal no underlying body through a top inside the torso box.

Items declare a grip; carry layers declare a shoulder pixel, in game pixels.
Placement translates this pixel onto the template anchor. Clipped pixels fail.
No rescaling, stretching or contour forcing to make generated art fit.

Per-layer gates: exact canvas, feet within2px, head box within3px, hand anchors
within4px, one 8-connected body/item silhouette, enclosed transparent holes
<=12 game-pixel square, top coverage, complete valid masks. Open leg gaps are
not holes. Per-set: fixed upper body, every frame/top/hair/item combination,
hole-free union, top occlusion, anchor alignment and no clipping.

Sidecars store schema_version1, id, kind, body_type, direction, frame, scale,
relative mask path, ordered operations, optional hands/grip/shoulder/extras.
Preserve generation prompt, references with SHA256, raw identity, tool result,
attempt index, elapsed minutes, processing and checker outcomes as provenance.
Only checker-passing assets enter normal boards; failures go to failure boards.
Style scores are reported after structure and do not replace it.

TRIAL3B amendment: `contact_ok` and `limb_ok` use the pixel definitions and
explicit sleep/overlay applicability in TEMPLATE.md. Optional `anatomical_mask`
is a relative L PNG path with anatomical labels, separate from material IDs.
A historical asset needs no new sidecar: template-window measurements run on
its existing alpha. No declaration of `contact_ok` or `limb_ok` is trusted.
