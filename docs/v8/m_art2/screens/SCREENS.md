# D19 current recapture — 2026-10-24

Same 14 saved moments, ticks and camera positions as the preserved D18 set below. Real windowed OpenGL captures at 1920×1080; no simulation advance. Raw PNGs: `/Users/rexxa/work/mart2-data/d19/screens/`. The old-renderer PNGs are unchanged. Full hashes, save hashes and test logs: `../D19_EVIDENCE.json`. Curated comparison and contact sheets now show D19.

| Shot | Tick | Zoom | D19 PNG SHA-256 (first 16) |
|---|---:|---:|---|
| `mart2_01_day6_noon_1x` | 7440.0 | 1.0 | `e7259db52695b345` |
| `mart2_02_day6_noon_half` | 7440.0 | 0.5 | `f0e1d789cf9dcc65` |
| `mart2_14_day6_noon_quarter` | 7440.0 | 0.25 | `b883374e4d85defb` |
| `mart2_08_storehouse_2x` | 7440.0 | 2.0 | `941f2244e1c3e293` |
| `mart2_09_dug_pocket_2x` | 7440.0 | 2.0 | `e8f7306150206031` |
| `mart2_10_ford_2x` | 7440.0 | 2.0 | `0867daf6f1367561` |
| `mart2_03_day6_night_1x` | 6840.0 | 1.0 | `93ced904925e48f7` |
| `mart2_04_day6_night_half` | 6840.0 | 0.5 | `72b1510c9929f633` |
| `mart2_07_house_sleepers_2x` | 6840.0 | 2.0 | `35c757852d173174` |
| `mart2_05_winter_noon_1x` | 26160.0 | 1.0 | `d897677eb94e854c` |
| `mart2_06_winter_noon_half` | 26160.0 | 0.5 | `a5d0d76ad56882ea` |
| `mart2_11_workshop_workers_2x` | 7710.0 | 2.0 | `c9849a9487286f76` |
| `mart2_12_wolf_attack_1x` | 19000.0 | 1.0 | `8105e5b3c15f10b1` |
| `mart2_13_migrant_arrival_1x` | 16140.0 | 1.0 | `2fb4af9ed2ebe6f2` |

## Preserved pre-D19 inventory

The following table and commentary describe the earlier capture, not the current curated sheets.

# M-ART2 screen set

The owner set (goal §11), captured by `tools/v8/capture.gd --mart2-set` (`tools/v8/mart2_shots.gd`) on colony seed 0
saves written by the probe with `tools/v8/m55_u_script.json` (day 6 noon tick 7440, day 6 02:00 tick 6840, first winter
noon tick 26160), windowed 1920×1080, `--rendering-driver opengl3`, the clock stopped; each shot also saved its world
(`<name>.wsave`), and the old renderer (the M5.5 base `29f720aa`, 16-px tiles) drew the same save and the same world
area with `tools/v8/mart2_old_shots.gd` (old zoom = new zoom × 4). Every frame is observed at a fixed time. The raw
PNGs and saves live outside the repository in `/Users/rexxa/work/mart2-data/screens/`; the curated sheets are
`docs/images/mart2_contact_sheet.png` (12 shots, manifest `contact_sheet_mart2.json`) and
`docs/images/mart2_compare_sheet.png` (6 old/new pairs, manifest `compare_sheet_mart2.json`).

Captured again after the review fixes (MART2_DECISIONS D18: darker hill tops, dug floors as packed earth, 2-px
drafted outline, the migrant shot framed below the dialog); the first capture is kept in
`/Users/rexxa/work/mart2-data/screens_v1/`. Hashes are the first 16 hex digits of the SHA-256 of the raw PNG.

| shot | tick | tile | zoom | caption | new | old |
|---|---|---|---|---|---|---|
| `mart2_01_day6_noon_1x` | 7440 | (91, 101) | 1.0× | 6일째 정오, 1× — 바닥은 차분하고 사람·물건·건물만 잉크 그림 | `99483c55695f776f` | `e1bcb072bb918917` |
| `mart2_02_day6_noon_half` | 7440 | (91, 101) | 0.5× | 6일째 정오, ½× — 사람은 상의 색 동그라미와 활동 표시 | `f8f43ab7cf646e9c` | `196a1c52a1728c88` |
| `mart2_03_day6_night_1x` | 6840 | (91, 101) | 1.0× | 6일째 02:00, 1× — 어둡고 모닥불 주변만 밝다 | `f1e7f7250adef774` | `2adb499247333b0e` |
| `mart2_04_day6_night_half` | 6840 | (91, 101) | 0.5× | 6일째 02:00, ½× | `3534bab61d331b8b` | `70c42412c6aec78d` |
| `mart2_05_winter_noon_1x` | 26160 | (91, 101) | 1.0× | 첫 겨울 정오, 1× — 바닥이 바래고 눈이 덮인다 | `48ef2ac75b2b444e` | `8d91e9e2ae7b69a8` |
| `mart2_06_winter_noon_half` | 26160 | (91, 101) | 0.5× | 첫 겨울 정오, ½× | `2db014d9d7aa2e90` | `4da3c32770a67ac7` |
| `mart2_07_house_sleepers_2x` | 6840 | (84, 107) | 2.0× | 집과 잠든 사람 (2×) — 지붕 없이 방 안 | `d344f9890f26422e` | `f8e36858a78dcf62` |
| `mart2_08_storehouse_2x` | 7440 | (103, 97) | 2.0× | 창고 (2×) — 벽 윗면이 벽 방향으로 누운 재료 | `7bc12fbf091fae20` | `683d0553768ebcf2` |
| `mart2_09_dug_pocket_2x` | 7440 | (80, 85) | 2.0× | 판 굴 (2×) — 입구 위 바위 면 | `dbbf5a7d6dc28e30` | `2f08f041ca28e44b` |
| `mart2_10_ford_2x` | 7440 | (87, 100) | 2.0× | 여울 (2×) — 물결 텍스처와 징검돌 | `79c1495767369e79` | `938081c7b40b25bd` |
| `mart2_11_workshop_workers_2x` | 7710 | (90, 103) | 2.0× | 작업장과 일하는 사람 3명 (2×) | `789c011793d0abfb` | `a1541ddb6b296707` |
| `mart2_12_wolf_attack_1x` | 19000 | (83, 91) | 1.0× | 늑대 공격과 소집한 4명 (1×) — 빨간 테두리 | `4814d19d24d1e7f4` | `430bd8f1162a4a97` |
| `mart2_13_migrant_arrival_1x` | 16140 | (91, 96) | 1.0× | 이주민 도착 (1×) — 대화창 아래 모닥불 곁에 이주민 | `f3bc6da160ead562` | `47ac0a6e390bd54f` |
| `mart2_14_day6_noon_quarter` | 7440 | (91, 101) | 0.25× | 6일째 정오, ¼× — 마을 전체 | `c2cceab654255f0f` | `e1903f81d4730926` |

Scenes found by the shot script (not hand-picked): the workshop shot is the first tick (30-tick steps from 7440) with
three people working within 3 tiles of a workshop (7710); the wolf shot the first hour with wolves within 30 tiles of
the hearth, then the four adults nearest the hearth drafted and ordered at the nearest wolf, the frame 40 ticks later
(tick 19000 as recorded); the
migrant shot the first wave waiting at the hearth (16140).
