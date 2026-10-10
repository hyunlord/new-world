# Production atlas runtime review

Read-only review of the 272-entry production atlas (256 production slots plus 16 D3 direction sprites). No renderer or simulation changes were made during this review.

## Runtime evidence

Commands run from the production checkout with the bounded Godot wrapper:

- `test_agent_activity.gd`: exit 0. Covers optional target compatibility, tile inference priorities, sleep visibility, facing, frame timing, shader variant decoding, actual Main snapshot context, and selected-agent ring lifecycle.
- `test_agent_native_size.gd`: exit 0. Confirms atlas frame size, scale 1, negative anchor offset, nearest filtering, and no state tint.

These are node/data checks, not windowed pixel acceptance. The atlas inspection below reads actual PNG pixels. Review montage: `/tmp/art_agents_production_review.png` (temporary review aid, not a delivery artifact).

## Findings at initial inspection

1. **Adult sleeping fallback was a thin bar.** `V_a3_13` and `V_b3_13` were 13 × 5 pixels with anchor `[6, 4]`; their visible output was a gold horizontal bar, not a recognizable lying adult. Both manifest sources were `procedural:animation:asleep`. The child sleeping frame `V_c2_13` was 23 × 14 and retained a recognizable rotated body. Parent assigned the second repair round to the converter owner; this initial finding must be rechecked against the final atlas.
2. **Directional and adult-body identity remain incomplete.** Exact atlas RGBA pixel SHA-256 prefix `27bdf0d98660` was shared by `V_a1_01`, `V_a1_03`, `V_a1_04`, `V_b1_01`, and `V_b1_04`. Adult A front/back/side and adult B front/side therefore show the same D3 gatherer silhouette, including its baked-in basket. The child front/back/side frames `V_c1_01`, `V_c1_03`, and `V_c1_04` shared prefix `39825877dc48`. Facing metadata and horizontal flip do not establish new directional art coverage. The fallback manifest already identifies unavailable same-view idles and adult-B identity.
3. **Mixed accepted/fallback north walk has a width change.** Adult B north walk frames 13–15 have canvas width 18, anchor x 9; accepted `V_b1_16` has width 15, anchor x 7. Anchored visible left extent changes from −9 to −7. This is a two-texel silhouette shift within the cycle, even though height and top baseline remain stable.
4. **Fallback passing-step feet lift one texel.** Adult fallback walk frames 2/4 have last opaque row 28 with anchor y 29; child equivalents have last opaque row 21 with anchor y 22. Their feet sit one texel above the anchor. The movement can read as a lift, but it does not prove the required last-opaque-row feet anchor contract.

## Positive observations and limits

- Adult idle/walk anchored head top was consistently −29; child idle/walk top was −22. No head-top jump was found in these frame groups.
- Adult front idle canvas 17 × 30 meets the requested displayed tall silhouette ratio (30/17 ≈ 1.76); native class budgets exclude the exterior outline, so the 30-pixel outlined canvas is not itself a size-budget failure.
- Activity mappings have the expected adult groups for idle, walk, carry, gather, build, drink, eat, talk, sit, and asleep. Child unsupported work/carry activities fall back to available idles; they must not be counted as distinct child activity art.
- Generated/accepted status and procedural/fallback status remain separate. A filled slot is not a verified new view or a distinct body.
- Windowed strip and calibration checks remain the authority for actual on-screen animation, palette output, y sorting, and pixel blocks.
