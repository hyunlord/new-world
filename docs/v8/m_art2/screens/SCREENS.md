# M-ART2 screen set

The owner set (goal §11), captured by `tools/v8/capture.gd --mart2-set` (`tools/v8/mart2_shots.gd`) on colony seed 0
saves written by the probe with `tools/v8/m55_u_script.json` (day 6 noon tick 7440, day 6 02:00 tick 6840, first winter
noon tick 26160), windowed 1920×1080, `--rendering-driver opengl3`, the clock stopped; each shot also saved its world
(`<name>.wsave`), and the old renderer (the M5.5 base `29f720aa`, 16-px tiles) drew the same save and the same world
area with `tools/v8/mart2_old_shots.gd` (old zoom = new zoom × 4). Every frame is observed at a fixed time. The raw
PNGs and saves live outside the repository in `/Users/rexxa/work/mart2-data/screens/`; the curated sheets are
`docs/images/mart2_contact_sheet.png` (12 shots, manifest `contact_sheet_mart2.json`) and
`docs/images/mart2_compare_sheet.png` (6 old/new pairs, manifest `compare_sheet_mart2.json`).

Hashes are the first 16 hex digits of the SHA-256 of the raw PNG.

| shot | tick | tile | zoom | caption | new | old |
|---|---|---|---|---|---|---|
| `mart2_01_day6_noon_1x` | 7440 | (91, 101) | 1.0× | 6일째 정오, 1× — 바닥은 차분하고 사람·물건·건물만 잉크 그림 | `99483c55695f776f` | `e1bcb072bb918917` |
| `mart2_02_day6_noon_half` | 7440 | (91, 101) | 0.5× | 6일째 정오, ½× — 사람은 상의 색 동그라미와 활동 표시 | `de3dbbce55f4ecc0` | `196a1c52a1728c88` |
| `mart2_03_day6_night_1x` | 6840 | (91, 101) | 1.0× | 6일째 02:00, 1× — 어둡고 모닥불 주변만 밝다 | `f1e7f7250adef774` | `9d98a93662396821` |
| `mart2_04_day6_night_half` | 6840 | (91, 101) | 0.5× | 6일째 02:00, ½× | `1198576144973422` | `e71ab550da458221` |
| `mart2_05_winter_noon_1x` | 26160 | (91, 101) | 1.0× | 첫 겨울 정오, 1× — 바닥이 바래고 눈이 덮인다 | `48ef2ac75b2b444e` | `8d91e9e2ae7b69a8` |
| `mart2_06_winter_noon_half` | 26160 | (91, 101) | 0.5× | 첫 겨울 정오, ½× | `a7ef5a90d46a4d67` | `4da3c32770a67ac7` |
| `mart2_07_house_sleepers_2x` | 6840 | (84, 107) | 2.0× | 집과 잠든 사람 (2×) — 지붕 없이 방 안 | `c2436732b605a9aa` | `66a59411b62317ec` |
| `mart2_08_storehouse_2x` | 7440 | (103, 97) | 2.0× | 창고 (2×) — 벽 윗면이 벽 방향으로 누운 재료 | `7bc12fbf091fae20` | `683d0553768ebcf2` |
| `mart2_09_dug_pocket_2x` | 7440 | (80, 85) | 2.0× | 판 굴 (2×) — 입구 위 바위 면 | `c722f2a4af6ecd16` | `2f08f041ca28e44b` |
| `mart2_10_ford_2x` | 7440 | (87, 100) | 2.0× | 여울 (2×) — 물결 텍스처와 징검돌 | `79c1495767369e79` | `938081c7b40b25bd` |
| `mart2_11_workshop_workers_2x` | 7710 | (90, 103) | 2.0× | 작업장과 일하는 사람 3명 (2×) | `1916aaedb1a37fbd` | `a1541ddb6b296707` |
| `mart2_12_wolf_attack_1x` | 19000 | (83, 91) | 1.0× | 늑대 공격과 소집한 4명 (1×) — 빨간 테두리 | `e09e4b6add2b7b09` | `3c11a836098f574a` |
| `mart2_13_migrant_arrival_1x` | 16140 | (91, 101) | 1.0× | 이주민 도착 (1×) | `5cc504b80befdeae` | `5e46ae446c86c3dc` |
| `mart2_14_day6_noon_quarter` | 7440 | (91, 101) | 0.25× | 6일째 정오, ¼× — 마을 전체 | `09d8c19010cdd083` | `e1903f81d4730926` |

Scenes found by the shot script (not hand-picked): the workshop shot is the first tick (30-tick steps from 7440) with
three people working within 3 tiles of a workshop (7710); the wolf shot the first hour with wolves within 30 tiles of
the hearth, then the four adults nearest the hearth drafted and ordered at the nearest wolf, the frame 40 ticks later
(tick 19000 as recorded); the
migrant shot the first wave waiting at the hearth (16140).
