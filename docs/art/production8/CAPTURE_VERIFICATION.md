# Part 5 final capture verification

All eight bounded capture jobs exited 0. Console scan found zero SCRIPT ERROR, ParseError or assertion failures. `CAPTURE_RESULTS.json` records individual times, exact external log paths and SHA256.

| Scenario | Images | Identity |
|---|---:|---|
| ART | 94 | live seed20261010; includes4calibration/lineup frames |
| ART_LONG | 30 | live seed20261010, measured game_v19 dates |
| ART_SOCIETY | 17 | seed20261013; no interventions |
| ART_GOD live | 10 | explicitly intervention-labelled |
| ART_GOD society | 4 | explicitly intervention-labelled |
| Paired world P | 7 | spoil_food60 ondays30/31/32; throughday120 |
| Paired world G | 7 | gift_food60 ondays30/31/32; throughday120 |
| Total | 169 | natural QA excludes4calibration/lineup frames |

All capture source revisions are `cefc77b088d6fa3baf21b766cdc40e3fdf045d36`. Four GOD runs preserve distinct run_id/world_epoch and source manifests in the combined manifest; merger verified raw hashes, unique stems and sidecar parity. Published segment duplicates remain externally preserved with SHA records in `god-segments-relocation.json`.

The separate final A30 job measured tick126994, population67, three living settlements, FV, 12alternating blocks of50 frames, 300on/300off. Raw protocol valid=true and errors=[]; process+render CPU median on18.984499979ms/off17.099000003ms. Camera, layer/process state, probes, render measurement flag and vsync restoration all verified. GPU0 is unavailable; old wall-clock pacing is reported separately in Part6.

Exact entity sidecars are collectively archived with per-member hashes and byte-exact restore verification (`EVIDENCE_ARCHIVE.md`), not discarded. PNG publication upgrades use existing full-size standard captures without resampling; source JPEGs remain immutable externally. The 40-frame noon/evening animations retain all decoded RGBA frames and durations.

P first departure is day31 06:00; G is day60 06:00. P already split before the final paired morning, so its pressure reset. These observations do not by themselves establish A33 PASS; the unchanged Part6 predicate is applied separately.
