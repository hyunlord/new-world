# Part1 focused validation

Five headless tests pass with no SCRIPT ERROR/ERROR in their final logs; `git diff --check` passes for owned files. Full gate and windowed natural visual acceptance remain leader-owned.

Decisions: frost rows use native frame y < height*0.4; source alpha is preserved, underside is applied only to opaque pixels beneath alternate roof-edge pixels (stable row-major ordering), then roof highlights take priority. This never adds silhouette pixels. Doorways are native frame center at ground anchor minus5 pixels, verified against the three installed S1 shelter frames. Cold puff reuses Cold.breath timing and the existing white3x2 texture construction. Tint is a screen CanvasLayer0 below existing UI CanvasLayer1, preserving daylight CanvasModulate. Cold-snap getter is optional and its .18 tint is the explicit Part5 exception to natural .12.

The first structure test emitted errors from its pre-existing ResourceDouble lacking get_agent_snapshot while returning exit0. Rejected log retained. Optional snapshot guard resolves fixture compatibility; final run has no errors. No simulation parameters/state changed, no localization keys added, no scene wiring needed.
