# Art Pass 7 presentation system

## Direction
Inherit project-owned D3 art and `docs/art/production/DESIGN.md`, with Pass 6 cold presentation. Native Godot pixels; no web framework or new dependencies. GOAL.md defines all new visual tokens.

## Palette
Village number palette: #E0B060, #7FB0D8, #B88AD0, #8CC07A, repeating by stable UI number. Underline, overview disc and selected-member outline share the same village colour. Exposure death (0.70,0.84,0.97); unknown death (0.55,0.55,0.55); other four reason colours inherited. Founding trail #E8D8A8 at alpha0.6; abandoned villages grey.

## Typography
Existing Galmuri small HUD font; world labels remain 9–12 screen pixels at every integer zoom. Locale en/ko only. Daughter labels explicitly name parent; count excludes founders walking out.

## Geometry
16px world tiles, native sprites and foot-based sort unchanged. Labels above hearths. Selected members only: 1px colour outline. Founding trail: 2px dash,3px gap; site pole1×6px, pennant4×3px, dark1px outline. Overview alpha0.28 unchanged.

## Components and states
Shared settlement selection and stable ID→number registry reset with the world. Labels, trails and site markers created under existing nodes. Retired hearths never borrow neighbouring fire/wood state. Four visible feed lines plus complete event history for measurement.

## Motion
One optional breath revision:5×3px birth alpha0.85,6px rise/1.2s,every1.6s. One windfall revision:2px pale dust ring/0.6s,drop held0.8s. Render-only changes, no simulation mutation.

## Verification
Native windowed Godot through shared window lock; headless behavior tests; natural/live, natural/society and intervention captures separate. Two blind observers see only frozen images and allowed answers. A1–A30 thresholds unchanged. Browser and Lighthouse checks are inapplicable to this Godot surface.

## 최종 독립 검토 반영

초기10px 이름표 설계는 역사 기록이다. 최종28f58e56은12화면px 글자,5×3px 여백, 어두운 배경판과 정적 지붕 회피를 사용한다. HUD 하단과 뷰포트 경계를 지키고 움직이는 주민 때문에 위치가 흔들리지 않는다. 한 번 수정 후 두 독립 검토자 모두 ACCEPT였으며 세 번째 시각 수정은 하지 않았다.
