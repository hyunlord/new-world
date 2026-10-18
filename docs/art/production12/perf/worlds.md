# Part 0 runtime worlds

All rows reproduced by reset and exact advance on tooling commit `9934f78513cd1657a5df8fa2f7854b9df9e02e95`. Live reset uses game_v25; stress reset uses game_v19. This census is identity/state evidence, not frame measurement.

| World | Seed | Tick | Population | Occupied villages |
|---|---:|---:|---:|---:|
| W0 | 20261010 | 40560 | 29 | 1 |
| W1 | 20261013 | 126994 | 66 | 2 |
| W200 | 15100 | 1440 | 200 | 1 |
| W500 | 15100 | 1440 | 500 | 1 |
| W-multi | 19400 | 121080 | 65 | 2 |

W-multi has two outbound food porters, each departure amount 10, village 2 → 1. Runtime clock is day index 84 at 10:00, matching the source-selected trip. P2 has one lit hearth at 02:00; P3 and P4 have two occupied villages. P5 retains 500 people. `OS.has_feature("double")` is false.

Raw census: `/Users/rexxa/work/art-production12-evidence/identity/derivation-opt2/all.json`
SHA256: `586c6dac73cd64c704c4ddf0a3bfc80010f409e762ac1391a051b4132a2b1f94`
Loaded bridge profile: dev opt-level=2; SHA256 `676c48942f42d6677a74b6837e64ec1237ff07673361cb4db1780d884666e4d2`.
Census complete=true, failures=[]; terminal exit 0.

The earlier default-dev census remains incomplete under `identity/derivation/`; it is excluded.
