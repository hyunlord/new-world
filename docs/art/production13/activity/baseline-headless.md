# Pre-redesign headless baseline

**HEADLESS_GEOMETRY_MEASURED; FRAMEBUFFER_ICON_MEASUREMENT_NOT_VERIFIED.** No GUI opened. Current exact-tick primary scenes were inspected through the installed CLI-Anything Godot harness and a synchronous SceneTree probe. Headless texture image data is available, including every existing icon. Ordinary rendering code is unchanged by this task.

Command (from project root):

```sh
GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot P13_BASELINE_OUT=/Users/rexxa/work/art-production13-evidence/activity-baseline-headless/raw.json /Users/rexxa/.local/bin/cli-anything-godot --json --project /Users/rexxa/work/new-world-art-prod script run scripts/test/art/probe_activity_baseline_p13.gd --timeout 240
```

The installed harness requires a relative script path: res:// was rejected as nonexistent; absolute path was incorrectly prefixed res://. Both failed launch records retained. Third relative-path invocation returned0, stderr empty, `P13_ACTIVITY_BASELINE_COMPLETE cases=3 textures=7`; raw JSON complete=true, display=headless. Explicit script sentinel is required because CLI wrapper return0 alone did not establish execution on the malformed absolute-path attempt. The synchronous probe finishes its work before wrapper --quit; it performs no asynchronous screenshot or rendered-frame wait.

Inspected HEAD `215bae332d647f38323a29ef00411d24dad3266f`. Raw source/bridge hashes recorded in raw.json. Cases: seed20261010 ticks41880(day30 10:00),42180(day30 15:00), seed20261018 tick85080(day60 10:00). All use zoom4, viewport1920×1080. Geometry below includes only in-tree-visible, viewport-intersecting rows whose icon id matches actual activity kind. Counts are observations across3 scenes, not distinct people or guaranteed unoccluded pictograms. Arrival metadata is retained in raw; some icons represent agents still walking to an activity.

| Kind | Matching in-view rows | Transformed texture dimensions px | Fill | Grass contrast | Dirt contrast | Roof contrast |
|---|---:|---|---|---:|---:|---:|
| TendFire | 4 | [(64.0, 64.0)] | #f39a43 | 3.122 | 1.898 | 1.202 |
| Craft | 8 | [(72.0, 72.0)] | #d87e33 | 2.281 | 1.387 | 1.645 |
| Repair | 1 | [(64.0, 64.0)] | #91bed9 | 3.469 | 2.109 | 1.082 |
| HearthCircle | 12 | [(64.0, 64.0)] | #d6a3e0 | 3.339 | 2.030 | 1.124 |
| Explore | 11 | [(64.0, 64.0)] | #98d47b | 3.951 | 2.402 | 1.053 |
| Play | 6 | [(64.0, 64.0)] | #f3d86b | 4.872 | 2.962 | 1.298 |
| SitByFire | 9 | [(64.0, 64.0)] | #79d5cd | 4.008 | 2.436 | 1.068 |

All generated source textures are16×16; Craft is18×18. Headless rectangle transforms measure64×64 or72×72. This verifies geometry, not actual framebuffer size/visibility/occlusion. Texture.get_image exposes source opaque bounds and color counts in raw.json. Six generated icons have source-designated main fill colors. Craft is multicolor; its orange#d87e33 is a representative material fill (12opaque pixels), not a falsely asserted unique main color. Full Craft palette is retained. Repair and Explore currently have no highlight pixels despite shared highlight capability.

## Actual baseline image background patches

Viewed existing P1.png directly; SHA256 `f778623c1fff62578cdd1db096e75cdac9b39d3f1398b1efa85558d781e84e79`,1920×1080, existing Z4 image from baseline. Rectangles are left/top inclusive, right/bottom exclusive:

| Patch | Rect xyxy | Pixels | Median per-pixel linear luminance | Visual justification |
|---|---|---:|---:|---|
| grass | [1200,300,1240,340] |1600|0.10235523619405569|Clear green field to the right of settlement, away from icons/structures/UI.|
| dirt | [1110,510,1150,550] |1600|0.2006391184103883|Exposed tan settlement ground to right of hearth and below eastern shelters.|
| shelter roof | [940,610,964,630] |480|0.5217417443223762|Visible light roof slope of isolated foreground shelter, below settlement crowd.|

These are explicitly selected visible representative patches, **not** per-icon hidden underlays and not a statistical proof they are the three most common backgrounds. Patch coordinates constitute the annotation; the original image is preserved unchanged. No pixels from a new headless screenshot are claimed. Contrast combines measured source fill RGB and actual pre-existing screenshot background median; it is a representative-patch contrast metric, not per-agent composited framebuffer contrast. Median is over per-pixel WCAG linear luminance; sRGB linearization0.04045/12.92/2.4; contrast `(Llighter+0.05)/(Ldarker+0.05)`.

## Evidence and limits

External directory `/Users/rexxa/work/art-production13-evidence/activity-baseline-headless/` contains raw.json, background-patches.json, metrics.json and three CLI transcripts. Raw JSON includes scene clock/info, icon geometry/activity identities, loaded bridge/source SHA256 and all seven texture histograms. User window prohibition prevents fresh framebuffer screenshot acceptance; headless data and older P1 samples do not replace StageA/StageB world screenshot evidence. No missing-kind coverage was invented.
