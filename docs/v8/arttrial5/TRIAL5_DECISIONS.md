# ART-TRIAL5 Part 0 결정 기록

- 범위: Part 0만 실행. 목표 문서의 전용 브랜치 `v8/art-trial5`와 `art/**`, `docs/v8/arttrial5/**` 제한을 적용했다. 일반 패스의 전역 CHANGELOG/ROADMAP 변경과 게임 실행은 이 후보 비교 작업에 추가하지 않았다.
- 목표 원문은 `bdf15b8b`에 고정했다. Part 1–4, 게임 설치, 병합은 실행하지 않는다.
- 원본 참조: person + hut에 STYLE.md의 공통 조명 참조 scene도 첨부했다. 이 작업의 투명 배경 지시가 기존 마젠타 배경 지시보다 우선한다.
- I 후보마다 3회, 총 12회 생성 후 종료. 1차는 비율, 2차는 단순화, 3차는 배경 및 I-2 망토/묶은 머리를 강조했다. 실패도 삭제하지 않는다.
- 비교 선택: I-1=2차, I-2=3차, I-3=2차, I-4=2차. I-1은 숫자와 비율, I-3/I-4는 상대적으로 나은 숫자와 비율을 우선했다. I-2는 숫자가 더 나쁜 3차지만 망토와 긴 묶은 머리가 실제로 보이는 유일한 시도를 선택했다.
- 원본 1024×1536과 모든 거절 시도는 외부 데이터 폴더에 보존했다. 저장소에는 머리부터 발끝까지 직사각형 크롭 후 균일 Lanczos 축소한 작업본을 둔다. 정확한 크롭/크기/해시는 후보 sidecar에 기록했다. 양자화, 픽셀 수술, 톤 조정은 없다.
- B는 무료 Standard에 실제 포함된 Superhero Male + Hair_Buzzed. 의복 메시가 없으므로 가죽 옷을 새로 만들거나 Part 1 의복을 이식하지 않는다. 맨몸 베이스라는 한계를 공개한다.
- 수치 통과와 오너 선택은 별개다. 모델 점수는 MISSING, 승인 상태는 미정이다. 기준을 완화하거나 실패를 통과로 바꾸지 않는다.
- 오너 선택: **미정**. 질문: **I-n / B / none**. 답변 전에는 다음 Part를 실행하지 않는다.

## 후속 승인 — 2026-10-23

- 오너 원문: **오너 선택: I-1. Part 1부터 진행**. 위 Part 0 중지 기록은 역사 기록이며 이 승인으로 후속 Route I 제작을 시작한다.
- 승인 기준: I-1 2차 원본 SHA-256 `833ac8180b9e0c4a1cd960e6fcf80b4eff5a98114913d97a34d4d6a56f8a6354`; 참조 작업본 `art/character/candidates/I-1.png` SHA-256 `e49180d144b97732d1e8e8ebfcc5a8ac7cff7c8f7f102afaaeec8af9a76974cc`.
- 36룩 × S/E = 72시트, 시트당 최대 2회. 모든 시트에 선택 후보를 첨부하고 E에는 해당 S 원본을 추가한다. 성별/나이/옷/머리만 지정된 범위에서 바꾸며 몸을 부품으로 나누지 않는다.
- 어린이와 여성의 loincloth 룩은 안전한 비성적 석기시대 복장으로, 불투명 허리천 및 필요한 가슴 가리개를 사용한다. 어린이에게 성인 근육·성적 특징을 묘사하지 않는다.
- 진행: (1) 계약·시도 장부 고정 → (2) S 생성 및 검사 → (3) 해당 S 참조로 E 생성 및 검사 → (4) 실패 시 남은 1회만 재생성 → (5) 랩/GIF/실재생 증거와 독립 리뷰 → (6) 허용 범위 내 한 수정 라운드 → (7) 숫자·제한 보고 및 푸시.
- 수치/구조/블라인드 가독성/성능 판정은 분리한다. 예산 소진·실패는 재시도 상한을 늘리거나 기준을 약화하는 근거가 아니다. 실패한 시트도 외부 진단 자료로 보존한다.
# Production processing corrections (2026-10-23)

- The owner explicitly chose I-1 and authorized Part1 onward. Route I remains review-only until the original S1/R1/P1 bars pass.
- Panel v1/v2 incorrectly constrained connected-figure extraction to disjoint horizontal rectangles. A read-only audit found26/27 initial sheets had five major components but overlapping rectangles. V3 uses complete-component ownership; every source alpha>0 RGBA pixel is assigned exactly once, with unchanged relative coordinates and exact reconstruction checked before resizing. Deterministic nearest-major Euclidean ownership resolves translucent pixels; equal distances choose the leftmost component, opaque dust fragments remain intact. This implements the goal's connected-component slicing, not painted repairs. Original v1/v2 judgments remain outside the repository.
- Normalization v3 rejected the size of the resized buffer even where Lanczos had made its outer padding completely transparent. V4 trims alpha==0 padding only AFTER the prescribed single Lanczos resize, records the trim and translation, and retains every alpha>0 pixel. Genuine visible overflow still fails. V3 judgments remain historical rather than final structural evidence.
- Frozen full-sheet judge-v2 acceptance is unchanged. Native extracted-person judge results are explicitly supplementary and cannot replace a failed full-sheet result. No thresholds were adjusted to improve acceptance.
- If strict canvases cannot represent the frozen review sample, a separately labeled extent diagnostic may show every source pixel at the same prescribed scale. It cannot establish delivered-set S1 or formal R1 acceptance. No unavailable person/state/facing is silently substituted.
