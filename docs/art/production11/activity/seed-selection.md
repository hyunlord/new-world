# Art Pass 11 활동 캡처 시드 선정

선정 시드: **20261018**. 종료 시점 인구 **92**, 캡처 시점 인구 **92**.

`game_v24`, release probe, 20261011..20261027(상한 제외) 16개 모두 완료. day60/tick86400의 day 표본 인구 최댓값으로 선정하고 동률이면 작은 시드를 택했다. 캡처용 달력 60일 10:00은 tick85080의 clock 표본이며 선정용 종료 시점과 분리했다.

| 시드 | 선정 시점 인구 (86400) | 캡처 시점 인구 (85080) |
|---|---:|---:|
| 20261018 | 92 | 92 |
| 20261022 | 75 | 75 |
| 20261024 | 68 | 68 |
| 20261013 | 66 | 66 |
| 20261017 | 63 | 63 |
| 20261021 | 52 | 50 |
| 20261025 | 44 | 44 |
| 20261023 | 39 | 41 |
| 20261015 | 37 | 38 |
| 20261016 | 37 | 37 |
| 20261026 | 32 | 33 |
| 20261014 | 30 | 29 |
| 20261011 | 27 | 27 |
| 20261019 | 26 | 26 |
| 20261012 | 24 | 24 |
| 20261020 | 21 | 21 |

## 검증과 출처

- summary scene/range/runs 16개, 모든 raw row의 scene/seed, 시드별 종료 및 clock 표본 각1개, summary alive_daily 마지막 값과 raw endpoint 일치 확인.
- summary day60 최대 인구와 선정 결과 일치 확인. 전체 raw/summary/build 로그 SHA256은 `selection.json`에 기록.
- 인접 기록의 빌드 HEAD: `ecf512d886d87ba7f9ca5fa7db92a48300dc1c68`.
- release binary SHA256: `5becc894060eaaf4d88c09abcf4c1edeca8edc3c9d006c1a4873f0c81e9b404b`. 현재 바이너리와 기록값 일치.
- **embedded git identity NOT_VERIFIED**: raw row의 `git_sha`는 모두 `unknown`. 위 HEAD는 인접 실행 기록이며 바이너리 내부에 검증된 SHA로 해석하지 않는다.
- 리더의 session58452 성공 종료 보고와 별도로 16개 완료 endpoint를 검증했다. 별도 exit-code 파일은 없어 프로세스 종료값을 새로 발명하지 않았다.
- probe의 `determinism_checked=false`: 별도 쌍실행 결정론 검사를 수행했다는 주장은 하지 않는다.
- 원본: `/Users/rexxa/work/art-production11-evidence/activity-probe/`. 실행 중 cargo/Godot 추가 실행 없음.
