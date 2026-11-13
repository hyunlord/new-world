# C3A pass2 native stone walls

The pass1 atlas stays held. This pass replaces its blurred broad top with one
sharp native stone-course field used on every cap and front, with the shipped
W2 geometry:64px tile,26px wall width,16px cap,22px front, anchor(32,59).
There is no upscaling, blur, recolouring or seam blend in any production image.
Only review boards are enlarged2x, by nearest-neighbour.

No new image generation was needed. `art/materials/textures/drystone/face.png`
is the unchanged sharp256x44 source from baseline f182769fe, SHA256
517c4f431303d3ae01362fb451a6757f77cc18797875760c0eabae945c73c682.
The exporter evaluates all2509 possible native64x32 crops, chooses the minimum
worst edge RGB difference (ties:sum,y,x), and repeats that crop twice vertically.
Selected crop origin(48,9), size64x32; horizontal/vertical mean edge RGB jumps
30.78125/32.96875. The motif repeats and the pixels are not mathematically
seamless. These nonzero measurements remain visible in the manifest; no seam
threshold was changed. Root and independent review found no visible64px grid
join or density break in the required room comparisons and accepted the art.

The W2 implementation was read from the external, untracked art-import-data
composer. Its shipped reference outputs are independently pinned to full commit
d086d7c54d1b29421c9f7540f3a9358f0c054314, under
assets/v8art/materials/textures/log/pieces/. The new code preserves the W2 thin
geometry and exposed2px ink/3-row shadow, but uses the same native stone field
for all surfaces instead of separate face/end patches. The four EW/NS wooden
doors are unchanged shipped pixels: reusing a wooden door in a stone wall does
not require regenerating or rotating a new door. No other worktree was edited.

`outputs/` contains16 masks,4 end caps,4 corner caps and4 doors =28 importable
wall-piece files; caps are exact aliases of their mask. The native course field
and four review boards make33 PNGs total. `rooms_1x.png` and `rooms_2x.png` show
5x4 and L plans next to the exact shipped log pieces, including closed/open EW
and NS doors. `contact_1x.png`/2x show every mask and door. Review accepted images
were byte-identical after the final exporter/metadata pass.

```sh
# New destination only; default remains pending visual review.
uv run --with pillow --with numpy --with pydantic python -m tools.v8.compose_wall_pieces --native art/materials/textures/drystone/face.png <new-directory>
# Only after independent review of unchanged PNG hashes, --native-approved
# records the already-made art decision; it does not import or enable runtime.
uv run --with pytest --with pillow --with numpy --with pydantic python -m pytest art/batch_c3a/walls_pass2/test_native.py tools/v8/test_compose_wall_pieces.py -q
```

`outputs/art_import.draft.json` has28 additive wall_piece_texture rows with a
source-commit placeholder for the leader to pin after the asset commit.
`construction_styles.patch.json` proposes only drystone.wall_pieces and preserves
all defaults/material looks. Paths are the imported C3A source paths resolved
through the visual-v3 ArtData.path_of_source contract, not guessed res:// paths.
Caps are references, never extra overlays. Runtime rows remain empty and no game
configuration is applied. Integrators must verify occupancy including doors and
mixed materials, chunks, tinting, construction stages and site ghosts.

Checks:6 Python tests (native1:1 pixels, topology/thickness, actual CLI export and
non-overwrite), Ruff/format, basedpyright and export/hash checks. Raw diagnostics,
all crop phases, source/door ledgers and visual verdicts live externally under
/Users/rexxa/work/art-batch/C3A/pass2/. Existing workshop room shells still use
the pass1 legacy material renderer; this atlas does not silently replace them.
