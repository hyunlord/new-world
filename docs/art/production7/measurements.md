# Art Pass 7 measurements

A1–A25: live seed; A24: live intervention. A26: all scenarios. A27–A30: society. A18_SOCIETY stays separate.

Canonical full numeric evidence and every failure: [measurements.json](measurements.json).

| Check | Status | Numeric result / evidence |
|---|---|---|
| CAPTURE_SET | PASS | 94/94 entries; missing=[]; extra=[] |
| A1 | PASS | {} |
| LONG_CAPTURE_SET | PASS | 27/27 captures; missing=[]; extra=[] |
| A2 | PASS | raw checker zero mixed pixels: [(4.0, True), (6.0, True)] |
| A3 | PASS | 1309 drawn rects; wrong=[]; adult S-idle frames=4, aspect failures=[]; absent S-idle cannot pass |
| A4 | PASS | [(4.0, 'north', 5232, 1.0), (4.0, 'south', 4672, 1.0), (6.0, 'north', 11772, 1.0), (6.0, 'south', 10512, 1.0)] |
| RAMP | PASS | actual framebuffer agrees with region-indexed ramp texels: [(4.0, 10, 1.0), (4.0, 11, 1.0), (4.0, 12, 1.0), (4.0, 13, 1.0), (4.0, 14, 1.0), (4.0, 15, 1.0), (4.0, 16, 1.0), (4.0, 17, 1.0), (4.0, 18, 1.0), (4.0, 19, 1.0), (4.0, 20, 1.0), (4.0, 21, 1.0), (6.0, 10, 1.0), (6.0, 11, 1.0), (6.0, 12, 1.0), (6.0, 13, 1.0), (6.0, 14, 1.0), (6.0, 15, 1.0), (6.0, 16, 1.0), (6.0, 17, 1.0), (6.0, 18, 1.0), (6.0, 19, 1.0), (6.0, 20, 1.0), (6.0, 21, 1.0)] |
| A5 | PASS | {} |
| A6 | PASS | {"lit_hearths": 1, "lit_check": "noon/night world=[0.31239058115943397, 0.15604615836727734]; 48px hearth means=[0.2554014763104743]; each hearth tested"} |
| A7 | PASS | P4 scope: followed moving samples=40, pass=True; P5 scope: all visible moving-villager samples in noon GIF=231, minimum=30; distinct frames/full moving second={0: 7, 1: 4, 2: 4}; changing fraction=1.000; duration=3.967s; interval range=0.048..0.123s |
| A8 | PASS | 116 scenes checked; failures=[] |
| A9 | PASS | 117 scenes checked; failures=[] |
| A10 | PASS | n=300, median=8.228ms, p95=9.502ms; live simulation=True |
| A11 | PASS | {"errors": []} |
| A13 | PASS | foot-anchor opaque symmetric difference/union; best frames=[('adult_a', 'walk', 'V_a1_11', 0.2111368909512761), ('adult_a', 'carry', 'V_a2_06', 0.17410714285714285), ('adult_a', 'gather', 'V_a2_11', 0.22648752399232247), ('adult_a', 'build', 'V_a2_15', 0.3249551166965889), ('adult_a', 'eat', 'V_a3_06', 0.30948419301164726), ('adult_a', 'drink', 'V_a3_03', 0.3197781885397412), ('adult_a', 'sit', 'V_a3_08', 0.26476190476190475), ('adult_a', 'talk', 'V_a3_09', 0.2923976608187... |
| A14 | PASS | captured tick=1680; forest-tree tiles=221/873 (0.25315); duplicate tree tiles=0; clearing centers=26; checked objects=322; violations=[] |
| A12 | PASS | 18 tuples/22 villagers; identical neighbors=0/6 (0.000); A12-old (diagnostic): 22 tuples; identical neighbors=0/6 (0.000); pass=True; 22 emulated from frozen P3 ece4e85e5e0b48945dddef61ba601a144db1397a ArtHash identity plus captured age-adjusted hair (not captured skin/clothing metadata); A12-P4 (counterfactual): 13 tuples; identical neighbors=1/6 (0.167); pass=False; body, adult hair 0..2 (elders 3..4), tone 0..2 emulated from same IDs; age inferred from captured P5 hair ... |
| A15 | PASS | {"tick": 1680, "checked_placements": 546, "clearing_centers": 26, "installed_decals": 13, "scope": "all shown placements, including offscreen; density denominator is every terrain tile"} |
| A16 | PASS | {} |
| A17 | PASS | {"errors": [], "carry_status": "observed", "moving_criterion": "current snapshot state == 1 (walking) AND observed displacement over <=8 ticks from synchronous advance OR adjacent captured same-animation snapshots with identical entity sets", "label_scope": "all settlement-assigned stockpiles require captured nodes; shown follows zoom>=2 independently of viewport; visible follows rectangle intersection"} |
| A18 | FAIL | {} |
| A11_FINAL_PIXEL_IDENTITY | PASS | Final calibration lineup matches original frozen 4x/2x grid pixels; see blind_identity_final.json |
| SCENARIO_PARTITION | PASS | {"live": 117, "society": 17, "intervention": 12} |
| A19 | PASS | {"captured_visible_cold": 5, "presentation": "observed", "cycles_checked": 2, "target_labels_checked": 72, "errors": []} |
| A21 | PASS | {"frames_checked": 14, "errors": [], "independent_observations": "docs/art/production7/visual_observations.json"} |
| A22 | FAIL | {"frames_checked": 3, "errors": ["l_d29_0630_F2: visible/rendered morning chill differs from independent FFI interval", "l_d29_0715_F2: visible/rendered morning chill differs from independent FFI interval"], "independent_observations": "docs/art/production7/visual_observations.json"} |
| A23 | PASS | {"frames_checked": 117, "errors": [], "independent_observations": "docs/art/production7/visual_observations.json"} |
| A24 | FAIL | {"frames_checked": 8, "errors": ["god_d10_1200_F2_spoil: observer explicitly saw only panel confirmation, not the world effect", "god_d10_1200_F2_spoil: effect not independently observed", "god_d23_1200_F2_gift: observer explicitly saw only panel confirmation, not the world effect", "god_d23_1200_F2_gift: effect not independently observed"], "independent_observations": "docs/art/production7/visual_observations.json"} |
| A25 | PASS | {"frames_checked": 9, "errors": [], "independent_observations": "docs/art/production7/visual_observations.json"} |
| A18_SOCIETY | FAIL | {} |
| A26 | PASS | {"frames": 146, "exposure_markers_in_view": 3, "errors": []} |
| A27 | PASS | {"frames": 12, "overview_frames": 1, "errors": []} |
| A28 | FAIL | {"walking_frames": 5, "errors": ["soc_d29_0630_FD_leader: site marker is outside view", "soc_d29_0700_FD_leader: site marker is outside view", "soc_d85_0630_FD_leader: site marker is outside view"], "bundle_drop": false} |
| A29 | PASS | {"rain_frames": 4, "toast_frames": 1, "same_frame_nights": 1, "errors": []} |
| A30 | FAIL | {"median_ms": [9.7695, 15.828], "ratio": 1.6201443267311528, "threshold": 1.15, "samples": [300, 300], "simulation_frozen": true, "load_condition": "Recorded host loads differ; not a controlled same-load comparison or causal attribution.", "scope": "back-to-back same-process render frame comparison only; frozen samples do not measure live simulation cost", "git_sha": "28f58e56a2289bde129946b7f630192cd94496b2", "expected_git_sha": "28f58e56a2289bde129946b7f630192cd94496b2",... |
| A20 | PASS | {"check": "A20", "passed": true, "errors": [], "installations": 2, "final_bound_frames": 74, "changed_villager_frames": 61, "animation_groups": 20, "head": "28f58e56a2289bde129946b7f630192cd94496b2", "atlas_sha256": "72a01917bc9cdd151da3a2e8074bfa2e1963627aadd693680f5a1eb143e8c274", "scope": "Current assets; inherited Pass 5 hair masks and harvest installed-frame provenance", "reference_base": "docs/art/production5"} |
