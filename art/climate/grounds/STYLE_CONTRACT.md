# B2 climate ground contract

Candidate art only; 64x64 opaque ground tiles, anchor (32,43), footprint (1,1).
Ground is straight overhead, diffuse upper-left lighting. Two locked originals
ground/scene plus actual imported art/style05/env/ground textures guide the set.
Historical ground_v2 is a failed candidate family, not an adopted reference.

Three main material clusters per tile, fine brown-black primary/detail ink 2:1,
three selective parallel / marks per cluster. These are prompt/visual targets,
not invented automatic stroke counts. Broad earth/slate/sage washes support
recognizable material, with standing flora stronger than ground at 1x.

Native generated pixels are full-frame Lanczos reduced to64x64. Existing ground
tone cap may reduce global contrast/chroma with measured before/after. Existing
mirror_blend only after two seam failures; all such use disclosed. No painted
replacement, forced contours or palette quantization.

Seams use existing measure: opposite-edge RGB MAE<=18, ratio<=2 in both axes.
Original-art tone: luminance std<=0.6x foreground person; saturation<=0.35.
Current runtime darker rendering is a separate cap, not pre-applied to masters.
Independent visual review and frozen numeric family style diagnostics remain
separate; report every failure. Crop and transparent-sprite checks do not apply
to opaque full-frame ground.

Eight material masters supply four season exports through the existing ground
renderer: spring shares summer, autumn uses the existing drought tint, winter
uses existing chroma35% reduction. These are disclosed derived exports, not new
generations; no new snow artwork is claimed. Climate memberships
are art proposals; no runtime band/season selector is implemented here. All
grounds are era-neutral natural environment available from paleolithic onward.
Mild Mediterranean winter remains moist/green without universal snow. Harsh
tundra/taiga winter snow and neutral meadow frost follow existing season intent.

Edges reuse art/compositor/edges.py world-seeded sampling (seed5005), 12px ground
blend, 10px water falloff. Textures are not rotated. Edge files are deterministic
derived recipes/exports, never counted as newly generated artwork.
