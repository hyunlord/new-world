# ART-PROD2 코드 검토

독립 검토자 `audit_code`는 수면 배율/검사, 재절단, 생성 예산, 인증 표본과 채점, Godot 계측 경로를 읽고 검증했다. 이는 그림 의미 판독이나 최종 P1 승인이 아니다.

## 발견·수정

- 재절단 compact 보고서에서 존재하지 않는 `checks.failures`를 읽던 문제를 `checks.findings`로 수정했다.
- 독립 세트 감사가 정규화 transform 기록을 확인하지 않던 문제를 원본에서 재계산한 transform 전체와 비교하도록 수정했다.
- 리뷰 재생성 허용 검사가 점수 파일 문자열에 look이 들어있는지만 보던 문제를 수정했다. 이제 schema2/실패한 round0/두 리뷰어의 각24개 고유 행과2배율 요약/실제 misread_looks 포함을 요구한다. 올바르게 읽힌 룩·round1·누락 행은 거절한다.
- 일반 Ruff 검사에서 발견한 정규식 별칭, immutable 기본값 위치, import 순서를 수정했다.
- Godot scene UID 누락을 engine ResourceSaver로 보완했다.

## 검증

- 원본96개와 prompt96개의 SHA256을 동결 인벤토리 및 실제 파일과 대조했다.
- 예약·실패 호출도 룩별2회/전체40회에 포함하며 prepare/finish는 동일 filesystem lock으로 직렬화한다.
- 수면 길이·연령별 상한, 별도 배율, 원본 재구성, alpha 보존을 확인했다.
-12명 표본의 어린이/성인여/상의/머리/활동 조건과 리뷰 후 표본 고정을 확인했다.
- 성인 sex만 포함하는83% 기준은 정수 ceil로 검사한다. unknown은 오답, 기존 mapping은 그대로다.
- Godot timer는 frame_post_draw를 세고 engine-drawn delta와 대조한다. viewport capture는 측정 구간 밖이다. 실제60초 실행은 별도 결과를 따른다.

전체 art pipeline/lab116테스트 및 이후 driver10테스트 통과. 최종 원시 검증 로그는 외부 데이터 루트의 `final-tests.log`, `final-lint.log`, `lab-typecheck.log`에 보존한다. 최종 fresh check_set과 R1/P1은 보고서에서 별도 판정한다.

최종 확대 타입 검사에서 두 테스트의 union 접근20오류를 발견해 JSON shape assertions 및 전체RGBA 값 비교로 고쳤다. 최종19Python파일은0오류·148경고(exit1)이며 경고를 숨기지 않았다. 전체117테스트 및 테스트 수정 후35개 재실행 통과; 최신 결과는 VERIFICATION.json을 따른다.
