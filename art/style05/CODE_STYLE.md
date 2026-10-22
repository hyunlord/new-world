# Procedural ink companions

The standalone prototype's `Companion.gd` draws four samples beside generated candidate art: a stockpile zone, blueprint outline, selection rectangle and progress bar. These are review samples, not game integration or style acceptance.

Use warm dark ink `#30291f`, 1.5-pixel antialiased strokes at 1×, no opaque panel fill, and at most 15% wash. The sample wash is RGB(0.40, 0.49, 0.47), alpha 0.15. Each line's midpoint gets a deterministic offset in [−0.5,+0.5] pixels; endpoints remain fixed so connected edges meet. Keep jitter independent of time to avoid shimmer. Dashed outlines use 5-pixel marks on a 9-pixel interval.

The stockpile has a light wash and dashed perimeter; the blueprint adds a sparse architectural interior; selection uses an unfilled continuous border; progress uses the same ink outline with a partial wash. The blue alpha-80 derived building ghosts are a separate trial requirement, not this outline sample's colour specification.

Inspect at actual 0.5×, 1× and 2× beside ground, objects and people. A shader or `_draw()` implementation is not evidence of visual consistency: retain the image-model capture scores and a direct visual verdict in the trial report. Numeric asset thresholds were calibrated on standalone anchor images; do not silently treat composite UI captures as accepted environment assets.

Current acceptance: PENDING capture review. No game consumer has been changed.
