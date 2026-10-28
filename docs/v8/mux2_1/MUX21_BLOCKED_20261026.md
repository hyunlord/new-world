## 구현 완료 보고 — M-UX2.1

**미완료 · BLOCKED · NOT_PUBLISHED.** 보고서 제목은 오너 지정 양식이다. 성능 수정 완료를 뜻하지 않는다.

공유 Mac의 부하가 측정 조건을 반복해서 위반했다. 유효한 필수 비교는 **0/12**다. 여러 연속 목표 턴에서 같은 외부 제약을 재확인한 뒤, 소유한 대기 실행기만 종료했다. 다른 세션의 프로세스에는 신호를 보내지 않았다. 6시간을 소진해서 종료한 것은 아니다. 시작은2026-10-26 08:32:12UTC, 원래 hard stop은14:32:12UTC다.

### 원인 (빌드 3개 비교, 이분 탐색으로 찾은 커밋, 프로파일 상위 5개)

- 세 빌드 `3a51636d`, `0207b497`, BASE `9855bd082`의 독립 release bridge와 동일한 공통 하네스를 준비하고 실제 창에서 짧은 호환성 검사를 통과했다. 이것은 필수30초 준비+60초 FPS 비교를 대체하지 않는다.
- 첫 장시간 M-ART2.1 N10 실행은 부하6.80→13.05 때문에 **INVALID**다. 477.529879FPS는 유효한 비교값이나 합격 근거로 사용하지 않는다. 초기10명 요청의 종료 인구는11명이었다. 원본 영수증을 보존했다.
- 이분 탐색은 **미실행**이다. 정적 후보는 M-UX2 p6 `1c8d2979e`의 매 프레임 `Layout.place()`다. 기존 마커는 이미 MultiMesh와 캐시된 텍스처를 사용한다. p4/p5/p6/p9 비교본을 준비했지만 범인 커밋으로 확정하지 않았다.
- BASE N200·¼× 진단은30초 준비+60.009742초,2054표본이다. 부하48.19→28.77의 **혼잡 진단**이므로 FPS 수용 근거가 아니다. 아래는 계측한 서로 겹치지 않는 주요 구간이며 전체 엔진/GPU 비용의 완전한 분해가 아니다.

| 계측 구간 | 평균 ms/프레임 |
|---|---:|
| 마커 갱신 전체 | 25.053 |
| 사람 상태 동기화 | 0.333 |
| 지형 처리 | 0.203 |
| 사람 정리·배치 준비 | 0.142 |
| 사람 파라미터·스냅숏 | 0.109 |

건물0.040ms, HUD0.037ms, 미니맵0.032ms, 알림0.009ms다. draw calls335.73, objects drawn1384.15, Performance process32.55ms다. 중첩된 agent 전체25.647ms를 위 구간에 다시 더하지 않는다. 측정 창의 chunk bake는0회였으므로 자연 재굽기 최대값은 **NOT_MEASURED**다.

### 고친 것

제품 렌더러와 UI는 **아직 수정하지 않았다**. 비교 도구와 증거 검증을 준비했다.

- BASE 하네스의 렌더 프레임·카메라·시간 절차를 공유하되 과거 빌드의 없는 bake 계측을 명시적으로 구분한다.
- 인구는 렌더 이후 집계한다. 초기 인구와 측정 중 최소·최대를 기록하고, 정확히 N명을 유지했는지 별도로 표시한다. 인구가 증가한 실행은 정확한 N명 성능의 증명이 아니다.
- 과거 소스는 전용 worktree 아래 보관한다. 독립 target 빌드, Git revision 내용 비교, 매핑할 bridge와 빌드 산출물의 일치, 실행 전후 입력 해시, 창·프레임·FPS 비율·bake 원본 해시를 검증한다.
- 공유 Cargo target이 서로 다른 과거 소스에 같은 바이너리를 재사용한 첫 준비 결과는 폐기 표시하고 보존했다. 독립 빌드로 교체했다.

### FPS 전후 표 (인구 10/40/200/1,000 × ¼×·½×·1×, 막대 판정)

전 값은 기존 `MUX2_OWNER_MEASUREMENTS.json`의 역사 기록이다. 후 값은 모두 미측정이며, 어떠한 막대도 이번 작업에서 PASS로 판정하지 않았다. `—`는 대응하는 유효 값이 없다는 뜻이다.

| 요청 인구 | ¼× 전 → 후 / 기준 | ½× 전 → 후 / 기준 | 1× 전 → 후 / 기준 |
|---:|---|---|---|
| 10 | — → 미측정 / ≥144 | — → 미측정 | — → 미측정 |
| 40 | — → 미측정 / ≥144 | — → 미측정 | — → 미측정 |
| 100 (추가 필수) | — → 미측정 | 164.9 → 미측정 / ≥150 | — → 미측정 |
| 200 | 35.1 → 미측정 / ≥120 | 119.3 → 미측정 / ≥105 | 225.8 → 미측정 / ≥200 |
| 1,000 | — → 미측정 / ≥100 | 35.98 → 미측정 (기존 실패 보존) | — → 미측정 |

별도 역사 참조: M-ART2.1 N1000·¼×137.5FPS. 이번 재측정값이 아니다. `stress N`은 초기 인구 요청이며 실제 인구 범위와 `population_exact`를 보고해야 한다. 정확한 인구 조건이나 서로 다른 빌드의 비교 가능성은 아직 입증되지 않았다.

### 가시성 재확인

M-UX2 GPU 낮/밤140px 대비, 선택·소집 테두리 검사는 **미실행**이다. 제품 소스가 BASE와 동일하다는 Git 확인은 새 시각 검증을 대체하지 않는다. 자연 재굽기의4ms/프레임 판정도 미실행이며, 기존 하네스의 chunk별 최대값으로 프레임 합계를 대신하지 않는다.

### 닫은 GitHub 이슈 / 새로 연 이슈

- 닫은 이슈: 없음.
- 신규 [#638](https://github.com/hyunlord/new-world/issues/638), 원장ISS-343: **OPEN**. 수정·게시되지 않았으므로 닫지 않았다.

### 변경된 파일 목록 (커밋별)

| 커밋 | 파일 |
|---|---|
| p0 `1b3518941` | `docs/v8/mux2_1/MUX21_GOAL.md` |
| p1 `7f3c95378` | `docs/ISSUES.md`, `docs/v8/mux2_1/MUX21_DECISIONS.md`, `tools/v8/mux21_fps.gd` |
| p2 `01f4f311e` | `docs/v8/mux2_1/MUX21_DECISIONS.md`, `tools/v8/mux21_profile.gd` |
| p3 `824488050` | `docs/v8/mux2_1/MUX21_DECISIONS.md`, `tools/v8/mux21_fps.gd` |
| p4 본 차단 보고 정리 | 본 보고서, `MUX21_DECISIONS.md`, `CHANGELOG.md`, `docs/HISTORY.md`, `docs/ROADMAP.md`, `.claude/skills/worldsim-pitfalls/SKILL.md` |

### 확인된 제한사항

- 원래 범위는 그대로 남아 있다: 유효한3빌드×4인구 비교, 실제 이분 탐색, 원인 확인·수정, 모든 FPS 막대, GPU 가시성, 자연 재굽기, 전체 gate, 게시가 미완료다.
- `scripts/`, `scenes/`, `shaders/`, `assets/`, `localization/`과 보호된 세 Rust crate의 BASE 대비 diff는0이다. 전체 identity·colony fixture gate를 실행했다는 뜻은 아니다.
- 공통 인구 계측은 렌더 이후 SimReads를 읽으므로 추가 관측 비용이 있다. 무계측 BASE와 완전히 같은 비용이라는 주장은 하지 않는다.
- 소유한 실행기69103은 차단 정리 과정에서 정상적으로 종료 신호를 받아 exit143이 됐다. 활성 FPS 측정은 없었다. 다른 프로세스에는 신호를 보내지 않았다.
- worktree는 미게시 브랜치·재개용 비교본·증거를 보존하기 위해 남겼다. 게시 뒤 제거 단계에 도달하지 않았으며 `--force`는 쓰지 않았다. `.gitignore` 수정과 `.ignore`는 외부 도구 변경으로 보존하고 스테이징하지 않았다.

원본 증거: `/Users/rexxa/work/mux21-data/`. 핵심 파일은 `comparison/mart21-n10-z0.25-a1/receipt.json`, `base-profile-contended/profile.json`, `admission-watch.log`, `instrumentation-strict.log`, `harness-postdraw-strict.log`, `provenance-controls/`, `product-unchanged.txt`, `remote-blocked.txt`다. 재개 시 기존 영수증·작업트리·원격·부하와 원래 시간 제한을 재확인해야 한다.

### Gate 결과

**전체 gate: NOT_RUN. 게시: NOT_PUBLISHED.** 짧은 창 실행과 계측 도구의 엄격 GDScript/FFI 검사는 통과했지만 전체 gate나 성능·시각 수용을 대신하지 않는다. 문서 검사는 **PASS** (`blocked-docs-check-rebuilt.log`, exit0). 첫 검사는 과거 빌드가 남긴 release 산출물 불일치로 시스템 표 생성에 실패했다. 소유 작업트리의 해당 로컬 패키지 산출물을 정리하고 BASE 소스로 재빌드한 뒤 통과했으며, 첫 실패 로그도 보존했다. 목표 원문 일치와 `git diff --check`도 통과했다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

현재 게시된 기존 M-UX2를 실행하는 명령이다. M-UX2.1 수정판은 게시되지 않았다.

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

git ls-remote 결과 (`remote-blocked.txt`, 차단 정리 시 조회):

```text
9855bd082ad88a4c49296410f206f148d98c4df9 refs/heads/v8/main
9855bd082ad88a4c49296410f206f148d98c4df9 refs/heads/v8/wip
```
