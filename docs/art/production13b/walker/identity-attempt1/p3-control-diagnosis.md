# P3 control mismatch diagnosis

**Exact identity remains FAIL.** The confirmed rendered difference is the pause indicator opacity: A and B show the dim indicator; A′ shows the bright indicator. This is not an agent geometry difference and is not hover styling. The originating input event is not recorded, so its human/OS/tool origin cannot be asserted.

## Measured framebuffer evidence

- Full-resolution A, A′ and existing control diff were inspected. A/A′: 924 pixels differ, maximum channel difference 149; bounding box `(1676, 58, 1706, 90)` (exclusive upper bounds). Every pixel outside that rectangle is identical. A/B is exact across the full image.
- Pixel `(1680,64)`: A RGB `(100,91,67)`, A′ RGB `(246,230,176)`. Pixel `(1684,72)`: A `(75,70,46)`, A′ `(174,169,115)`. Adjacent panel background at `(1675,58)` and `(1707,74)` is `(21,16,9)` in both.
- Across all 924 changed pixels, compare A against `round(0.35 * A′ + 0.65 * (21,16,9))` per channel: 868 pixels have zero residual, 56 have a maximum residual of 1 channel level (8-bit compositing rounding). This is the exact opacity relationship selected by the HUD source, to framebuffer quantization precision. This calculation is diagnosis only; no tolerance was applied to the identity gate.
- A/P3.json and Aprime/P3.json are byte-identical, including tick, process/capture schedule, census and draw-clock evidence. The sidecar does not record camera `_paused`, HUD pause alpha, or input events. Therefore equal capture clocks cannot certify equal pause UI state.

## Frozen source evidence

All four cited source files were SHA-verified against A/provenance/initial.json before interpretation; these are the actual frozen capture sources.

- `scripts/ui/panels/hud_topbar.gd:80`: creates `ArtUI.icon(14)`, the pause icon. Line 213 sets `_pause_icon.modulate.a = 1.0` when `/root/Main/Camera2D._paused` is true, otherwise `0.35`. `_process` at line 103 calls `_refresh` every process frame; there is no refresh timer/interval governing this indicator.
- `scripts/ui/panels/art_ui.gd:17–25`: the icon holder is a plain Control with `MOUSE_FILTER_IGNORE` and a Sprite2D at 2× scale. There is no button-hover theme or mouse-entered styling on this icon. Thus the bright A′ icon is not explained by hover.
- `scripts/ui/camera_controller.gd:27`: `_paused` starts false. Lines 64–67 route a pressed, non-echo `KEY_P` to `_toggle_pause`; lines 116–125 flip `_paused` and change WorldSim process mode. Repository search found this to be the only production assignment changing `_paused` and this to be the only production call to `_toggle_pause`.
- `scripts/test/art/capture_framings_p13b.gd:116–123`: freezes WorldSim with speed 0 and disabled processing, then adds Main and configures camera auto-track/smoothing. It does not disable camera processing/unhandled input, set a deterministic `_paused`, or stamp that value. A KEY_P can therefore alter the HUD pause state while simulation speed remains 0; unchanged target tick is consistent with this failure mode.
- By contrast, `scripts/test/art/capture_identity_p13b.gd:47` disables camera process mode for the other still framings. P3 uses the general framing collector, so it lacks that isolation.

## Conclusions and boundary

The concrete framebuffer cause is differing pause-state presentation, matched quantitatively to the source's 0.35/1.0 alpha branches. The collector has a concrete isolation gap: camera input remains enabled and pause state is absent from capture invariants. Neither a walker regression nor render-clock drift explains the observed region as directly as this evidence; all world pixels and A/B remain exact. The recorded log has no pause/input trace. It is therefore **not verified** who or what produced a KEY_P event, or precisely when it occurred; the evidence must not claim observed user input. No runtime reproduction was launched under the read-only task.

Recommended leader-owned follow-up: isolate camera input deterministically in the P3 identity-only path, normalize the pause state before the 120-frame settle, and stamp pause state/alpha. Then collect fresh A/A′/B controls without overwriting this failure. Keep full-frame exact RGBA thresholds and preserve all failed evidence; do not crop, mask, exclude, or tolerate the 924 pixels.

## Artifacts

- `comparison-attempt1/walker-identity.json`: combined FAIL, supplemental polarity/provenance errors empty.
- `comparison-attempt1/rgba-reference/comparison.json`: P3 control FAIL; all six A/B exact.
- `comparison-attempt1/rgba-reference/P3-control-diff.png` and `previews/P3-control-diff-x16.png`.
- `comparison-attempt1/running-reference.json`: all five running worlds PASS exact JSONL.
- `comparison-attempt1.log`, `comparison-attempt1.exit`: validator exit 1.

No images edited, captures rerun, thresholds weakened, product/source modified, or commits made.

SHA256 evidence:
- `six/A/P3.png`: `47e33fac19234224fc9e05085c81a278f9aff3cd9e33517b6cc21c640564e453`
- `six/Aprime/P3.png`: `51f51bae95640434014c6618d4fdf32c95578319dce8fea45c82c2204caba6f5`
- `six/B/P3.png`: `47e33fac19234224fc9e05085c81a278f9aff3cd9e33517b6cc21c640564e453`
- `six/A/P3.json`: `74097e01df3da6641dfb9f2aaa61a7900689f2f454d1db52e7c778031498407e`
- `six/Aprime/P3.json`: `74097e01df3da6641dfb9f2aaa61a7900689f2f454d1db52e7c778031498407e`
