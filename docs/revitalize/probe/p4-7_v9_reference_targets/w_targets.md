# Pass 4 W targets

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| W1 | twins_10 | PASS | 64 seeds, determinism checked | 64 |
| W1 | twins_120 | PASS | 32 seeds, determinism checked | 32 |
| W1 | legacy_pin | PASS | legacy_pin verified externally | true |
| W1 | v8_repro | PASS | v8_repro verified externally | true |
| W1 | v9_repro | PASS | v9_repro verified externally | true |
| W2 | day_10 | PASS | median 0, max <= 1 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| W2 | day_120 | PASS | total median 0, max <= 2 | {"max": 1.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W3 | day_10 | PASS | zero in every seed | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| W3 | day_120 | PASS | total median 0, max <= 3 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W4 | every_day | PASS | one settlement at every day sample from day 1 | {"max": 1.0, "median": 1.0, "min": 1.0, "n": 4480, "p10": 1.0, "p90": 1.0} |
| W5 | birth_by_day4 | FAIL | births >= 1 by the day-4 checkpoint in every seed (10-day round) | {"max": 7.0, "median": 7.0, "min": 0.0, "n": 64, "p10": 3.0, "p90": 7.0} |
| W5 | alive_day10 | FAIL | day-10 alive >= 30 in every seed | {"max": 42.0, "median": 42.0, "min": 24.0, "n": 64, "p10": 31.0, "p90": 42.0} |
| W5 | stagnation | FAIL | longest run of days 2..N with members <= sustainable-3 and no birth <= 4 in every seed | {"max": 39.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 15.800000000000004} |
| W6 | old_age_by_day30 | FAIL | >= 1 old-age death by day 30 in every seed | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | replacement | PASS | births - deaths over days 30..120 >= -3 in every seed | {"max": 18.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 7.800000000000004} |
| W6 | alive_band | FAIL | every day sample from day 30 has alive in [32, 44] | {"max": 42.0, "median": 42.0, "min": 24.0, "n": 2912, "p10": 42.0, "p90": 42.0} |
| W6 | child_share | FAIL | day-120 child share median in [0.03, 0.20] | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W6 | elder_share | FAIL | day-120 elder share median in [0.08, 0.35] | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| W7 | autumn_store | FAIL | autumn-closing store; not applicable without seasons | "no autumn observed (seasons off or run too short)" |
| W7 | hoard_cap | FAIL | max stock - 3 x max target <= 0 in every seed | {"max": 1798.0, "median": 486.0, "min": 59.0, "n": 32, "p10": 91.5, "p90": 1259.9000000000003} |
| W8 | noon_idle | FAIL | noon idle share median <= 0.45 over spring-autumn days (all days without seasons) | {"max": 0.7380952380952381, "median": 0.47619047619047616, "min": 0.30952380952380953, "n": 32, "p10": 0.3333333333333333, "p90": 0.7014800514800515} |
| W8 | forage_trips | FAIL | forage trips per day median >= 10 in summer and autumn (all days without seasons) | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 3840, "p10": 0.0, "p90": 0.0} |
| W9 | night_in_shelter | PASS | median >= 0.70 | {"max": 1.0, "median": 0.9755952380952381, "min": 0.9047619047619048, "n": 32, "p10": 0.9285714285714286, "p90": 1.0} |
| W9 | evening_near_hearth | FAIL | median >= 0.50 and every seed >= 0.30 | {"max": 0.9, "median": 0.5857699805068226, "min": 0.13043478260869565, "n": 32, "p10": 0.3176136363636364, "p90": 0.7923645320197045} |
| W10 | home_distance | FAIL | day-10 median <= 7.0, every seed <= 10 | {"max": 10.523809523809524, "median": 5.35632183908046, "min": 2.7560975609756095, "n": 64, "p10": 3.626190476190476, "p90": 7.938095238095238} |
| W10 | t6a_10 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.0025176233635448137, "median": 0.0005260964219290539, "min": 0.0, "p10": 0.0, "p90": 0.0013145006543399198} |
| W10 | social_10 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.7969918805881581, "median": 0.6451207610987819, "min": 0.5394135803790451, "n": 64, "p10": 0.5941736923323326, "p90": 0.7157097661737061} |
| W10 | t6a_120 | PASS | cumulative actionable idle-breach share median <= 1% | {"max": 0.005516220716099628, "median": 0.0007952985136236361, "min": 0.00011786297587176509, "p10": 0.00019868775151588461, "p90": 0.003170734756143343} |
| W10 | social_120 | PASS | median >= 0.40 and > 0 in every seed | {"max": 0.8752209476939709, "median": 0.8411430474327553, "min": 0.7817253379617417, "n": 32, "p10": 0.8156339562251139, "p90": 0.8626073369755005} |
| W11 | day10 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0011702719737690642, "max": 0.0009435559389303046, "median": 0.0007801813158460428, "min": 0.0007089738996171232, "n": 64, "p10": 0.0007372038218251793, "p90": 0.0008803190232728914} |
| W11 | day120 | PASS | median ms/agent-tick <= 1.5 x same-host Pass 3 game_v9 | {"limit": 0.0013351547707594372, "max": 0.0011330299675527445, "median": 0.0008901031805062915, "min": 0.0007715433659115464, "n": 32, "p10": 0.000792192145629411, "p90": 0.0009941436743588895} |
