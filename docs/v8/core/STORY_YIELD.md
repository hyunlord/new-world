# M-CORE 이야기 수확량

상태: **MEASURED — 참고용 측정 완료, 수치 수용 막대 없음**. `heldout-final-1/session-terminal.json`은 `CAPTURE_AND_CHECKS_PASS`, 일곱 하위 실행 모두 exit0이다. 전체 M-CORE gate·성능·시각·게시 수용을 뜻하지 않는다.

## 비교 입력과 정의

각32세계(seed1232–1263)×48일은 각각 **64개 완전한 world-year**다. 각 연도 observed_ticks는34,560이며 부분 연도 외삽은 없다. 깊이는 표시용 라벨이 아니라 검증된 record 간선 수이고 최소3이다. kind 경로는 ID·미해결 suffix·종료 표지를 제외한 관측된 순서이며 완전한 역사를 추정하지 않는다. 누락 ancestry는 확인된 깊이까지만 남기고 dangling target은 간선을 더하지 않는다. 순환은 제외하되 따로 보고한다. 인물 지표는 연도마다 ≥3항목에 등장한 서로 다른 사람 수이며 여러 연도 합계를 전역 고유 인물 수로 읽지 않는다.

BASE는 **기존 runtime에 동결 final2 관측 계기를 더한 빌드**로 정확한 BASE 소스 빌드와 구분한다. source `e5835e51760fc08808b324e7cb62c718ee79fa22`, binary `1ccfab77e5342e06a7df28830182ef618a90860fe0a2958b88755007ee66de6a`. M-CORE는 p7 `be12050c951dfb025f3440d34b657eec0663b817`, binary `772a193fd2ebcfff7823c41068b75ea4a296fb2fb8d6d4fedd100f62490d7d41`이다.

## 결과

| 지표 | BASE 합계 | M-CORE 합계 | BASE / world-year | M-CORE / world-year |
|---|---:|---:|---:|---:|
| 연대기 항목 | 7676 | 19722 | 119.9375 | 308.15625 |
| 검증 깊이≥3 항목 | 0 | 0 | 0.0 | 0.0 |
| 서로 다른 깊은 kind 경로 | 0 | 0 | 0.0 | 0.0 |
| 연도별 ≥3항목 등장 인물 | 556 | 651 | 8.6875 | 10.171875 |
| 누락 ancestry 사슬 | 121 | 125 | 1.890625 | 1.953125 |
| 순환 사슬 | 0 | 0 | 0.0 | 0.0 |

양쪽 깊은 항목/사슬이 0이라는 유효 측정은 깊은 이야기가 성공했음을 증명하지 않는다. 전체 항목 증가도 깊이나 질 개선으로 등치하지 않는다. 누락 사슬121/125를 그대로 유지한다. `numerical_bar:null`, `acceptance_claim:false`이며 결과는 참고용이다.

## 원시 증거

루트 `/Users/rexxa/work/core-data/heldout-final-1/`의 `story.json`, `story-{base,mind}-receipt.json`, 각 story 원시 보고서와 exit/log를 보존한다.

- story.json SHA-256 `7fb7a6ceb55ac1d08cd5667e679e7e3064e19c8aa21761e8c8bede594970d9ce`.
- base report SHA-256 `c994e2c754e6247bf9f140efa47b8959e7b7140ee156fc9ba7ece108595e0417`, receipt `9eee7c0ee6f83f8eb6f292a0907afd898cdc2b45facaf170525d0ab40ea8578e`.
- mind report SHA-256 `669da9bf81fe046a8d4b2ae274058bc46f0f6225a3e26f626d9106766a2b2af5`, receipt `bf6f3022ed50fab67252e3945e3801c42f9c76150efc66eb2d05ea513e55ed44`.
- BASE instrument provenance `3940186bc40e5447c5b467a53906828edc2b47398e8c9db9fa5acee36d412e1e`; checker `e308e867f915d56ebdc80154011de0e9914f565546ba89772b63fcfcadfca03e`.
