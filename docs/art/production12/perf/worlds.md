# Part 0 runtime worlds

All rows reproduced by reset and exact advance on tooling commit `a4901cb12e29afb74783039ed66aef4a74014148`. Live reset uses game_v25; stress reset uses game_v19. This census is identity/state evidence, not frame measurement.

| World | Seed | Tick | Population | Occupied villages |
|---|---:|---:|---:|---:|
| W0 | 20261010 | 40560 | 29 | 1 |
| W1 | 20261013 | 126994 | 66 | 2 |
| W200 | 15100 | 1440 | 200 | 1 |
| W500 | 15100 | 1440 | 500 | 1 |
| W-multi | 19400 | 121080 | 65 | 2 |

W-multi has two outbound food porters, each departure amount 10, village 2 → 1. Runtime clock is day index 84 at 10:00, matching the source-selected trip. P2 has one lit hearth at 02:00; P3 and P4 have two occupied villages. P5 retains 500 people. `OS.has_feature("double")` is false.

Raw census: `/Users/rexxa/work/art-production12-evidence/identity-v2/derivation/all.json`
SHA256: `5ff97573dad1dd7df1b69e4eaa54816d0110de7e15db7be99ec9ecc19e01dad9`
Loaded bridge profile: dev opt-level=2; SHA256 `676c48942f42d6677a74b6837e64ec1237ff07673361cb4db1780d884666e4d2`.
Census complete=true, failures=[]; terminal exit 0.

The earlier default-dev census remains incomplete under `identity/derivation/`; it is excluded.

Reconfirmed after corrected tooling checkout and build/import. Initial 9934f785 census remains preserved under identity/derivation-opt2/all.json; all world populations and occupied-village counts agree.
