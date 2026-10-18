# Stage B 최종 동일성 증거 검증

**PASS** — I1·I2·I3·I4·I5 및 v19/v20 핀의 7개 검사. 생성 UTC: 2026-10-15T10:47:16.785753Z.

원본 [identity.json](/Users/rexxa/work/new-world-pass4/docs/revitalize/verification/p17-5_b_identity/identity.json) SHA256 `6b627b091095b84f1f207ddadd77706ce068dc1ef431f18edf60bb7d83c7d7a4`.
[전체 감사 JSON](b_identity_final_audit.json) SHA256 `85649459532643ef0e1f8453390fe319fce60f1d6262e527a729a2b164c3b881`. 전체 입력 401개 파일의 경로·바이트·해시는 JSON에 수록한다.

- manifest 참조 73개 / 고유 증거 파일 31개 해시 확인.
- 보존된 후보 바이너리 5개, Stage A 바이너리 5개, launch 참조 40개 해시 확인.
- 원본 파일 306개 전부 재해시; 디렉터리와 원장의 차이는 원장 자체 `raw_sha256.txt`뿐.
- 생산자·launch·감사 시점의 Rust 트리 동일, 관련 커밋 선후 관계 및 기록된 Stage A I5 기준 요약 일치.

| 검사 | 장면 | 세계·일수 | 비시간 차이 | 종료 UTC |
|---|---|---|---:|---|
| I1 | game_v19 | seeds 0–31, 32 × 120일 | 0 | 2026-10-15T10:15:27Z |
| I3 | game_v16 | seeds 0–31, 32 × 120일 | 0 | 2026-10-15T10:19:59Z |
| I4 | game_v20 | seeds 0–31, 32 × 120일 | 0 | 2026-10-15T10:27:22Z |
| I5 | game_v21 | seeds 0–31, 32 × 120일 | 0 | 2026-10-15T10:43:32Z |

네 요약은 모두 32개 고유 seed, 1–120 day key, seed별 120일 시계열, `determinism_checked=true`, 로그 마지막 `DETERMINISM PASS`를 만족한다. 저장소 요약은 원본 요약과 바이트 단위로 같다. 기존 엄격 비교기의 세 timing 키만 제외했다.

I2의 20개 항목(18 NPY + 2 interventions.json)은 기준과 후보의 실제 파일 해시까지 모두 같다. v19/v20 핀은 각각 정확히 3 passed / 0 failed / 0 ignored이며 두 종료 코드는 0이다.

최종 workload 종료: `2026-10-15T10:43:47.947316Z`, exit 0. 완료 후의 RUNLOG·바이너리 검증 영수증·raw manifest 해시가 identity.json과 일치한다.

## 생산자와 launch

- Stage B 생산자: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`
- launch: `c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee`
- 감사 시점 HEAD: `43f22e81f120a755de33712b02a62469c67ab840`
- Stage A 생산자: `50a73474e06d3a9abbb535729e9c7e1e134954b7`
- Stage A 판정/I5 커밋: `44aebafa5acf5668d21418f611d8f0ce3a5d13d3`

## 보존된 후보 바이너리

| 파일 | SHA256 |
|---|---|
| `worldsim_probe` | `2bd672960600f60a58114849037a55fb74be96fefa278e77186dd849996b4ec0` |
| `worldsim_wm` | `ef87df3104109484e61befb20f3c3e4b7ae2e6732cade4e40d18f3f4fbedceb6` |
| `worldsim_perf` | `89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f` |
| `v19_pins` | `e2e5ee5adf2844dae9ed1794de0b42e5454b36184103b25b5fb6258626c0d6f9` |
| `v20_pins` | `bb523f82e37a5b48b97b6064b429969faf6805ee3c44243ae9b2abca37f888d5` |

새 프로브·빌드·테스트·평가기·성능 실행은 하지 않았다. 저장소/runner 수정 없이 낮은 우선순위의 파일 읽기·해시 검증만 수행했다. 이 작업은 Part 8 독립 최종 리뷰가 아니다.
