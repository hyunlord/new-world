# ENV1 장면 출처와 재현

판정: **실제 장면 세 개 확보**. 192×192 원본 ASCII와 사람 위치 다중집합을 별도 읽기 전용 추출 결과와 대조하여 모두 일치했다. 시뮬레이션 소스는 수정하지 않았다. 최종 합성의 M3–M5 배치 및 사람 외형·활동 재지정은 별도 선언 오버레이이며 이 실제 자료와 구별한다.

## 고정한 소스

- `v8/wip` 조사 시점 HEAD: `3a3b236c125c0c25e9b74261be88536473fc01e9`.
- 분리된 detached worktree: `/Users/rexxa/work/artenv1-data/probe-worktree`.
- 원본 데이터·로그·빌드 산출물: `/Users/rexxa/work/artenv1-data/`.
- `rust/Cargo.lock`은 같은 HEAD의 `/Users/rexxa/work/new-world-pass4/rust/Cargo.lock`에서 복사했다. 두 HEAD와 Cargo.toml 차이 없음 확인. SHA-256: `37fe61094dc5e6e582c8b9c6e303bffc679a86f2e16c5b9361f9f78ab9398dcb`.
- 명령 스크립트: `tools/v8/soak_script.json`, SHA-256 `0548d0b141f61950daee04baf78d13ca1e3ae287fcb6f0a6577e9b8006e052f0`.
- 빌드: `CARGO_BUILD_JOBS=8 cargo build --locked --release -p sim-test --bin worldsim_probe` (worktree의 `rust/`에서 실행).
- native probe SHA-256: `b425da5f8f43cbe2c900ad753230be9cdf40eae8fada467de6ea650bae258fd0`.
- 추출기 SHA-256: `04f7d4739a5002a25992831dc608c410648a00a6146e910a771adbfdda32d3fd`.

초기 빌드 명령은 작업 디렉터리를 잘못 계산한 lock 복사 때문에 `--locked` 오류로 종료했다. 정확한 경로로 lock을 복사한 뒤 빌드 성공을 확인했다. 시뮬레이션 동작이나 소스는 변경하지 않았다.

## 날짜

식: `UI 날짜 = floor((T + 480) / 1440) + 1`, `시각 분 = (T + 480) mod 1440`.
colony는 6일×4계절, 시작 시각 08:00이다. 목표의 day30은 겨울이 아니므로 허용된 대로 첫 겨울 정오를 사용한다.

| 장면 | 완료 tick | UI 날짜·시각 | 계절 | 사람 | 잠든 사람 | 실제 운반 |
|---|---:|---|---|---:|---:|---:|
| `scene_day6_noon` | 7440 | 6일째 12:00 | 봄 6일째 | 10 | 0 | 0 |
| `scene_day6_night` | 6840 | 6일째 02:00 | 봄 6일째 | 10 | 10 | 0 |
| `scene_winter_noon` | 26160 | 19일째 12:00 | 겨울 1일째 | 10 | 0 | 0 |

모든 장면의 실제 화덕 중심은 `(91,101)`이고 사람은 성인 10명(여성 7·남성 3)이다. 렌더 화면 안 사람 수는 줌·HUD 가림·화면 밖 위치로 달라지므로 전역 인구 10을 화면 정답으로 그대로 사용하지 않는다.

## 실제 실행 명령

아래 명령을 native worktree 루트에서 실행했다. `NAME,T` 세 쌍은 위 표의 `day6_noon,7440`, `day6_night,6840`, `winter_noon,26160`이다. 변수 치환 후의 완전한 명령은 각 committed scene JSON의 `provenance.command`와 `provenance.extract_command`에 보존되어 있다.

```sh
./rust/target/release/worldsim_probe --scene colony --seeds 0..1 --ascii --tick T --commands tools/v8/soak_script.json --out /Users/rexxa/work/artenv1-data/scenes/scene_NAME.txt --people-json /Users/rexxa/work/artenv1-data/scenes/people_NAME.json
./rust/target/release/env1-probe-extract 0 T tools/v8/soak_script.json /Users/rexxa/work/artenv1-data/scenes/scene_NAME.txt /Users/rexxa/work/artenv1-data/scenes/people_NAME.json /Users/rexxa/work/artenv1-data/scenes/rich_NAME.json
```

추출기 빌드와 포장(art worktree 루트):

```sh
CARGO_TARGET_DIR=/Users/rexxa/work/artenv1-data/probe-worktree/rust/target CARGO_BUILD_JOBS=8 cargo build --offline --release --manifest-path art/compositor/probe_extract/Cargo.toml
python3 art/compositor/source_scene.py /Users/rexxa/work/artenv1-data
```

추출기 `Cargo.toml`의 절대 path 의존성은 위의 고정 detached worktree만 가리킨다. 다른 머신에서는 같은 SHA를 checkout한 뒤 이 path를 조정해야 한다. 기존 시뮬레이션 crate와 이미 사용 중인 serde/serde_json만 재사용한다. 빌드 target은 저장소 밖이다.

## native CLI의 실제 계약과 보충

`--people-json`은 `{seed,tick,count,people:[[x,y],...]}`만 내보낸다. 성별·AgentId·생애 단계·활동이 들어 있다는 목표의 가정은 현재 CLI와 다르다. 또한 ASCII는 앞 레이어가 뒤 레이어를 가리는 단일 문자 그리드다.

이를 추측으로 채우지 않기 위해 `art/compositor/probe_extract/`가 기존 `ascii::engine_at`을 그대로 호출한다. 동일 seed·tick·script로 다시 만든 상태의 `ascii::render_with`가 native ASCII와 바이트 단위로 같고, `ascii::people_tiles`가 native JSON의 정렬된 위치 다중집합과 같은 경우에만 다음을 내보낸다.

- `collect_terrain_snapshot`: 물체 아래를 포함한 실제 지형 전체(`terrain_rows`).
- `collect_agent_snapshot` + `collect_agent_jobs`: 진짜 AgentId·좌표·생애 단계·상태·목표·작업·운반 종류와 수량.
- `collect_agent_family`(entity bits로 조회): 기록된 성별. entity bits를 AgentId로 오인하지 않는다.
- `collect_time_snapshot`, 실제 화덕 건물 위치.
- 같은 상태의 save 파일. 현행 렌더러 비교 캡처가 이 파일을 읽으므로 native ASCII 장면과 동일한 상태다.

`source_scene.py`는 타입 검증 후 자료를 합성기 JSON으로 포장한다. `id`는 진짜 AgentId, `index`는 좌표순 행 번호다. `activity`는 asleep 우선, 다음 실제 carry>0, 다음 job=Chop이며 작업 중(state=3), 그 밖은 awake다. `awake`는 세부 활동을 알 수 없다는 화상 프레임 기본값이며, 원래 상태·목표·작업 코드를 함께 보존한다. top/hair/방향은 예술적 매핑이지 시뮬레이션 관측값이 아니다.

중요한 native 제한:

1. `ascii::engine_at`은 스크립트의 각 entry를 최초 tick에 한 번만 실행한다. 일반 colony runner의 `every_days` 반복 및 `after` 조건 대기·재시도는 실행하지 않는다. 목표가 지정한 ASCII CLI의 실제 결과를 보존했으며, 일반 soak 결과와 동일하다고 주장하지 않는다.
2. `l/L`은 모든 ground item kind를 작은/큰 스택으로 축약한다. 기존 주석·문서의 “logs”는 최신 ItemKind와 맞지 않는다. ASCII만으로 종류를 판정할 수 없다.
3. 벽·문·바닥·침구·굴착·작업장 세부·동물·의복·자원 노드·캐리 종류·방 역할은 ASCII만으로 복원되지 않는다. 이 패스의 M3–M5 추가 배치는 선언 오버레이다.
4. 실제 지형은 보충 추출로 정확히 복구했지만 사람·나무 등 뒤에 숨은 foreground 객체는 glyph_rows만으로 복구하지 않는다. 9명 이상 공유 타일도 native people JSON의 정확한 다중집합을 사용한다.

## 검증

| 검사 | 결과 | 증거 |
|---|---|---|
| native release probe 빌드 | 성공 | 외부 `probe-build.log` |
| 3장 ASCII와 추출기 grid | 3/3 byte equal | scene JSON provenance 및 `rich_*.json`의 `native_ascii_equal` |
| 3장 사람 위치·중복 수 | 3/3 exact multiset equal | `native_people_multiset_equal` |
| 손상 ASCII 1문자 음성 시험 | exit 1, 결과 파일 없음 | 외부 `scenes/negative_source_checks.json` |
| 사람 좌표 1개 손상 음성 시험 | exit 1, 결과 파일 없음 | 같은 파일 |
| Rust fmt / clippy `-D warnings` | 성공 | 외부 `extract-clippy.log`, fmt 명령 |
| Python ruff / basedpyright | 성공, 0 errors·0 warnings | 외부 `source-typecheck.log` |
| native worktree `git status --short` | 빈 출력 | game tracked source 변경 없음 |

주요 산출물: `art/compositor/scenes/scene_*.json`, `docs/v8/artenv1/scenes/source_manifest.json`. 각 파일은 약 83KB이며 1.5MB 제한 아래다. raw ASCII·raw JSON·save·빌드 로그는 외부 데이터 디렉터리에만 둔다.

## 기존 렌더러 참조

M5 contact sheet의 원본을 `/Users/rexxa/work/m5-data/shots/m5/`에서 찾았다. `m5_set.json`에는 각 스크린샷의 tick·시각·사람 화면 좌표·실제 carry·job·stage·수면 및 소스 save가 기록되어 있다. 초기 참조 `m5_01_wolves_banner.png`, `m5_05_cold_snap_banner.png`, `m5_07_hunted_deer.png`는 원본 그대로 `reference/`에 복사했다. 첫 그림은 seed0, UI 14일째 11:30, 1920×1080, 16px 타일이므로 05의 직접 점수 비교용으로는 조건이 다르다.

그래서 동일 native 상태의 save 세 개로 현행 렌더러를 실제 Godot 창에서 캡처했다. `baseline_capture.gd`는 `load_world` 후 시뮬레이션을 정지하고 camera zoom4/2로 64/32px 타일을 찍는다. 첫 실행에서 1600×900 여섯 장 생성, `ENV1_BASELINE_CAPTURE_DONE 6`, 오류 없음, day/noon 및 night 영상을 직접 확인했다. 최종 화면 중심 정렬·HUD 크롭·참조 파일·좌표 정답의 권위는 후속 담당자의 `BASELINE.md` 및 baseline manifest다. 첫 raw 실행은 `/Users/rexxa/work/artenv1-data/baseline/`와 `baseline-capture.log`에 남는다.

이 문서는 시뮬레이션 장면 출처 증거이며, 합성 가독성·성능·오너 승인 판정은 아니다.
