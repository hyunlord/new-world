# Set B characters: source copies

Byte copies of the accepted set B characters, made once by hand; `tools/3d/blender/chars_b.py` bakes them into
`assets/3d/vat/b_<subject>_*`. Do not edit these files; re-copy them from the source when the art changes.

* Source: `/Users/rexxa/work/art-3d-chars/assets/3d/chars/B/<subject>/` (worktree of branch `v8/art-3d-chars`)
* Commit: `f0b91f999` (f0b91f999e96c9095e32b9d649a835b40e37f6e7, "Keep accepted B characters readable through
  work and rest"); the copied files match that commit (no uncommitted changes at copy time).
* Copied per subject (`man woman child goat aurochs`): `<subject>.glb` (skinned, clips embedded, atlas embedded),
  `atlas.png` (512 x 512), `receipt.json`. Not copied: contact sheets, motion review images, Godot `.import` files.
* Humans: 17 bones, clips `idle walk work carry sleep`, plus a `CarryLoad` mesh scaled to ~0 outside `carry`.
  Animals: 12 bones, clips `idle walk graze lie_down`. Scale as authored: man 1.7 m, woman 1.7 m, child 1.2 m.
