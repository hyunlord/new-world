# Parts 2–3 implementation preparation

Source-only audit; no Godot/cargo, no screenshot measurements, no product changes. Read Godot skill and frontend router; the explicit WorldSim pixel-art brief is the design contract. Browser/React prescriptions do not describe this Godot surface. All runtime assertions below remain NOT_VERIFIED until exercised on the frozen appropriate build.

## Activity path and exact existing state

- `scripts/ui/activity_icons_p11.gd` owns shared cached `texture(kind)` and `icon_id(kind)`; six generated textures are16×16. Craft is `assets/art/production/U1/04.png` via icon id `U1_04`. Its actual PNG dimensions are18×18 (Pillow header inspection), so measure Craft separately. `MASKS` include `o` highlights and `HIGHLIGHT=FFF2D4`, so the brief's historical one-colour description is not literally current source; no badge exists yet.
- `scripts/ui/agent_activity.gd:184` `show_activity_icon` requires zoom>=2, fallback kind idle_icon/wood_icon/activity_icon and nonempty icon id. `apply_activity_icon` uses generated texture only for p11_activity_*; the Craft U1_04 takes `ArtSpriteLibrary.apply`. Generated icons have nearest filtering, scale1, offset(-8,-16), metadata frame_size16×16 and anchor(8,16); position is `(0,sprite.offset.y-2)`.
- `scripts/ui/art_sprite_library.gd:65` resets sprite scale1. `scripts/ui/camera_controller.gd:4` has discrete1,2,3,4,6,8 zoom; default4. Consequently the source-predicted generated canvas width is64px at Z4, not16px. Do not label this a measured screen size: capture transformed opaque/pictogram/badge bounds and each background median first.
- `scripts/ui/village_activity_summary_p11.gd` shares `Icons.texture`, `ICON_SCALE=2`, HEIGHT34, count font Galmuri9 size20. Display at zoom<2 only; ranks top3 arrived kinds, identity-joins entity ids to agent ids, then village membership. A20×20 texture becomes40px high at summary2×, exceeding existing34px row height: adjust geometry alongside badge and report at Z1. Preserve shared art rather than independent summary drawings.

Execution boundary: keep seven meanings and activity_icon path. Route Craft through shared badge path too; preserve U1_04 pictogram. Use2 source-pixel margin, dark211C25 alpha0.85 rounded plate, main+highlight pictogram, integer scaling. Measure all21 pictogram-mask pair XOR/union (exclude common badge so badge cannot obscure distinctness), all21 hue distances>=40°, capture fill/background luminance contrast for grass/dirt/shelter and real Z4 dimensions>=20. Freeze Stage A input/order/results before any redesign; at most one failing-icon redesign/retest with a new reader; Stage B fresh reader receives legend first,14 crops; no-legend reader separate. Preserve failed evidence.

## Tooltip insertion points

`AgentRenderer.drawn_entity_at(world_position)` and `drawn_entities_at` at769/773 already pick visible drawn sprite opaque bounding rectangles, honoring flip and local transform; sorted frontmost by y then id. This is bounded-rectangle picking, not per-pixel alpha picking. Use existing drawn geometry, not simulation tile center. `WorldRenderer._try_agent_click` demonstrates consumer routing to inspector; tooltip must not call the click handler or mutate selection.

Renderer sprite metadata includes activity_kind, activity_place_known, entity_bits. `AgentActivity.activity_rows` provides tick-cached activity kind/place/until_tick. Resolve entity bits through existing snapshot agent_ids before `PersonNames.display_name`/present. Keep current seven activity names; no activity means hidden. Require same hovered entity continuously0.4s, reset on mouse-out/entity change/reset/hidden UI; suppress under blocking UI, hide immediately on no activity. Do not reuse rich-text link-hover ownership as tooltip state: `set_hovered_person_p12`/clear support separate person link outlines.

Source search found no UI_TOOLTIP_PERSON_ACTIVITY_FMT. Existing UI_ACTIVITY_* keys occupy exactly five files: `localization/fluent/en/messages.ftl`, `localization/fluent/ko/messages.ftl`, `localization/compiled/en.json`, `localization/compiled/ko.json`, `localization/key_registry.json`. This is two languages plus registry, not five languages. Add only the requested key using existing registry conventions, en/ko value `%s — %s`; interpolate person and localized activity once. Existing Korean TendFire is 불을 지키는 중; reuse existing values rather than replace with brief suggestions. Real-input suite must cover <0.4s hidden, >=0.4s visible, matching localized name/activity, mouse-out hidden, no-activity hidden and no inspector selection change.

## Font grid and complete syllable domain

Font-file inspection with fontTools, no renderer: `Galmuri11.ttf` and `Galmuri11-Bold.ttf` both head.unitsPerEm=1200; gcd of every outline coordinate=100. Thus the vector outlines encode a12-pixel em grid; the family label11 is not a command to render at22px. hhea ascent1200, descent-200. Current24px is2×grid and next integer multiple is36px. Store this extraction and font SHA with glyph results. The fonts have vector glyf data, no EBLC bitmap strike; derive grid from outline coordinates rather than invent a strike size.

`assets/ui/worldsim_theme.tres:21` default24, regular Galmuri11; Life RichTextLabel at inspector164 inherits theme, while `PersonNames.prepare_rich_text` overrides bold font to Galmuri11-Bold without overriding size. Source expectation regular/bold24; read actual `normal_font_size` AND `bold_font_size` and font resource paths from live Life control, since theme inheritance is runtime evidence. Existing probe_life_font_p13 tests only 티/디 and is insufficient for7.1.

PersonNames has46 EN and46 KO tokens. Blocklist suffix increments/carry apply modulo46 to existing indices; no additional token can be introduced. Capitalization changes the first Latin token. Therefore test92 distinct Latin spellings (46 lower+46 initial-capital forms), and46 Hangul spellings, all same-script unordered pairs for each regular/bold face. This includes uppercase/lowercase cross-comparisons, a conservative superset. U+2060 word joiners are formatting, not syllables; preserve plain-vs-joined visual identity check separately. Exhaustive token domain proves closure even without sampling IDs.

Latin lowercase:

`a i o e ka ki ku ko ta ti tu to na ni nu no ma mi mu mo ra ri ru ro sa si su so ha hi hu ho ya yu yo wa ba bi bo bu da do ga go de ge`

Latin initial-capital:

`A I O E Ka Ki Ku Ko Ta Ti Tu To Na Ni Nu No Ma Mi Mu Mo Ra Ri Ru Ro Sa Si Su So Ha Hi Hu Ho Ya Yu Yo Wa Ba Bi Bo Bu Da Do Ga Go De Ge`

Hangul:

`아 이 오 에 카 키 쿠 코 타 티 투 토 나 니 누 노 마 미 무 모 라 리 루 로 사 시 수 소 하 히 후 호 야 유 요 와 바 비 보 부 다 도 가 고 데 게`

Render each on a common transparent canvas with identical baseline/origin and sufficient extent; threshold alpha0.5 and count binary XOR pixels (not RGB antialias difference). Report10 closest pairs per script/weight and every failing pair, threshold>=12 unchanged. If fail at24, increase name text to36 and rerender, with layout/real-input verification; do not apply an arbitrary noninteger size. Check all4186 Latin and1035 Hangul pairs per weight. No actual glyph pass is claimed here.

## game_v26 fact-capture candidates

Use p13 capture_life and activity harnesses with fresh reset and exact advance; do not accept copied historical multi19400/121080 or estranged19414/34680 as v26 selection without re-deriving. Live activity primary: seed20261010 day30 10:00 tick41880 and15:00 tick42180; seed20261018 day60 10:00 tick85080. Supplement only by prior protocol and count coverage honestly.

Wmulti19603: person62 departed90720 sender3→receiver2 aidfood10 and delivered90840; capture Life at90840 (display64 10:00), examine porter state/actual inventory, do not assume outbound. Chronicle row remains readable after delivery; choose a truly visible porter by live census for glyph/pose. For reconciliation, same seed has `[122400,21,null,null,null,1,2,0,-143]` (display86 08:00); candidate122520 is same day's10:00. Live census must confirm passage, living selected subject and visible kind21 panel entry. Raw source is `/Users/rexxa/work/p21-data/p2/A/chronicle_game_v26/chronicle_19603.json` at evaluation revision8f681805, so current bridge reproduction remains required.

Reader receives captures only and no data answers. Freeze transcript; score visible non-name days/causes/kinds/village roles/counts against snapshot/chronicle references, all must be correct. Score names separately with confusion pairs and count; unshown claims are unverified, not name or fact failures. Include one porter Life, reconciliation if present, and existing p12 Life/chronicle cases migrated to actual v26. Before/after screenshot pairs are necessary for each optional Life-text fix. Never treat source facts, font metadata or copied fixtures as H8/H9/H10 runtime passes.
