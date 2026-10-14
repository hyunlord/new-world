# Pass 14 Part 0 — factual timeline (`game_v19`, seeds 12000–12031)

Rules: PASS14_GOAL.md 5.3, implemented in `tools/revitalize/timeline_p14.py` (module docs). Display days are one-based (`(tick + 480) // 1440 + 1`; the probe's day fields are zero-based). A check's tick is its 06:00 tick; a cast at a check tick is applied before that tick's systems run (`export.rs` `apply_due` before `engine.tick()`), so it changes that same morning's s.

Food line check: the line max(1, floor(members × 2.4 × 4)) reproduces the logged s on 2059 of 2059 eligible checks (all settlements, all seeds).

Seeds with T_low 28 / 32 (0.875; 2 from the after-first-departure clause), T_high 26 / 32 (0.812), T_eve 32 / 32. Seeds that never depart: 11. Cancelled home departures: 0. Largest s = 0 food/line ratio before the first departure (per seed with an s = 0 check): min 1.168, median 1.557, max 1.836 (n 27).

| seed | eligible home checks s > 0 / s = 0 | first s > 0 day (food / line) | first departure | departures | T_low | T_high (ratio) | T_eve |
|---:|---|---|---|---:|---|---|---|
| 12000 | 20 / 19 | d12 (208 / 278) | 90600 (d64 06:00) | 1 | 15720 (d12 06:00) | 63240 (d45 06:00) (1.561) | 29400 (d21 18:00) |
| 12001 | 32 / 48 | d29 (205 / 336) | 169800 (d119 06:00) | 1 | 40200 (d29 06:00) | 58920 (d42 06:00) (1.626) | 29400 (d21 18:00) |
| 12002 | 23 / 33 | d29 (181 / 403) | 123720 (d87 06:00) | 2 | 40200 (d29 06:00) | 24360 (d18 06:00) (1.623) | 29400 (d21 18:00) |
| 12003 | 28 / 34 | d29 (234 / 364) | 90600 (d64 06:00) | 2 | 40200 (d29 06:00) | 24360 (d18 06:00) (1.651) | 29400 (d21 18:00) |
| 12004 | 47 / 35 | d12 (198 / 316) | never | 0 | 15720 (d12 06:00) | 105000 (d74 06:00) (1.541) | 29400 (d21 18:00) |
| 12005 | 42 / 40 | d12 (254 / 307) | never | 0 | 15720 (d12 06:00) | 103560 (d73 06:00) (1.541) | 29400 (d21 18:00) |
| 12006 | 0 / 0 | — | never | 0 | — | — | 29400 (d21 18:00) |
| 12007 | 38 / 44 | d29 (232 / 441) | never | 0 | 40200 (d29 06:00) | 106440 (d75 06:00) (1.651) | 29400 (d21 18:00) |
| 12008 | 28 / 14 | d12 (182 / 297) | 15720 (d12 06:00) | 1 | 56040 (d40 06:00) | — | 29400 (d21 18:00) |
| 12009 | 41 / 40 | d12 (338 / 355) | 171240 (d120 06:00) | 1 | 15720 (d12 06:00) | 107880 (d76 06:00) (1.594) | 29400 (d21 18:00) |
| 12010 | 0 / 5 | — | never | 0 | — | 21480 (d16 06:00) (1.836) | 29400 (d21 18:00) |
| 12011 | 34 / 28 | d29 (199 / 364) | 45960 (d33 06:00) | 1 | 40200 (d29 06:00) | 25800 (d19 06:00) (1.585) | 29400 (d21 18:00) |
| 12012 | 33 / 49 | d29 (222 / 326) | never | 0 | 40200 (d29 06:00) | 18600 (d14 06:00) (1.629) | 29400 (d21 18:00) |
| 12013 | 16 / 5 | d12 (276 / 384) | 41640 (d30 06:00) | 2 | 15720 (d12 06:00) | — (1.168) | 29400 (d21 18:00) |
| 12014 | 14 / 8 | d12 (302 / 384) | 48840 (d35 06:00) | 1 | 15720 (d12 06:00) | 25800 (d19 06:00) (1.505) | 29400 (d21 18:00) |
| 12015 | 0 / 0 | — | never | 0 | — | — | 29400 (d21 18:00) |
| 12016 | 19 / 23 | d29 (228 / 480) | 44520 (d32 06:00) | 1 | 40200 (d29 06:00) | 25800 (d19 06:00) (1.500) | 29400 (d21 18:00) |
| 12017 | 26 / 36 | d12 (340 / 412) | 15720 (d12 06:00) | 1 | 80520 (d57 06:00) | — | 29400 (d21 18:00) |
| 12018 | 26 / 30 | d29 (244 / 480) | 120840 (d85 06:00) | 2 | 40200 (d29 06:00) | 28680 (d21 06:00) (1.551) | 29400 (d21 18:00) |
| 12019 | 29 / 33 | d29 (234 / 422) | 47400 (d34 06:00) | 1 | 40200 (d29 06:00) | 24360 (d18 06:00) (1.623) | 29400 (d21 18:00) |
| 12020 | 27 / 35 | d29 (317 / 480) | 130920 (d92 06:00) | 1 | 40200 (d29 06:00) | 69000 (d49 06:00) (1.483) | 29400 (d21 18:00) |
| 12021 | 15 / 19 | d12 (357 / 393) | 50280 (d36 06:00) | 3 | 15720 (d12 06:00) | 27240 (d20 06:00) (1.557) | 29400 (d21 18:00) |
| 12022 | 34 / 28 | d12 (288 / 336) | 51720 (d37 06:00) | 1 | 15720 (d12 06:00) | 27240 (d20 06:00) (1.491) | 29400 (d21 18:00) |
| 12023 | 37 / 45 | d29 (169 / 297) | never | 0 | 40200 (d29 06:00) | 109320 (d77 06:00) (1.663) | 29400 (d21 18:00) |
| 12024 | 49 / 32 | d12 (225 / 316) | 171240 (d120 06:00) | 1 | 15720 (d12 06:00) | 105000 (d74 06:00) (1.489) | 29400 (d21 18:00) |
| 12025 | 35 / 20 | d12 (244 / 278) | 48840 (d35 06:00) | 1 | 15720 (d12 06:00) | 20040 (d15 06:00) (1.252) | 29400 (d21 18:00) |
| 12026 | 34 / 28 | d12 (360 / 393) | 132360 (d93 06:00) | 1 | 15720 (d12 06:00) | 24360 (d18 06:00) (1.522) | 29400 (d21 18:00) |
| 12027 | 27 / 29 | d12 (286 / 326) | 123720 (d87 06:00) | 1 | 15720 (d12 06:00) | 103560 (d73 06:00) (1.571) | 29400 (d21 18:00) |
| 12028 | 0 / 0 | — | never | 0 | — | — | 29400 (d21 18:00) |
| 12029 | 49 / 33 | d12 (326 / 393) | never | 0 | 15720 (d12 06:00) | 67560 (d48 06:00) (1.512) | 29400 (d21 18:00) |
| 12030 | 33 / 49 | d29 (261 / 364) | never | 0 | 40200 (d29 06:00) | 61800 (d44 06:00) (1.596) | 29400 (d21 18:00) |
| 12031 | 22 / 20 | d12 (402 / 412) | 90600 (d64 06:00) | 2 | 15720 (d12 06:00) | 28680 (d21 06:00) (1.483) | 29400 (d21 18:00) |

Per-seed lists of every eligible home check (display day, food, line, ratio, s) are in `timeline.json`.

Rule notes (review round 1, L2): the after-first-departure clause also applies when shortfall checks exist but none lies at least 3 days before the first departure; a candidate time outside [fork tick, end − 4 d) is skipped for the next one; cancelled departures count as departures; a seed that never departs has no departure bound on T_high. The three names are store states, not a time order: T_low falls on day 12 (the first eligible check) or day 29 (the first spring check, after T_eve), and T_high often falls after T_low.
