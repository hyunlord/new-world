# Commit 208 - A* open-set를 BinaryHeap 우선순위 큐로 전환

## 커밋 요약
- `sim-systems` A*에서 최소 f-score 노드 선택을 선형 스캔(`Vec`)에서 우선순위 큐(`BinaryHeap`)로 전환해 open-set 처리 비용을 추가 절감.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `OpenEntry { idx, f }` 및 `Ord/PartialOrd` 구현 추가(최소 f 우선 pop).
  - `open_set` 타입을 `Vec<usize>`에서 `BinaryHeap<OpenEntry>`로 교체.
  - neighbor 업데이트 시 decrease-key 대신 최신 항목을 heap에 push하고,
    - stale entry(`entry.f > f_score[idx]`)는 pop 시 건너뛰는 방식 적용.
  - `max_steps` 카운팅을 유효 확장 노드에만 적용하도록 정렬해 stale pop으로 인한 조기 종료를 방지.

## 기능 영향
- path 결과 checksum은 유지.
- A* open-set 최소값 탐색이 O(n) 스캔에서 heap pop 기반으로 전환되어 경로탐색 CPU 비용 추가 개선.

## 검증
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100) 유지
  - `stress checksum=24032652.00000`(@10k) 유지
  - `needs checksum=38457848.00000`(@10k) 유지
- pathfinding 성능 관측:
  - 이전(Commit 207): `ns_per_iter ≈ 2907532.5`
  - 이후: `ns_per_iter ≈ 1986499.6`
