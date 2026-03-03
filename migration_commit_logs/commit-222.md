# Commit 222 - GPU placeholder 실행 경로 모듈 분리

## 커밋 요약
- `sim-bridge`의 GPU placeholder pathfinding 함수들을 `pathfinding_gpu.rs`로 분리해, 추후 실제 GPU 커널/디스패처 통합 지점을 독립 모듈로 정리.

## 상세 변경
- 신규 파일: `rust/crates/sim-bridge/src/pathfinding_gpu.rs`
  - `pathfind_grid_gpu_bytes(...)`
  - `pathfind_grid_batch_vec2_gpu_bytes(...)`
  - `pathfind_grid_batch_xy_gpu_bytes(...)`
  - 현재는 동작 안정성을 위해 CPU 경로 호출로 폴백.
- `rust/crates/sim-bridge/src/lib.rs`
  - 위 GPU placeholder 함수 정의를 제거하고 `pathfinding_gpu` 모듈 import로 대체.
  - 기존 dispatch 경로(`dispatch_pathfind_*`)는 동일 인터페이스로 모듈 함수를 호출.

## 기능 영향
- 런타임 결과/체크섬 변화 없이 구조만 개선.
- GPU 실구현 시 `pathfinding_gpu.rs` 내부 교체만으로 단계적 도입이 가능해져, 브리지 본문 리스크를 줄임.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
