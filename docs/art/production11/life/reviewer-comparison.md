# 독립 생애 판독 대조

H8: **FAIL**. 원문45문장: PASS 44 / FAIL 1 / NOT_VERIFIED 0. P3-S07의 오후디는 원본 subject54의 오후티와 다르다. 추론은 판정에서 제외했다.

원본16장 판독을 그대로 보존했다. 최초 누락된7개 항목은 seed20261010, tick85080의 별도 원본 재현으로 대조했다. 재현은 Part2 UI에서 실행했지만 simulation bridge SHA가 원촬영과 같고, 인물0·25의 person/chronicle 전체가 원촬영과 동일하다. 재현 JSON에는 HEAD 자체가 찍혀 있지 않으므로 실행 후 관찰한 HEAD94cc1960은 문맥으로만 기록한다. 원촬영 HEAD32723918과 소스 SHA는 대체하지 않았다.

이름은 촬영 당시 SHA와 일치하는 PersonNames/ArtHash를 Python으로 동일 계산했다. 날짜는 `(tick+480)//1440+1`, 사망 나이는 `value//1440`. PackedByteArray 문자열은 JSON 배열로 해석했고 원본을 보존했다. 각 원문·원시 필드·배열 위치·해시는 JSON 보고서에 있다.

| 문장 | 판정 | 근거/제한 |
|---|---|---|
| P1-S01 | PASS | PASS: detail x23/y18; agent snapshot age58/stage2(Elder); society home_agent_ids index0=25 maps home_settlement_ids index0=1. |
| P1-S02 | PASS | PASS: state4 Activity, target8 ActivityPlace; matching entity8589934617 activity index0 kind6 SitByFire, place23/18; carried_foods index27=0. |
| P1-S03 | PASS | female=0; latest Paired other participant39=Bimuo/비무오; no subsequent pair/loss/death39 in world history through capture tick. |
| P1-S04 | PASS | mother0=Nitiso/니티소; alive=false. English ellipsis remains a visual finding. |
| P1-S05 | PASS | father1=Baho/바호; died at55740 before85080. |
| P1-S06 | PASS | PASS: family children exactly36,40,50; all alive=true and names Totosu/Yori/Gawa. |
| P1-S07 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S08 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S09 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S10 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S11 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S12 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S13 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S14 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S15 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S16 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S17 | PASS | Event kind, participant roles, settlement where stated and derived day agree; deterministic names verified. Village-parent qualifier is a visible localized qualification, not a biological assertion. |
| P1-S18 | PASS | Labels/bars without numbers are presentation observations; no temperament numeric value asserted. |
| P2-S01 | PASS | alive=false, founderkind1, died31800=>day23, cause3=old age. |
| P2-S02 | PASS | female0; paired0+1; Baho death precedes capture. |
| P2-S03 | PASS | mother=-1 and father=-1. |
| P2-S04 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S05 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S06 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S07 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S08 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S09 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S10 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S11 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S12 | PASS | Event roles/day/name agree; death age floor(81847/1440)=56 and cause3 where asserted. |
| P2-S13 | PASS | Labels/bars presentation only; no numerical temperament claim. |
| P3-S01 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S02 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S03 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S04 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S05 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S06 | PASS | Event participants, deterministic names, day and settlement agree; death age floor(value/1440) and cause3=OldAge agree where asserted. |
| P3-S07 | FAIL | FAIL: world event index176 subject54 maps to Ohuti/오후티, but frozen reviewer wrote 오후디. Day57 and Village1 agree. Clipped parent names are not reconstructed. |
| P4-S01 | PASS | day_index59=>Day60; time10; season0=spring; day_of_season4. Daytime at10 corroborated. |
| P4-S02 | PASS | PASS:29 live entities; four building kind1 shelters; zero construction IDs. |
| P4-S03 | PASS | PASS for raw quantities: food174,wood20,cold0,berries48(sum kind0 amounts<255),births32,deaths28(sum five death counters). Speed1.0x is solely the visible UI setting, not evidence of running at1x: HUD _display_speed defaults1.0 while capture/replay deliberately disable simulation processing. No runtime speed claim verified. |
| P4-S04 | PASS | PASS: settlement_ids[0]=1,member_counts[0]=29. |
| P4-S05 | PASS | Empty panel is a visible observation, not a claim no historical village news exists. |
| P4-S06 | PASS | Control labels/translations are presentation-only; no intervention execution or effect claimed. |
| P4-S07 | PASS | PASS: food_stocks[0]=174,wood_stocks[0]=20,wood_targets[0]=39; Korean text in English remains a screenshot-observed localization defect. |

## 판독 한계와 시각 결함

영어 어머니 상태 말줄임, 스크롤 가장자리 잘림, 한국어 이름 줄바꿈, 영어 화면의 한국어 식량·장작 표기를 유지한다. 라벨·빈 패널·버튼의 PASS는 화면 관찰에 한정한다. 속도1.0x는 표시된 UI 설정이며 실제 시뮬레이션1배속 실행을 증명하지 않는다. 캡처와 재현은 진행을 멈춘 상태다.

60–90일 실제31일 자료에 Departed가 없어 **다섯 이주 원인(코드0–4) 모두 N/A**이다. 임의 사건을 만들지 않았다. 원본 독립 판독자에게 정답 키를 전달하지 않았다.
