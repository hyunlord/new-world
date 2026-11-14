# C3A stone wall stencils — Pass 4

Pass 4 supersedes the Pass 2/3 geometry policy. Pass 3's full-tile south
junction blocks removed small-room floor; that policy and its tests are removed.
Each stone mask now copies the exact shipped log alpha from
`d086d7c54d1b29421c9f7540f3a9358f0c054314` without filling any extra pixels.

The unchanged Pass 2 native64×32 crop from `drystone/face.png` repeats in world
coordinates `(tile*64 + local) % texture_size`. Both periods divide64, so each
exported tile has identical phase at every integer world tile, including negative
coordinates. One-pixel exposed edges receive RGB-only shade, lighter at north/west
than south/east. Source alpha below96 (including contact shadows) retains RGBA.
No scale, blur or image generation creates production geometry.

The pinned source has16 masks and4 oriented64×60 doors; no separately named
corner/end cap PNG exists there. Eight cap exports explicitly reference the
corresponding shipped mask stencil, with decoded pixel/alpha parity tested.
Door moving leaves/hardware keep shipped RGBA; only hand-inspected fixed frame
regions use stone RGB. Every exported alpha and canvas size remains exact.
Frame overlays and the complete source ledger live in external Pass4 evidence.

Required5×4,L,T+X rooms compare stone and shipped logs at1×/nearest2×, with every
corner/T/X instance shown nearest4×. Exact source alpha preserves identical floor
shapes, including source imperfections:64×60 NS doors leave4px before the next
64px-stride wall and some shipped horizontal edges have different alpha spans.
These inherited gaps must not be described as gap-free or silently filled.

```sh
uv run --with pillow --with numpy --with pydantic python -m tools.v8.compose_wall_pieces --native art/materials/textures/drystone/face.png <new-directory>
uv run --with pytest --with pillow --with numpy --with pydantic python -m pytest art/batch_c3a/walls_pass2 tools/v8/test_compose_wall_pieces.py -q
```

Final reviewed files are in `outputs/`, with reproducible external copies under
`/Users/rexxa/work/art-batch/C3A/pass4/final_held/`. Status is
`HOLD_REFERENCE_DOOR_GAP_NOT_IMPORTED`: wall-only geometry passed review, but the
literal doorway no-gap criterion remains held (independent final score88).
All28 exported alpha channels match their pinned sources with zero differing
pixels;39 scoped exporter tests pass, with Ruff/format and type checks clean.
No approved export was used;
no assets are imported and runtime rows remain empty.
The importer/selector drafts preserve every unrelated material/default.
Runtime occupancy, mixed materials, chunk joins, tint, construction and ghosts
remain integration work. No Rust, Godot runtime or other art is changed here.

Held art source commit: `5aa67f48ce1a7fcf47c1cd5531110a51a91448dc`. This pin is reproducibility, not approval.
