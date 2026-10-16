# Capture crowd and wood-carrier diagnosis

This bounded investigation found no demonstrated actor-rendering or seat-allocation defect. Runtime geometry and activity rules were retained unchanged. Original captures and initial QA results were preserved.

## Evidence

- Initial audit: `docs/art/production4/ingame_qa_initial.json`.
- Day 10: `/Users/rexxa/work/art-production4-evidence/shots/art_long/l_d10_1500_F2.entities.json`.
- Day 18: `/Users/rexxa/work/art-production4-evidence/shots/art_long/l_d18_1200_F2.entities.json`.
- Day 24: `/Users/rexxa/work/art-production4-evidence/shots/art_long/l_d24_1200_F2.entities.json`.

## A17: historical displacement versus the captured activity

Day-10 IDs 15 and 6 have `state=0`, `target_kind=0`, and `carried_wood=2` at both recorded snapshot endpoints. ID 15 moved from simulation tile (10,20) to (11,19); ID 6 moved from (17,26) to (18,25). Their captured activity and pose are `idle`.

The sidecar labels this displacement `observed_tile_displacement_during_final_synchronous_advance`. `scripts/test/capture_framings.gd` subsequently refreshes the renderer, freezes simulation, and waits 1.15 render seconds plus five process frames before capture. Therefore these historical displacements do not establish movement at the captured instant. The settled, stationary `ipos` correctly resolves to idle under sections 7.1 and 8.2. No carrier-pose override was introduced. The initial two failures remain historical diagnostic evidence; the metrics owner separately handles contemporaneous movement qualification.

## A18: limits of the specified formation

The sole hearth is at simulation tile (23,18), with the section-7 seat-coordinate feet position (824,330). All visible seated pairs in the day-18 and day-24 sidecars are at least 14 pixels apart. Hidden or retired sprite metadata must not be counted as currently occupied seats.

| Capture | Failed actor | Simulation tile | Chebyshev radius | Diagnosis |
| --- | ---: | --- | ---: | --- |
| Day 18 | 28 | (18,19) | 5 | Outside required radius 4; unseated |
| Day 18 | 9 | (19,23) | 5 | Outside required radius 4; unseated |
| Day 18 | 0 | (17,24) | 6 | Outside required radius 4; unseated |
| Day 24 | 25 | (21,17) | 2 | State 1 and walk pose; not eligible for stationary seating |
| Day 24 | 29 | (20,12) | 6 | Outside required radius 4; unseated |
| Day 24 | 5 | (22,13) | 5 | Outside required radius 4; unseated |
| Day 24 | 30 | (21,15) | 3 | Valid seat at (800,316); seating does not prevent all sprite overlap |
| Day 24 | 40 | (22,14) | 4 | No remaining legal seat within 48 pixels |
| Day 24 | 39 | (21,15) | 3 | No remaining legal seat within 48 pixels |

For ID 40, interpolated settled position including identity jitter is (804,265). After accounting for occupied seats and known building/deadwood exclusions, its only remaining geometric candidate within 48 pixels is (768,288), mapping to tile (20,15). For ID 39, settled position is (793,283); the only remaining geometric candidates are (768,288) and (760,302), mapping to tiles (20,15) and (19,16). Both tiles are water and are correctly excluded. These calculations used the exact specified lattice, ellipse, hearth clearance, tile rounding, 48-pixel reach, and 14-pixel occupied-seat separation.

The water classification was verified with a bounded headless Godot read of the actual main scene's terrain snapshot. The temporary probe disabled WorldSim processing, read terrain at the two coordinates, and exited successfully:

```text
TERRAIN_PROBE (20, 15) code=4
TERRAIN_PROBE (19, 16) code=4
```

Probe command: `bash tools/art_pipeline/run_bounded.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script /tmp/p4_actor_terrain_probe.gd`. Terrain code 4 is the same water exclusion used by the renderer. The temporary probe is supplementary evidence, not a committed test or modified capture.

Initial A18 remains day 18 **3/39** against limit **1**, and day 24 **6/41** against limit **2**. Expanding eligibility radius, changing the mandated offsets, or reallocating persistent seats merely to reduce this measurement would change section 7's explicit contract. These failures are retained as observed visual limitations.
