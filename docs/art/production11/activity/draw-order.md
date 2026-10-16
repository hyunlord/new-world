# Complete world Sprite2D blocker ordering, pre-crop revision

External files only. Install these together for NEW capture attempt3; attempt2 remains rejected for occlusion eligibility even if its full screenshots and pose checks are useful:

- scripts/test/art/capture_activities_p11.gd (includes main's P10 oracle environment fix)
- scripts/test/art/capture_sprite_order_p11.gd (new helper)
- scripts/test/art/test_capture_sprite_order_p11.gd (new scheduled synthetic test)
- tools/art_pipeline/plan_activity_blind_p11.py (requires complete ordering schema2)

No pose logic, first-two protocol, crop geometry or threshold changed.

## Source-grounded order

Godot4.6 official source revision `3b853ac6f016102b16e5056e2c2f33971f228892` was resolved from refs/heads/4.6. Downloaded branch source equals pinned source by cmp.

- [Z buckets concatenate ascending](https://github.com/godotengine/godot/blob/3b853ac6f016102b16e5056e2c2f33971f228892/servers/rendering/renderer_canvas_cull.cpp#L78).
- [Y-sort child transforms, snapping, index, inherited z](https://github.com/godotengine/godot/blob/3b853ac6f016102b16e5056e2c2f33971f228892/servers/rendering/renderer_canvas_cull.cpp#L98).
- [Effective relative/absolute z, flattened y-sort, behind/body/front recursion](https://github.com/godotengine/godot/blob/3b853ac6f016102b16e5056e2c2f33971f228892/servers/rendering/renderer_canvas_cull.cpp#L430).
- [Approximate equal y uses traversal index](https://github.com/godotengine/godot/blob/3b853ac6f016102b16e5056e2c2f33971f228892/servers/rendering/renderer_canvas_cull.h#L115).

WorldObjects top groups use snapped local y then sibling order. Within each unsorted group, recurse behind children, body, front children. Sort resulting records by clamped effective inherited/absolute z then traversal order. Every visible textured Sprite2D under WorldObjects is included irrespective of art_kind: trees, rocks, props, construction, all body descendants including icons, CarryLoad, cold/settlement outlines and ash. Rows store node/parent path, top group, effective z, traversal index and final draw_rank. Body holder looks up its rank by node path. Strictly higher ranks block; own behind child at equal z does not. A behind child with higher z still draws later, as the engine z buckets require.

Fail closed (draw_order_complete false + capture errors) for nested y-sort, CanvasGroup, top-level nodes, clipping, non-default visibility layer, unsupported non-Sprite drawable, non-Node2D world descendants, physics interpolation. These unsupported states are not guessed. Current static world graph uses supported direct Sprite2D groups and Sprite2D descendants; runtime must confirm it.

Rectangles remain get_global_transform_with_canvas()*get_rect(), matching the existing holder sidecar and original crop logic. This is transformed sprite-rectangle coverage, not alpha-mask coverage; renderer snap can change pixel raster boundaries. UI Control/CanvasLayer overlays and non-world overlays are outside WorldObjects Sprite2D scope and have NOT_VERIFIED overlap status; no new eligibility filter was invented. Existing margin/frame checks remain unchanged.

## Checks

- `python3 .../capture-prep/tools/art_pipeline/test_activity_protocol_p11.py`:5/5 PASS (includes exact original geometry25 cases and rectangle union boundary checks).
- `python3 .../capture-prep/tools/art_pipeline/p11_activity_oracle/test_activity_oracle.py`:17/17 PASS.
- `python3 -m compileall -q .../capture-prep/tools/art_pipeline`:PASS.
- `cmp godot46_renderer_canvas_cull.cpp godot46_pinned_canvas_cull.cpp`:exit0.

Synthetic Godot test is WRITTEN, NOT_RUN due quiet-lock/no-runtime instruction. It expects Absolute-z0, Behind, Body, Icon, Tree, ElevatedBehind-z5 order; verifies behind child below its body, own icon above, foreground tree blocking group, and nested-y-sort refusal. Schedule after import: `$GODOT_BIN --headless --path CHECKOUT --script res://scripts/test/art/test_capture_sprite_order_p11.gd` (through leader scheduling rules). No actual runtime validation claim.

## Leader verification after installation

Synthetic Godot ordering test exited0 with no ERROR lines; strict check50files and6FFI bindings PASS. Main Python protocol6/6 and original pose oracle17/17 PASS. No blind crops made before this change.

Static hierarchy audit found no known unsupported WorldObjects nodes: agent/building/nature/effect roots and their rendered children are Sprite2D. Other line/canvas effects are outside this subtree; UI overlap remains unverified. Village summary quota identity includes seed plus village ID because IDs are world-local; synthetic test preserves maximum two pairs per village across independent worlds.
