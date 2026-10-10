# Part 6 — final round 2 measurement

Runtime source: `9f8b49af7e67f5947c247c027d654604b7027188`. Three separately locked capture commands completed; 107 raw SHA256 checks passed, 99 gameplay metadata records use schema 4 and eight calibration/lineup records use schema 3. Nineteen paused stills include observed motion intervals; 80 live-strip frames are temporal sequences. SCRIPT ERROR count: 0. Full capture receipt: `capture_verification.json`.

## Final acceptance

A5, A11, A12 and A18 FAIL. All other numbered checks PASS. This is bounded partial delivery, not complete visual acceptance. Full numbers and definitions: `ingame_qa.md`, `ingame_qa.json`, `measurement_summary.json`.

- A5 edge median noon 0.345408 and winter 0.354405; all measurable actors exceed 0.10. Noon 13/17 measurable passes; winter 25/42 = 59.5238% fails the unchanged 60% coverage threshold. Initial winter capture was 25/41 and passed. Final capture is authoritative.
- A11 frozen independent results: 19/24 at 4x, 14/24 at 2x, both fail. Round2 lineup file hashes and decoded pixels exactly equal initial; reuse proof in `blind/round2_reuse_proof.json`.
- A12 11/17 natural tuples fails minimum12; same-frame old tuple emulation16/17 passes. Old skin/clothing values are exact frozen P3 ArtHash emulation, not captured old metadata.
- A16 corrected d32 spring passes; old d40 spring definition fails. Winter/summer blue:red ratio1.16748; 41/42 winter actors measurable with all channels within1; nine elders have hair3/4.
- A17 initial90 violations, corrected initial88 after excluding two currently stationary historical movers, final0. Final70 hearth,586 deadwood,99HUD observations; three current moving wood carriers use recorded fallbacks, zero real wood frames.
- A18 half-occluded/visible: noon1/17, dinner0/20, d18 3/39, d24 6/42. Last two fail unchanged limits1 and2. Exact requested seat rules retained; diagnosis in `crowd_diagnosis.md`.

Initial and corrected initial audits and capture manifests remain preserved. The one runtime fix refreshed structure/resource presentation on snapshot ticks and placed piles at deterministic legal adjacent footprints. No second speculative runtime fix was made. Independent final still review in `manual_visual_review.md` holds overall acceptance because crowding and detached baskets remain.

## Deliverables and limits

All six requested artifacts exist, decode and have verified SHA256 records in `presentation_artifacts.json` and `report_artifacts.json`. Both GIFs contain40 frames and are below8MB: noon2,103,090bytes, evening2,897,462bytes. Six artifacts total5,737,141bytes. P3/P4 framing and zoom match for all three comparisons. Natural sheet has43 rows at4x; wood fallback rows are explicitly labelled. Colour comparison emulates exact P3 sources for16/17 actors and65,531 texels; newly recovered ID24 is explicitly unchanged.

Evening strip shows agent11 eating in two frames; static19:30 and08:30 samples show none. Breakfast06:00–08:00 remains unobserved. Sprite clicking has headless integration evidence, not a manual windowed click. Timings: median7.547ms,p9515.329ms versus P3 6.048/15.162ms, separate runs rather than a controlled equivalence experiment.
