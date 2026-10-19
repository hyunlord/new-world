# H10 current-build capture protocol

The new `scripts/test/art/capture_h10_p13b.gd` subclasses the unchanged13 Life tool. Its sole responsibility is H10 live-world evidence. WorldSim/Godot skills were read. No new localization keys or product changes. No Python runner was required.

Run after source freeze, one case per fresh output, with window lock and the inherited quiet-lock wait. Example for `road`; run independently for `relations` and `chronicle` so one failed requirement does not prevent collecting another:

```bash
P13B_H10_CASE=road \
P13B_H10_OUT=/Users/rexxa/work/art-production13b-evidence/h10-road-attempt1 \
P13B_H10_BUILD_PROFILE=release \
P13B_H10_TICK=122430 \
bash tools/art_pipeline/with_window_lock.sh 1800 bash -c \
'while [[ -d /Users/rexxa/work/.mac_quiet.lock ]]; do sleep 30; done; exec "$@"' \
h10 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
--path /Users/rexxa/work/new-world-art-prod --fixed-fps 60 --resolution 1920x1080 \
--script res://scripts/test/art/capture_h10_p13b.gd
```

Capture stdout/stderr outside the fresh output directory. Do not pre-create `P13B_H10_OUT`. An existing directory is refused without writing to it. The outer window-lock wait must be supervised by the leader's deadline; its command limit starts after acquiring the window lock.

- Seed is fixed19603, actual game_v26 main scene and reset. Default tick122430 (display day86 08:30),30 ticks after the historical kind21 and porter departures. Override tick is positive and strictly below172320 (display day121 starts there). Tick selection does not inject data or reorder sprites.
- `road`: actual outbound porter, camera centred on it at Z4, EN/KO, body/glyph frame evidence, expected `p12_porter_food` or `p12_porter_wood`, actual carry/carry_wood pose. Metadata assertions do not replace visual road-position/pose review.
- `relations`: Z1, camera centred on actual formation anchors, EN/KO; requires two distinct relation colours with each counted line's endpoints onscreen. On-frame relation metadata and native screenshots retained. Metadata is not framebuffer/occlusion proof.
- `chronicle`: requires actual kind21, captures every real chronicle panel page in EN/KO, then checks its localized sentence is in the rendered text. Inspect the saved page containing that sentence for visible, readable made-peace text; rendered text alone is not visual acceptance.
- Inherits exact frozen tick assertion, complete world/history references, source/font/atlas/bridge hashes and PNG hashes. Additional HEAD check before each image. Explicit release profile requires release-loaded SHA equality. Failures retain PNGs/sidecars plus `failure.json` with references. Success produces `reference-data.json` with `visual_acceptance: NOT_VERIFIED` until independently reviewed.
- All cases are real world captures. No fake bridge or fabricated history fallback exists in this tool. Three-strike accounting remains with leader, including debug launches.

Validation at handoff: source inspection and `git diff --check`; no Godot execution or parser/runtime claim. The leader must run parser/windowed capture checks. Source history does not guarantee current-build reproduction; absent outbound porter/two-colour lines/reconciliation cause explicit failure. Native screenshots require final visual review. No blind-reader answers or keys were inspected.
