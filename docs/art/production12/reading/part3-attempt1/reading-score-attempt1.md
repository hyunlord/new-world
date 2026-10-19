# Frozen Life reading score — attempt 1

**Life reading: FAIL. Crowd correspondence: 3/3 PASS.**

Answers SHA-256: `54a6c5fd988bd2ebd0c1739f753929e1108199b7b7cf4646e2bfb5f7e2bb99e5`. All 140 page observations and the scene summaries were reviewed. Raw references and immutable observed text remain in `reference-audit/factual-ledger.json`. No reader contact or answer edits occurred.

Repeated names are counted as separate recorded assertions below, not new events. Later uncertainty is retained; asserted wrong transcriptions are not reclassified as inference.

## Wrong claims

- image_0016.png: `58 Simowa partners Muyoe` → Nuyoe.
- image_0017.png: `57 Onuti born V1 Gakaho/Wano` → Ohuti.
- image_0026.png: `31 Sogeru born V1 Tubido/Batiho` → Sogaru.
- image_0032.png: `KO57 오누티 born1 가카호 와노` → 오후티.
- image_0039.png: `31 소거루 born1 투비도 바티호` → 소가루.
- image_0062.png: `Hugade #69 Carrying food x10 to Village1. Adult24 V2 from1 (21,48) State On the way Target Another villager. Selected foreground figure outlined with path line.` → Porter destination is receiver village1 hearth; target_kind5 is a simulation self-target scheduling detail, not another resident..
- image_0065.png: `KO 후가데 #69 짐 나르는 중—식량10→1번마을 adult24 V2 from1 (21,48) 이동중 target다른주민.` → Porter destination is receiver village1 hearth; target_kind5 is a simulation self-target scheduling detail, not another resident..
- image_0074.png: `74 Komi bornV1 Onuti/Nuyoe` → Ohuti.
- image_0076.png: `71 Oyuto bornV1 Onuti/Nuyoe.` → Ohuti.
- image_0077.png: `70 Sogeru died60 old age` → Sogaru.
- image_0080.png: `68 Matunu bornV1 Onuti/Nuyoe` → Ohuti.
- image_0082.png: `65 Kumisu bornV1 Onuti/Nuyoe` → Ohuti.
- image_0083.png: `64 Sayage bornV1 Onuti/Nuyoe` → Ohuti.
- image_0088.png: `74 코미 born1 오누티 누요에` → 오후티.
- image_0090.png: `71 오유토 born1 오누티 누요에` → 오후티.
- image_0091.png: `70 소거루 died60 노환` → 소가루.
- image_0091.png: `69 다와비 partner 오누키` → 오후키.
- image_0093.png: `68 마투누 born1 오누티 누요에` → 오후티.
- image_0095.png: `65 쿠미수 born1 오누티 누요에` → 오후티.
- image_0096.png: `64 사야게 born1 오누티 누요에` → 오후티.
- image_0098.png: `Tiea male spouse Kiyu mother Miyuna father Kumihu children1 Onuti. Day4 bornV1 partial at bottom.` → Ohuti.
- image_0108.png: `24 Mawado bornV1 Wano/Sogeru` → Sogaru.
- image_0108.png: `23 Hahima bornV2 Kohua/Godosa` → Hanima.
- image_0110.png: `20 Wano partner Sogeru.` → Sogaru.
- image_0111.png: `20 Wano partner Sogeru overlap` → Sogaru.
- image_0119.png: `11 Sogeru birth partial.` → Sogaru.
- image_0120.png: `11 Sogeru bornV1 Dewa/Nobibu.` → Sogaru.
- image_0122.png: `24 마와도 born1 와노 소거루.` → 소가루.
- image_0125.png: `20 와노 partner 소거루` → 소가루.
- image_0134.png: `11 소거루 born1 데와 노비부.` → 소가루.

## Unverified claims

These are not automatically wrong, and are not counted as verified facts. Raw stock/resource/building tallies and daily-activity snapshots were not preserved in the supplied reference data.
- image_0001.png: `shelter4` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0001.png: `construction0` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0001.png: `food174` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0001.png: `wood20` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0001.png: `berries48` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0001.png: `Ikimu #25, Elder age58 days, Village1 position(23,18), Resting by the fire; target Place(23,18); carrying0 food. Day60 10:00 spring4; pop29 shelter4 construction0 food174 wood20 cold0 berries48 births32 deaths28.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0005.png: `KO 이키무 #25 노인 나이58일 1번마을 위치(23,18) 불가에서 쉬는 중 목표 장소(23,18) 식량0개 운반 중.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0043.png: `shelter10` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0043.png: `food369` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0043.png: `wood12` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0043.png: `berries54` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0043.png: `New Day85 10:00 Spring1 pop65 villages2 shelter10 food369 wood12 cold0 berries54 births88 deaths45. Rabati #45 elder age55 V2 from1 position(3,62), resting by fire target Place(3,62), carrying0.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0048.png: `KO 라바티 #45 elder55 V2 from1 position(3,62) 불가에서 쉬는 중 target 장소(3,62), carrying0.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0052.png: `Bisoki #47 elder54 V2 from1 position(6,63) Sitting at the hearth target Place(6,63) carrying0.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0057.png: `KO 비소키 #47 elder54 V2 from1 (6,63) 화덕가에 둘러앉음 target장소(6,63),carrying0.` — Daily activity kind and place target coordinates are not persisted in reference_worlds. Position/age/carried food are verified; resting/hearth activity and place-target description remain unverified by raw data in this audit.
- image_0097.png: `shelter12` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0097.png: `food407` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0097.png: `wood65` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0097.png: `berries40` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0135.png: `food135` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0135.png: `wood12` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0135.png: `berries30` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0137.png: `shelter14` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0137.png: `food489` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0137.png: `wood84` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0137.png: `berries104` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0139.png: `shelter4` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0139.png: `food161` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0139.png: `wood46` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.
- image_0139.png: `berries40` — Raw HUD stock/resource/building tally getter is absent from reference-data; cannot substantiate from chronicle/person records.

## Remaining results

Historical days, birth/parent/child/partner roles, departure1/2 meanings, porter food10 sender2→receiver1, quarrel23/57 and villages1/2 estrangement agree with the checked raw records apart from the exact name readings listed above. Agent ages, life stages, positions, carried food, population and event birth/death counts agree with captured arrays/chronicles.

Crowd answers Baho/hearth, Simowa/fire-rest, Wano/crafting match kinds3/6/1; nearby counts25/37/25 and body/icon coverage0/0 are from crowd-proof sidecars. Reader relied on panel labels, as explicitly recorded.

Reader reports partial-line clipping and several glyph ambiguities. Those remain visual findings; this raw-data comparison does not independently re-score pixels. No visible raw localization key, unexpanded percent, or joiner artifact was reported. No intra-name split was confidently reported.

No delivery18 or reconciliation21 exists in these snapshots, so these captures cannot validate grouped deliveries. Crowd2 contains departurecause3 without a corresponding Life reading.
