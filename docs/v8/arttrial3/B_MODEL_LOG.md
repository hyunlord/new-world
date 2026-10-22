# TRIAL3B — 모델·클립 비용 사전 확인

2026-10-22 KST. **STOPPED_COST_UNAVAILABLE**. 생성 제출 **0건**, 소비 **0크레딧**, 보유 **10크레딧**. 필요 크레딧 **산출 불가 / 보유 10**. 10 이내임을 입증하지 못해 Part 1에 진입하지 않았다. 비용 미확인을 0크레딧이나 무료 호출 가능으로 해석하지 않는다.

## 적용한 사용자 변경

- `auth status` 대신 `auth token`, `workspace status`, `generate cost`로 확인한다.
- 64×64 시험 생성은 하지 않는다. 모든 예정 호출의 견적을 먼저 기록한다.
- 모델 1회 × 3 설정 + 클립 비용이 10을 넘거나 free plan에서 3D/리깅 호출 불가이면 중단한다.
- 10 이내가 검증되어 재개할 때도 설정마다 실제 소비·잔액을 기록하고 다음 설정 비용보다 잔액이 적으면 중단한다.

## 계정과 목록

CLI: `higgsfield 1.2.2 (53301eadf7311fc6a6a4c024cf4aff432028e974)`.
`higgsfield auth token --json`: exit 0, 비어 있지 않은 stdout; 토큰은 출력/저장하지 않고 REDACTED만 기록했다.
`higgsfield workspace status --json`: exit 0, `plan_type=free`, `credits=10`, `is_selected=true` (조회 전후 동일).
`model list`, `workflow list`, 아래 세 모델의 `model get`, 다섯 종류의 `preset list animation-action --query ...`는 모두 exit 0.
목록 노출과 견적 성공은 실제 생성 권한·출력 품질의 증거가 아니다.

## 예정 단가표

| 용도 | 모델/설정 | 1회 크레딧 | 견적 exit / 결과 |
|---|---|---:|---|
| A-pose 참조 이미지 | `nano_banana_2_lite`, 1k, 2:3 | **1** | 0, `{"credits":1}` |
| 인물 + 리깅 + idle | `image_to_3d`, texture/rigging/animation=true, a-pose, action 0 | **UNKNOWN** | 3, alpha v2 cost estimation 미지원 |
| 별도 idle (비교용, 합계에 중복 산입 안 함) | `3d_rigging`, animation=true, action 0 Idle | **UNKNOWN** | 3, 동일 오류 |
| walk | `3d_rigging`, action 30 Casual_Walk | **UNKNOWN** | 3, 동일 오류 |
| two-hand chop 후보 | `3d_rigging`, action 237 Charged_Axe_Chop | **UNKNOWN** | 3, 동일 오류 |
| carry walk 후보 | `3d_rigging`, action 611 Carry_Heavy_Object_Walk_inplace | **UNKNOWN** | 3, 동일 오류 |
| sleep 후보 | `3d_rigging`, action 267 Sleep_Normally | **UNKNOWN** | 3, 동일 오류 |
| Part 2 Blender 렌더/셰이더 | 로컬 처리, 생성 API 호출 계획 없음 | 해당 없음 | 미실행; 실행시간·로컬 비용 미측정 |

클립 ID/이름은 실시간 preset 목록에서 확인했다. 동작 적합성(양손 도끼, 통나무 운반, 누운 자세)은 재생 검증 전이므로 후보이다. 아직 다른 생성 모델을 선택하지 않았다. Meshy 별도 API로 우회하지 않았다.

## 합계와 중단 조건

M = 리깅+idle 포함 모델 견적, C30/C237/C611/C267 = 추가 클립 견적.
요청한 모델·클립 합계는 **3M + C30 + C237 + C611 + C267 = UNKNOWN**.
참조 이미지 1장을 공유하면 전체 **1 + 3M + ΣC**, 설정마다 이미지 1장이 필요하면 **3 × (1 + M) + ΣC**이며 둘 다 UNKNOWN이다. 실제 모델 출력 부재로 의복/머리/소품 분리 충족 여부도 미검증이다.

**필요 크레딧 UNKNOWN / 보유 10.** 예산 초과를 확인한 것은 아니며 free plan 호출 불가를 확인한 것도 아니다. 필수 견적 명령 실패로 10 이내임을 확인하지 못했으므로 원래 §1.1의 도구 실패 중단 규칙에 따라 중단한다. 접지·비율 개정, A 재검사, 모델/렌더/GIF 작업은 미실행이다.

## 실행 명령 및 실패 보존

```bash
higgsfield generate cost nano_banana_2_lite --prompt 'Stylized stone-age adult male full body A-pose, head 30 percent of height, broad shoulders, distinct hands and feet, neutral grey skin, plain loincloth, isolated reference' --resolution 1k --aspect_ratio 2:3 --json
higgsfield generate cost image_to_3d --image 00000000-0000-0000-0000-000000000000 --should_texture true --enable_rigging true --enable_animation true --pose_mode a-pose --animation_action_id 0 --json
higgsfield generate cost 3d_rigging --model_url https://example.com/trial3b-cost-only.glb --enable_animation true --animation_action_id 30 --json
```

마지막 명령은 action 0/30/237/611/267 각각 조회했다. nil UUID와 example.com URL은 **견적 전용 placeholder**이며 실제 업로드/생성 자산이 아니다. 이 응답은 유효 자산에 대한 생성 가능성 검증이 아니다. 로컬 파일 자동 업로드·생성·과금 요청은 하지 않았다.

정확한 최종 stderr (각 exit 3, stdout 비어 있음):

```text
Error: job_type 'image_to_3d' does not support alpha v2 cost estimation
Error: job_type '3d_rigging' does not support alpha v2 cost estimation
```

초기 견적은 필수 `image_references` / `model_url` 누락으로 exit 4였다. 이미지 URL 및 빈 배열 입력도 UUID/파일 경로 검증에서 exit 4였다. 이 시도 역시 보존했으며 실제 생성 시도로 세지 않는다. 입력 형식 보완 후에도 위의 미지원 오류가 남았다.

Raw 증거: `/Users/rexxa/work/arttrial3-data/b/preflight-resume-20261022/`의 모든 JSON. 각 파일은 argv, exit, stdout, stderr를 담는다(토큰만 제거). `cost-reference-image.json`, `cost-model-nil-reference.json`, `cost-complete-params-clip-*.json`, `workspace-after.json`이 최종 판단 근거다. 생성 설정 0/3, 렌더 설정 0/3. 실제 소비 로그에 기록할 생성 호출은 없다.

## 문서 검증

`git diff --check` 통과. `bash tools/v8/check_docs.sh` exit 1: 기존 `art/proto/CharacterLab.gd`, `wash_tint.gdshader`의 tracked `.uid` 누락 및 `scenes/camp.tscn`, `scenes/character_lab.tscn` header uid 누락. 해당 파일은 이번에 수정하지 않았다. 코드/런타임 gate는 미실행. 허용 경로가 `art/**`, `docs/v8/arttrial3/**`이므로 공통 문서나 스킬 파일은 변경하지 않았다.

## 재개 확인 2 — 2026-10-22 08:05 UTC

직전 실행은 단가 조회·문서 게시로 진전이 있었다. 이번에는 동일한 리깅 견적을 재조회했으며 exit 3과 미지원 stderr가 재현되었다. workspace는 계속 free / 10이다. Raw: 위 증거 디렉터리의 `recheck-2/`. 생성 제출은 여전히 0건이다.

공식 CLI 저장소의 현재 공개 트리에는 구현 소스가 없어 해당 오류의 정확한 발생 위치를 입증할 수 없었다. [공식 모델 명세](https://github.com/higgsfield-ai/cli/blob/fd21e7cb550b5835cc00c6b4abfea0c02fe77ff2/MODELS.md#L823-L864)는 두 모델 및 애니메이션 인자를 문서화하지만, 이것은 가격이나 free plan 권한 증거가 아니다. [공식 README](https://github.com/higgsfield-ai/cli/blob/fd21e7cb550b5835cc00c6b4abfea0c02fe77ff2/README.md) 조사에서도 문서화된 3D 견적 우회 경로는 찾지 못했다. CLI 도움말은 여전히 `generate cost`만 제공한다. 서버 오류인지 클라이언트 제한인지 확정하지 않으며, 가격 미확인을 해소하려고 생성 호출을 시험하지 않는다.

동일 비용 차단의 두 번째 연속 goal turn이다. 현재 목표는 active이며 완료가 아니다. 필수 견적 지원 또는 검증 가능한 비용 정보가 생기기 전에는 예산 조건을 통과시킬 근거가 없다.
