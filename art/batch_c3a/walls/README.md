# C3A stone wall pieces

These are deterministic compositions of the shipped `drystone`/`granite` material kit, not new image generations. `manifest.json` preserves source and output SHA256, N=1/E=2/S=4/W=8 topology, canvas64×64, anchor(32,59), image origin(0,−16), and world base(32,43). The source kit and compositor are already present at art-tree `f182769fe06b1cb60105e4d0bf127b6524277838`.

Generate into a **new** directory:

```sh
uv run --with pillow --with numpy --with pydantic python -m tools.v8.compose_wall_pieces <new-output-directory>
```

The 16 mask images, four end caps, four corner caps and EW open/closed doors use existing pixels. Bottom padding is transparent. The corner/end caps are byte-identical aliases of their respective masks. NS door artwork is absent; it is explicitly null rather than a rotated EW drawing.

The 2× sheet shows every mask; `assembly_5x4_1x.png` shows the unchanged legacy material compositor on a wall ring. The legacy style uses a broad64px top and blurred top material: south-connected masks fill most of a tile, and this is not W1's thin interlocking visual language. These limitations are visible rather than hidden by rescaling or new ink. No independent visual style pass or runtime acceptance is claimed.

`manifest.json.capability` is a **disabled proposed** `wall_pieces` registration, with `runtime_rows_to_append=[]`. The named composer requested by BRIEF3 was absent from art-tree, published W2 (`dd7b500dd`) and current main (`c1aeb3585`); this thin exporter supplies the art-side operation using existing functions. It does not add Godot schema/resolver/baker support. Integration must verify occupancy including doors and other materials, anchors, corner joins, chunk boundaries and site-ghost parity before enabling the capability.

No construction progress or damage threshold is invented. B3 room-stage recipes remain separate layer compositions, not single painted building sprites. Held candidate export counts:16 mask PNGs +8 cap aliases +2 EW door PNGs; sheets are evidence, not additional adopted art.

The separate `v8/visual-v3` importer at `87c67b1` already contains a log-only wall_pieces schema splice and local 20-file composition hook. That runtime capability does not validate these drystone candidates. Their visual failure remains blocking.
