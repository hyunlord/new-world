# Part 3 — society snapshot 검증 기록

## 구현 계약

`world_node.rs`의 추가 전용 `get_society_snapshot`은 순수 Rust `collect_society_snapshot(&World, &SimResources)` 결과를 Godot packed arrays로 변환한다. 기존 함수·시뮬레이션 상태·스케줄링·규칙을 수정하지 않는다. 시뮬레이션 Pass 12가 보존할 읽기 계약은 `Settlement::{settlement_id,parent,founded_at,member_agents}`, `Agent::id`, `Founding::{site,fission_id,leader,since_tick}`, `SimResources::{settlements,fission_log,retired_hearths,current_tick}`, `FissionRecord::{fission_id,parent,site,depart_tick,members,daughter,founded_tick,cancelled_tick}`이다.

- 마을 행의 표시 필터와 순서는 기존 `collect_settlement_snapshot`을 그대로 호출해 공유한다. 좌표가 있는 구성원이 없는 마을은 마을 행에서 빠진다.
- `member_counts`는 명부 전체 인원이다. 걷는 창립 무리도 창립 전까지 부모 명부와 인원에 포함한다. 이전 getter의 positioned-member count를 변경하지 않았다.
- 소속 행은 모든 마을 명부를 읽고 unsigned AgentId 순서로 정렬한다. 표시 필터로 숨겨진 마을의 명부도 소속에서는 보존한다. 명부에 없는 주민을 임의로 귀속하지 않는다.
- 창립 무리는 AgentId 순서, fission 이력은 fission_id 순서, 폐기된 화덕은 기존 BTreeSet의 좌표 순서다.
- PackedInt64의 AgentId는 기존 `get_agent_snapshot.agent_ids`와 동일한 bit representation을 쓴다. 큰 u64를 작은 정수로 줄이거나 entity id와 혼동하지 않는다. 없는 부모·딸·이벤트 tick은 `-1`이다.
- 24개 키: PackedInt32 15개, PackedInt64 7개, PackedByte 1개, int `current_tick` 1개.

## 자율 결정

GOAL의 좁은 Rust 예외를 우선하여 기존 대형 `world_node.rs`를 분할하거나 포맷 정리하지 않았다. 변경량은 추가 210행이다. 새 Godot 테스트는 주석·공백 제외 92행이다. getter docstring은 공개 FFI 역할을 기록하고, 정수 변환 주석은 기존 bridge의 unsigned-to-signed 호환 이유를 기록한다.

`InterventionRecord.soaked_wood`가 빠진 기존 단위 테스트 fixture는 Part 0의 알려진 컴파일 차단이며 수정 허용 범위 밖이다. 이를 수정하거나 테스트를 삭제하지 않았다. 새 단위 테스트가 파일 안에 존재하는 것과 실행에 성공한 것은 별도 사실로 기록한다.

## 테스트 설계와 선행 실패

`society-red.log`: 새 테스트를 이전 bridge에 실행하여 종료 코드 1, `Missing real get_society_snapshot FFI`를 확인했다. 실제 `WorldSimNode`를 호출하는 테스트이며 stub getter가 아니다.

Rust 단위 테스트 `society_joins_built_world_agents_and_preserves_order_filter_and_history`: 실제 `game_v17(20261013)`을 구성하고 1440tick 진행한 뒤 agent identity join을 검사한다. 이어 같은 월드에 테스트용 명부·창립·fission·retired fixture를 구성하여 누락 sentinel, 순서, 숨겨진 마을 필터, u64 identity bit preservation, 전체 명부 인원, 현재 tick을 검사한다. 이 후반 fixture는 자연 시나리오 증거가 아니다.

Godot 테스트 `test_society_snapshot_p7.gd`: 실제 FFI에서 packed type, 병렬 길이, identity 정렬, settlement getter와 필터 일치, agent_ids join, 명부 인원, 반복 조회 불변성, reset을 검사한다. `SOCIETY_LONG_TEST=1` 실행은 seed 20261013을 자연 진행하여 tick40200 출발13명, 부모 명부 유지, tick40329 창립, 딸2·부모1, Founding 제거, 로그 보존을 검사한다.

## 자체 검토

읽기 전용 bridge snapshot이라는 책임을 유지한다. 변환 전에는 정적 Rust 필드만 사용하며 동적 dictionary는 Godot 경계에서만 만든다. unsafe·추가 의존성·새 시뮬레이션 파라미터가 없다. 생산 코드에 unwrap/expect를 추가하지 않았다. `git diff --check` 통과. Rust gates와 bridge build는 리더가 직렬 실행하고, 실제 FFI 실행 결과는 별도 로그로 기록한다.

## 실제 실행 결과

| 검사 | 결과 | 로그 |
|---|---|---|
| 변경 전 real FFI 테스트 | 기대한 실패, exit1: getter 없음 | `society-red.log` |
| bridge build | exit0 | `bridge.log` |
| `cargo clippy --workspace -- -D warnings` | exit0 | `clippy.log` |
| `cargo test -p sim-bridge` | exit101: 기존 `InterventionRecord.soaked_wood` fixture 누락 | `cargo-test.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit101: 같은 기존 fixture 누락 | `clippy-all.log` |
| 실제 Godot FFI 짧은 테스트 | exit0, failures=0 | `society-short.log` |
| 실제 Godot FFI 자연 분열 테스트 | exit0, failures=0, long=1 | `society-natural.log` |

두 Godot 실행 모두 headless, window lock 불필요, SCRIPT ERROR 없음. 자연 실행은 300초 제한 안에 완료했다. 로그는 `seed=20261013 depart=40200 found=40329 size=13`을 보고했다. 수정되지 않은 실제 `game_v17`의 부모 명부 유지 → 딸2 창립 → founder 제거 → reset을 실제 FFI로 확인했다. 취소·해산·retired hearth의 자연 발생은 이 시드에서 없으므로 실제 자연 증거로 주장하지 않는다. 해당 Rust fixture 경계 검사는 작성했지만 기존 컴파일 차단 때문에 미실행이다.
