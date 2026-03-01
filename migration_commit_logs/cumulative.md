# Rust Migration Commit Cumulative Log

## Commit 001
- Rust 워크스페이스 컴파일/테스트 안정화.
- `sim-systems`에 A* pathfinding 모듈 추가.
- `sim-bridge`에 pathfinding 브리지 API + Godot 클래스 노출(`WorldSimBridge`) 추가.
- `sim-bridge`를 `cdylib`로 빌드 가능하게 구성하고 `.gdextension` 파일 추가.
- 데이터 로더와 enum/serde 테스트 호환성 문제 수정.
