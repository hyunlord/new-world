# Art Pass 8 measurements

N/A is not PASS. Natural A1–A25; society A26–A31 and separate A18. Intervention evidence remains explicitly separate.

Full exact details: [measurements.json.gz](measurements.json.gz). Input audit: [qa_evidence.json.gz](qa_evidence.json.gz), lossless gzip JSON. [Reading compressed evidence](EVIDENCE_FORMAT.md).

| Check | Status | Summary |
|---|---|---|
| CAPTURE_SET | PASS | 94/94 entries; missing=[]; extra=[] |
| A1 | PASS | thresholds: 2 entries; old_all_frames: 3 entries; gameplay_only: 2 entries; excluded: 4 entries |
| LONG_CAPTURE_SET | PASS | expected: 30 entries; actual: 30 entries |
| A2 | PASS | raw checker zero mixed pixels: [(4.0, True), (6.0, True)] |
| A3 | PASS | 1438 drawn rects; wrong=[]; adult S-idle frames=4, aspect failures=[]; absent S-idle cannot pass |
| A4 | PASS | [(4.0, 'north', 5232, 1.0), (4.0, 'south', 4672, 1.0), (6.0, 'north', 11772, 1.0), (6.0, 'south', 10512, 1.0)] |
| RAMP | PASS | actual framebuffer agrees with region-indexed ramp texels: [(4.0, 10, 1.0), (4.0, 11, 1.0), (4.0, 12, 1.0), (4.0, 13, 1.0), (4.0, 14, 1.0), (4.0, 15, 1.0), (4.0, 16, 1.0), (4.0, 17, 1.0), (4.0, 18, 1.0), (4.0, 19, 1.0), (4.0, 20, 1.0), (4.0, 21, 1.0), (6.0, 10, 1.0), (6.0, 11, 1.0), (6.0, 12, 1.0), (6.0, 13, 1.0), (6.0, 14, 1.0), (6.0, 15, 1.0), (6.0, 16, 1.0), (6.0, 17, 1.0), (6.0, 18, 1.0), (6.0, 19, 1.0), (6.0, 20, 1.0), (6.0, 21, 1.0)] |
| A5 | PASS | 2 detailed records; see exact evidence |
| A6 | PASS | lit_hearths=1; lit_check=noon/night world=[0.31217438082862, 0.1563832398589272]; 48px hearth means=[0.2749087196048529]; each hearth tested; world_means: 2 entries; unknown: 0 entries; unlit_glow_violations: 0 entries |
| A7 | PASS | P4 scope: followed moving samples=40, pass=True; P5 scope: all visible moving-villager samples in noon GIF=240, minimum=30; distinct frames/full moving second={0: 7, 1: 4, 2: 4}; changing fraction=1.000; duration=3.977s; interval range=0.053..0.120s |
| A8 | PASS | 119 scenes checked; failures=[] |
| A9 | PASS | 120 scenes checked; failures=[] |
| A10 | PASS | n=300, median=11.387ms, p95=13.352ms; live simulation=True |
| A11 | FAIL | observers: 2 entries; errors: 1 entries |
| A13 | PASS | foot-anchor opaque symmetric difference/union; best frames=[('adult_a', 'walk', 'V_a1_11', 0.2111368909512761), ('adult_a', 'carry', 'V_a2_06', 0.17410714285714285), ('adult_a', 'gather', 'V_a2_11', 0.22648752399232247), ('adult_a', 'build', 'V_a2_15', 0.3249551166965889), ('adult_a', 'eat', 'V_a3_06', 0.30948419301164726), ('adult_a', 'drink', 'V_a3_03', 0.3197781885397412), ('adult_a', 'sit', 'V_a3_08', 0.26476190476190475), ('adult_a', 'talk', 'V_a3_09', 0.29239766081871343), ('adult_b', 'walk', 'V_b1_11', 0.2420091324200913), ('adult_b', 'carry', 'V_b2_06', 0.23076923076923078), ('adult_b' |
| A14 | PASS | captured tick=1680; forest-tree tiles=221/873 (0.25315); duplicate tree tiles=0; clearing centers=26; checked objects=315; violations=[] |
| A12 | PASS | 18 tuples/22 villagers; identical neighbors=0/5 (0.000); A12-old (diagnostic): 22 tuples; identical neighbors=0/5 (0.000); pass=True; 22 emulated from frozen P3 ece4e85e5e0b48945dddef61ba601a144db1397a ArtHash identity plus captured age-adjusted hair (not captured skin/clothing metadata); A12-P4 (counterfactual): 13 tuples; identical neighbors=0/5 (0.000); pass=True; body, adult hair 0..2 (elders 3..4), tone 0..2 emulated from same IDs; age inferred from captured P5 hair >=5; not historical pixel evidence |
| A15 | PASS | tick=1680; checked_placements=534; clearing_centers=26; installed_decals=13; scope=all shown placements, including offscreen; density denominator is every terrain tile; violations: 0 entries; duplicate_tiles: 2 entries; densities: 4 entries; native: 13 entries |
| A16 | FAIL | A16-new: 3 entries; A16-old (diagnostic): 3 entries |
| A17 | PASS | carry_status=observed; moving_criterion=current snapshot state == 1 (walking) AND observed displacement over <=8 ticks from synchronous advance OR adjacent captured same-animation snapshots with identical entity sets; label_scope=all settlement-assigned stockpiles require captured nodes; shown follows zoom>=2 independently of viewport; visible follows rectangle intersection; counts: 8 entries; motion_sources: 120 entries; real_carrier_ids: 16 entries; real_bodies: 2 entries; fallbacks: 0 entries; errors: 0 entries |
| A18 | FAIL | frames: 120 entries; Art Pass 3: 4 entries |
| A11_FINAL_PIXEL_IDENTITY | PASS | proof=docs/art/production8/blind_identity_final.json; scope=Final calibration pixels versus frozen observer grid; no natural recognition inference; source_sha256: 2 entries; errors: 0 entries |
| SCENARIO_PARTITION | PASS | natural_errors: 0 entries; society_errors: 0 entries |
| A19 | PASS | captured_visible_cold=9; presentation=observed; cycles_checked=4; target_labels_checked=75; tests: 3 entries; cold_counts_by_frame: 120 entries; errors: 0 entries |
| A18_SOCIETY | FAIL | 17 detailed records; see exact evidence |
| A21 | FAIL | frames_checked=12; independent_observations=docs/art/production8/visual_observations.json; errors: 9 entries |
| A22 | FAIL | frames_checked=6; independent_observations=docs/art/production8/visual_observations.json; errors: 5 entries |
| A23 | PASS | frames_checked=120; independent_observations=docs/art/production8/visual_observations.json; errors: 0 entries |
| A24 | FAIL | frames_checked=10; independent_observations=docs/art/production8/visual_observations.json; errors: 10 entries |
| A25 | PASS | frames_checked=9; independent_observations=docs/art/production8/visual_observations.json; errors: 0 entries |
| A26 | FAIL | frames=17; exposure_markers_in_view=0; errors: 1 entries |
| A27 | PASS | frames=13; overview_frames=8; errors: 0 entries; blind: 3 entries |
| A28 | FAIL | walking_frames=4; bundle_drop=False; errors: 3 entries; blind: 3 entries |
| A29 | FAIL | rain_frames=4; toast_frames=1; same_frame_nights=1; errors: 1 entries |
| A20 | PASS | check=A20; passed=True; installations=2; final_bound_frames=74; changed_villager_frames=61; animation_groups=20; head=cefc77b088d6fa3baf21b766cdc40e3fdf045d36; atlas_sha256=72a01917bc9cdd151da3a2e8074bfa2e1963627aadd693680f5a1eb143e8c274; scope=Current assets; inherited Pass 5 hair masks and harvest installed-frame provenance; reference_base=docs/art/production5; errors: 0 entries; rows: 72 entries |
| A30 | PASS | on_median_ms=18.984499979019166; off_median_ms=17.09900000333786; ratio=1.110269604965977; delta_ms=1.8854999756813058; formula=median(process_ms + render_cpu_ms); setup CPU separate; gpu_status=unavailable; samples: 2 entries; including_setup_diagnostic_ms: 2 entries; old_pacing_ms: 2 entries; errors: 0 entries |
| A31 | PASS | frames=17; summaries=1249; departure_feed_frames=1; event_oracle=expected_event_ids must derive from raw snapshot histories, not emitted feed; errors: 0 entries |
| A32_GEOMETRY | PASS | blind=Separate SHA-bound two-observer direction assessment required for A32; frames: 4 entries; errors: 0 entries |
| A33 | FAIL | pressure_valid=False; departure_valid=True; same_seed_and_cast_days=True; both_reached_day120=True; meter_ratios: 2 entries; departure_ticks: 2 entries |
| A32 | FAIL | geometry: 3 entries; blind: 3 entries |
| A18b | OBSERVED | frames=12; source_binding_valid=True; threshold=Companion observation only; never replaces A18; answers: 12 entries |
