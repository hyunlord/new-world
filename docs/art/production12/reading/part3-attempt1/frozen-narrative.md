# Frozen blind Life / Chronicle reading

All 140 native PNGs were inspected individually before the stop message arrived. Inspection coverage is complete: 140/140 images. Existing observations are frozen at the leader's request. No further image inspection followed that message. Only instructions.json and the 140 permitted PNGs were used; no source, sidecar, mapping, reference history, or truth key was opened.

## Deduplicated scene summary

- Images 0001–0008: **Ikimu / 이키무 #25, Day60**. Elder58, V1 (23,18), resting by fire, food0. Male; spouse Bimuo; deceased parents Nitiso and Baho; children Totosu, Yori, Gawa. Life: day1 bornV1 with parents by village's reckoning;5 adult;6 partnerSikino;30 lostSikino,partnerDewa,childTotosu;32 Yori;47 lostDewa,partnerBimuo,elder;49 Gawa.
- Images 0009–0015: **Nitiso / 니티소 #0**. Deceased founder; died day23 old age aged56. Male; deceased spouseBaho; parentsNone. Life:1 founderV1 and childIkimu;2 Timiga/Mabito;3 Yubama;4 partnerBaho;12 elder;18 Simowa;23 death.
- Images 0016–0042: **Day60 Chronicle**. Overlapping scroll coverage of days59–25; detailed visible event transcriptions are in each page record.
- Images 0043–0051: **Rabati / 라바티 #45, Day85**. Elder55,V2from1(3,62),restingfire,food0. Male;spouseGodosa;deceased parentsYubama/Gawa;childrenYunoni,Baga,Dawabi.30bornV1;34adult;36Godosa;39Yunoni;56Baga;60leftV1 in Rabati group with household and settledV2;62Dawabi;76elder.
- Images 0052–0061: **Bisoki / 비소키 #47, Day85**. Elder54,V2from1(6,63),sittinghearth,food0. Male;spouseGawa;deceased parentsDeyo/Gakaho;childrenGeono,Hugade,Ahumo.31bornV1;35adult;47partnerGawa;48Geono;60leftV1 in Rabati group among first to go and settledV2;61Hugade;64Ahumo;77elder.
- Images 0062–0069: **Hugade / 후가데 #69 and maps, Day85**. Adult24,V2from1(21,48),carryingfood10 toV1,on way,targetAnother villager. Female;spouseInayo;motherGawa,fatherBisoki;children0.61bornV2;65adult;68partnerInayo;85setoutV2→V1 carrying10food(aid). Maps show V1(41),V2from1(24).
- Images 0070–0096: **Day85 Chronicle**. Overlapping scroll coverage of days85–64; detailed visible transcriptions are in page records.
- Images 0097–0105: **Tiea / 티에아 #23, Day25**. Winter4,harshwinter. Adult21,V1(23,33),moving,targetWater,food6. Male;spouseKiyu;motherMiyuna,fatherKumihu;one childOhuti (glyph uncertain).4bornV1;8adult;17partnerKiyu;18child;23quarrelSuyuri ofV2 overberrybush. MapV1(42),V2from1(32),red dashed relationship label 사이가 틀어짐 partly obscured.
- Images 0106–0134: **Day25 Chronicle**. Overlapping days25–11, including25V1/V2fellout,berrybushquarrels;16V2founded fromV1 and V1split ledDehona. Detailed visible transcriptions are in page records.
- Images 0135–0140: **Three crowded scenes EN/KO**. Baho sitting at hearth; Simowa resting by fire; Wano making things. See crowd_answers.

## Crowded scenes

- image_0135.png, image_0136.png: **Baho / 바호 #1**, day 30 10:00, Elder age 46, village 1, position (23,18). Activity: **Sitting at the hearth / 화덕가에 둘러앉음**. Target Place (23,18); food 0. The selected-person panel explicitly states this activity; its target matches the displayed current position. The selection ring is around a gray-haired seated figure in the crowded tent/hearth area. Motive limit: Not explicitly stated. A social/rest motive would be inference.
- image_0137.png, image_0138.png: **Simowa / 시모와 #29**, day 60 10:00, Elder age 57, village 1, position (41,33). Activity: **Resting by the fire / 불가에서 쉬는 중**. Target Place (41,33); food 0. The selected-person panel explicitly states the activity and matching current/target position. Crowding limits independent identification of the isolated sprite. Motive limit: Not explicitly stated. Age or a need to rest cannot establish the reason from these pixels.
- image_0139.png, image_0140.png: **Wano / 와노 #30**, day 30 15:00, Adult age 9, village 1, position (24,21). Activity: **Making things / 무언가 만드는 중**. Target Place (24,21); food 0. The selected-person panel explicitly states the activity, with current and target positions equal. Motive limit: The exact item being made and motive are not visible. This does not establish construction.

## Visible defects and limits

- viewport_clipping (observed): Scroll-page boundaries clip partial lines at the top or bottom on numerous pages. Overlapping following pages often recover the text. Examples: image_0009 first Life line at bottom; image_0010 day18 entry recovered on 0011; 0014 day18 name recovered on 0015; 0020 Batiho death recovered on 0021; 0021 Buyobi partner recovered on 0022. Per-page notes retain other clipping observations. Boundary clipping alone does not establish permanent content loss.
- ordinary_word_wrapping (observed): Korean ordinary words can wrap across lines, including 0103/0104 다 / 툼 and 0050/0051 이 / 끈 이. No personal-name split was confidently established.
- raw_localization_keys (not_observed): No visible raw localization key confidently identified in this inspection.
- unexpanded_percent_placeholders (not_observed): No visible unexpanded percent placeholder confidently identified.
- joiner_artifacts (not_observed): No visible joiner glyph artifact confidently identified. Pixel inspection cannot establish absence of invisible Unicode codepoints.
- split_personal_names (not_confidently_observed): Whole-name wrapping was visible. No intra-name split confidently established; ambiguous glyph readings are recorded separately.

## Uncertainty

- Observations are visual transcriptions only, not verified against source or historical truth. Inspection coverage is complete: 140/140 images. No further image input was opened after the freeze instruction.
- Keep separate displayed scene contexts separate: Day60 initial Life/Chronicle bundle, Day85 migration/delivery bundle, Day25 winter/quarrel bundle, and the last three crowded scenes. Repeated names do not prove identical histories across those contexts.
- Font ambiguity affects EN n/h readings: several notes transcribe Onuti, but later visible KO is 오후티 and EN may be Ohuti. Preserve the uncertainty rather than silently normalize all notes.
- Image0016 partner name was initially read Muyoe; image0031 KO 누요에 and later EN Nuyoe suggest Nuyoe. Initial reading remains uncertain.
- Image0078 Dawabi partner was read Onuki/Ohuki uncertain; image0091 KO was read 오누키.
- Image0108 name was read Hahima; image0123 KO was read 하니마, suggesting Hanima. This is an explicitly uncertain cross-language interpretation.
- Image0006 initial heading reading 기품/기질 was uncertain; later pages clearly show 기질.
- Per-page notes include overlapping scroll content. Treat repetitions as overlap, not additional event counts. Partial lines are not complete factual claims.
- Crowd panel text identifies activity but does not disclose underlying motives; exact item in Making things is unknown.

## Complete inspected-page inventory and observations

Each entry below represents an image actually opened. Repeated scroll entries are overlap, not additional events. This inventory preserves original visual transcriptions and uncertainties rather than silently correcting names from assumptions.

### image_0001.png

Inspected with view_image. Ikimu #25, Elder age58 days, Village1 position(23,18), Resting by the fire; target Place(23,18); carrying0 food. Day60 10:00 spring4; pop29 shelter4 construction0 food174 wood20 cold0 berries48 births32 deaths28.

### image_0002.png

Inspected with view_image. Ikimu: male spouse Bimuo; mother N itiso (display N itiso visually reads Nitiso) deceased, father Baho deceased; children3, Totosu visible. Temperament Curiosity/Caution/Warmth/Persistence bars.

### image_0003.png

Inspected with view_image. Children Totosu,Yori,Gawa. Life Day1 born Village1 parents Nitiso and Baho (parents by the village's reckoning); Day5 came of age; Day6 partners Sikino; Day30 lost Sikino; Day30 partners Dewa.

### image_0004.png

Inspected with view_image. Day30 child Totosu; Day32 child Yori; Day47 lost Dewa; Day47 partners Bimuo; Day47 elder; Day49 child Gawa.

### image_0005.png

Inspected with view_image. KO 이키무 #25 노인 나이58일 1번마을 위치(23,18) 불가에서 쉬는 중 목표 장소(23,18) 식량0개 운반 중.

### image_0006.png

Inspected with view_image. KO 기품? heading visually 기품/기질 uncertain; 호기심 신중함 정 끈기. 가족 남 배우자 비무오 모 니티소 사망자 부 바호 사망자 자녀3명 토토수 요리 가와.

### image_0007.png

Inspected with view_image. KO 이키무 life: day1 born 1번마을 parents 니티소 바호 (마을이 정한 부모);5 adult;6 partner 시키노;30 lost 시키노 and partner 데와, child 토토수;32 child 요리.

### image_0008.png

Inspected with view_image. KO overlaps6/30/32;47 lost 데와, partner 비무오, elder;49 child 가와.

### image_0009.png

Inspected with view_image. Nitiso #0 Deceased, founder died day23 cause old age. Male spouse Baho deceased mother/father None. Life first line clipped at bottom.

### image_0010.png

Inspected with view_image. Nitiso life day1 founder Village1 and child Ikimu;2 child Timiga and Mabito;3 child Yubama;4 partner Baho;12 elder;18 child line clipped bottom.

### image_0011.png

Inspected with view_image. Overlap day2-12;18 child Simowa;23 died aged56 days old age.

### image_0012.png

Inspected with view_image. Duplicate day2 Timiga,Mabito;3 Yubama;4 partner Baho;12 elder;18 child Simowa;23 died aged56 old age.

### image_0013.png

Inspected with view_image. KO 니티소 #0 사망자 첫주민 23일째 세상을 떠남 사인 노환;남 배우자 바호 사망자 모없음 부없음.

### image_0014.png

Inspected with view_image. KO 니티소 day1 founder 1번마을 and child 이키무;2 children 티미가 마비토;3 유바마;4 partner 바호;12 elder;18 child name below viewport.

### image_0015.png

Inspected with view_image. KO overlaps1-12;18 child 시모와;23 died aged56 노환.

### image_0016.png

Inspected with view_image. Chronicle day59 Katago partners Kohua;58 Simowa partners Muyoe;58 Haritu born Village1 parents Buyobi and Totosu;58 Yubama died aged54 old age;57 Demu born Village1 parents Tahuku and Yubama.

### image_0017.png

Inspected with view_image. Chronicle57 Demu birth parents Tahuku/Yubama overlap;57 Timiga died55 old age;57 Onuti born V1 Gakaho/Wano;57 Yunoni died3 cold;53 Nuyoe born V1 Tanuka/Rabati.

### image_0018.png

Inspected with view_image. 53 Nuyoe overlap clipped top;53 Tanuka partner Rabati;53 Tubido died62 old age;53 Yunoni born V1 Bisoki/Godosa;53 Yuruge died58 old age.

### image_0019.png

Inspected with view_image. Top prior death clipped;51 Nieba born V1 Kuroku/Haa;51 Nobibu died59 old age;49 Gawa born V1 Bimuo/Ikimu;49 Bosobu died55 old age.

### image_0020.png

Inspected with view_image. 47 Bimuo partner Ikimu;47 Kohua born V1 Buyobi/Totosu;47 Dewa died57 old age;46 Katago born V1 Tahuku/Yubama;46 Batiho died55 of old [bottom clipped].

### image_0021.png

Inspected with view_image. 46 Batiho died55 old age;45 Bisoki partner Godosa;45 Kuroku partner Haa;44 Tahuku partner Yubama;41 Buyobi partner [bottom clipped].

### image_0022.png

Inspected with view_image. 41 Buyobi partner Totosu;41 Bisoki born V1 Gakaho/Wano;41 Hubo died56 old age;40 Godosa born V1 Timiga/Nobibu;40 Baho died55 of old [clipped].

### image_0023.png

Inspected with view_image. 40 Baho died55 old age;37 Rabati born V1 Gakaho/Wano;37 Kumihu died58 old age;37 Tanuka born V1 Yuruge/Bosobu;37 Huesi died61 of old [clipped].

### image_0024.png

Inspected with view_image. 37 Huesi died61 old age;36 Yuruge partner Bosobu;36 Kuroku born V1 Timiga/Nobibu;36 Tinuyu died59 old age;35 Haa born V1 Tubido/Batiho.

### image_0025.png

Inspected with view_image. 35 Haa overlap;35 Tiea died58 old age;33 Anunu born V1 Gakaho/Wano;32 Yori born V1 Dewa/Ikimu;32 Dehona died60 old age.

### image_0026.png

Inspected with view_image. 32 Dehona overlap;32 Bimuo born V1 Timiga/Nobibu;31 Sogeru born V1 Tubido/Batiho;31 Tasoe died59 old age;31 Sahi died62 old age.

### image_0027.png

Inspected with view_image. 31 Sahi death overlap;31 Kiyu born V1 Gakaho/Wano;31 Gawa died53 old age;30 Totosu born V1 Dewa/Ikimu;30 Dewa partner Ikimu.

### image_0028.png

Inspected with view_image. 30 Dewa partner overlap;30 Birige died57 old age;30 Etuba died57 old age;30 Buyobi born V1 Mabito/Gawa;30 Sikino died58 old age.

### image_0029.png

Inspected with view_image. 29 Gayo born V1 Timiga/Nobibu;29 Timiku died52 old age;28 Gakaho partner Wano;28 Deyo born V1 Tubido/Batiho.

### image_0030.png

Inspected with view_image. 29 Timiku/28 Gakaho/28 Deyo overlap;28 Miyuna died59 old age;25 Timiga partner Nobibu.

### image_0031.png

Inspected with view_image. KO59 카타고 partner 코후아;58 시모와 partner 누요에;58 하리투 born1 parents 부요비 토토수;58 유바마 died54 노환;57 데무 born1 타후쿠 유바마;57 티미가 died55 노환. EN16 partner looked Muyoe, KO 누요에; uncertain initial EN letter.

### image_0032.png

Inspected with view_image. KO57 오누티 born1 가카호 와노;57 유노니 died3 추위;53 누요에 born1 타누카 라바티;53 타누카 partner 라바티;53 투비도 died62 노환;53 유노니 birth parents below edge.

### image_0033.png

Inspected with view_image. KO53 유노니 born1 비소키 고도사;53 유루게 died58 노환;51 니에바 born1 쿠로쿠 하아;51 노비부 died59 노환;49 가와 born1 비무오 이키무.

### image_0034.png

Inspected with view_image. KO49 보소부 died55 노환;47 비무오 partner 이키무;47 코후아 born1 부요비 토토수;47 데와 died57 노환;46 카타고 born1 타후쿠 유바마.

### image_0035.png

Inspected with view_image. KO46 바티호 died55 노환;45 비소키 partner 고도사;45 쿠로쿠 partner 하아;44 타후쿠 partner 유바마;41 부요비 partner 토토수;41 비소키 born1 가카호 와노.

### image_0036.png

Inspected with view_image. KO41 후보 died56 노환;40 고도사 born1 티미가 노비부;40 바호 died55 노환;37 라바티 born1 가카호 와노;37 쿠미후 died58 노환.

### image_0037.png

Inspected with view_image. KO37 쿠미후 overlap;37 타누카 born1 유루게 보소부;37 후에시 died61 노환;36 유루게 partner 보소부;36 쿠로쿠 born1 티미가 노비부;36 티누유 died59 노환.

### image_0038.png

Inspected with view_image. KO36 티누유 overlap;35 하아 born1 투비도 바티호;35 티에아 died58 노환;33 아누누 born1 가카호 와노;32 요리 born1 데와 이키무;32 데호나 died60 노환.

### image_0039.png

Inspected with view_image. KO32 데호나 death60;32 비무오 born1 티미가 노비부;31 소거루 born1 투비도 바티호;31 타소에 died59;31 사히 died62;31 키유 born1 가카호 와노. Death causes 노환.

### image_0040.png

Inspected with view_image. KO31 키유 overlap;31 가와 died53 노환;30 토토수 born1 데와 이키무;30 데와 partner 이키무;30 비리게 and 에투바 each died57 노환.

### image_0041.png

Inspected with view_image. KO30 에투바 overlap;30 부요비 born1 마비토 가와;30 시키노 died58 노환;29 가요 born1 티미가 노비부;29 티미쿠 died52 노환;28 가카호 partner 와노.

### image_0042.png

Inspected with view_image. KO29 가요/티미쿠 and28 가카호 overlap;28 데요 born1 투비도 바티호;28 미유나 died59 노환;25 티미가 partner 노비부.

### image_0043.png

Inspected with view_image. New Day85 10:00 Spring1 pop65 villages2 shelter10 food369 wood12 cold0 berries54 births88 deaths45. Rabati #45 elder age55 V2 from1 position(3,62), resting by fire target Place(3,62), carrying0.

### image_0044.png

Inspected with view_image. Rabati male spouse Godosa mother Yubama deceased father Gawa deceased children3; Yunoni visible.

### image_0045.png

Inspected with view_image. Children Yunoni Baga Dawabi. Day30 born V1 Yubama/Gawa;34 adult;36 partner Godosa;39 child Yunoni;56 child Baga;60 departure line clipped.

### image_0046.png

Inspected with view_image. 39/56 overlap;60 Left V1 in Rabati's group; went with household.60 settled new village V2 from V1;62 child Dawabi;76 elder.

### image_0047.png

Inspected with view_image. Duplicate of46 all facts readable (39,56,60 departure/settlement,62 child Dawabi,76 elder).

### image_0048.png

Inspected with view_image. KO 라바티 #45 elder55 V2 from1 position(3,62) 불가에서 쉬는 중 target 장소(3,62), carrying0.

### image_0049.png

Inspected with view_image. KO male spouse 고도사 mother 유바마 deceased father 가와 deceased children3 유노니 바가 다와비.

### image_0050.png

Inspected with view_image. KO30 born1 유바마 가와;34 adult;36 partner 고도사;39 child 유노니;56 child 바가;60 떠남—1번마을 이끈이 라바티 가족을 따라감;60 new settlement2 from1.

### image_0051.png

Inspected with view_image. KO overlap34/36/39/56/60;62 child 다와비;76 elder.

### image_0052.png

Inspected with view_image. Bisoki #47 elder54 V2 from1 position(6,63) Sitting at the hearth target Place(6,63) carrying0.

### image_0053.png

Inspected with view_image. Bisoki male spouse Gawa mother Deyo deceased father Gakaho deceased children3 Geono.

### image_0054.png

Inspected with view_image. Children Geono Hugade Ahumo.31 born V1 Deyo/Gakaho;35 adult;47 partner Gawa;48 child Geono;60 left V1 in Rabati's group among first to go.

### image_0055.png

Inspected with view_image. 48 child Geono overlap;60 departure among first and settled V2 from V1;61 child Hugade;64 child Ahumo;77 elder.

### image_0056.png

Inspected with view_image. Duplicate55 same48/60/61/64/77 events.

### image_0057.png

Inspected with view_image. KO 비소키 #47 elder54 V2 from1 (6,63) 화덕가에 둘러앉음 target장소(6,63),carrying0.

### image_0058.png

Inspected with view_image. KO male spouse 가와 mother 데요 deceased father 가카호 deceased children3 게오노 후가데 아후모.

### image_0059.png

Inspected with view_image. KO31 born1 데요/가카호;35 adult;47 partner 가와;48 child 게오노;60 left1 leader 라바티 먼저 떠나기로 함;60 settled2 from1;61 child name clipped.

### image_0060.png

Inspected with view_image. KO47/48/60 overlap;61 child 후가데;64 child 아후모;77 elder.

### image_0061.png

Inspected with view_image. Duplicate60 47/48/60/61/64/77 same.

### image_0062.png

Inspected with view_image. Hugade #69 Carrying food x10 to Village1. Adult24 V2 from1 (21,48) State On the way Target Another villager. Selected foreground figure outlined with path line.

### image_0063.png

Inspected with view_image. Hugade carrying10 food, female spouse Inayo mother Gawa father Bisoki children0.

### image_0064.png

Inspected with view_image. Hugade61 bornV2 Gawa/Bisoki;65 adult;68 partner Inayo;85 set out V2 for V1 with food x10 (aid).

### image_0065.png

Inspected with view_image. KO 후가데 #69 짐 나르는 중—식량10→1번마을 adult24 V2 from1 (21,48) 이동중 target다른주민.

### image_0066.png

Inspected with view_image. KO carrying10 female spouse 이나요 mother 가와 father 비소키 children0.

### image_0067.png

Inspected with view_image. KO61 born2 가와 비소키;65 adult;68 partner 이나요;85 짐을 지고 떠남2→1 식량10(원조).

### image_0068.png

Inspected with view_image. KO map view, no person panel. Selected ring at lower-middle person carrying basket with overhead tan marker and line to village. HUD same85; no new Life facts.

### image_0069.png

Inspected with view_image. KO zoomed map labels1번마을(41),2번마을·1번에서(24); numeric icon counts3,1,1 at second village; no person/Life panel.

### image_0070.png

Inspected with view_image. Chronicle85 Yui bornV1 Satua/Muroba;85 Mubabu died3 cold;84 Tikota partner Muara;84 Sihua bornV1 Mimia/Yamo;84 Haa died60 old age.

### image_0071.png

Inspected with view_image. 84 Haa overlap;82 Satua partner Muroba;82 Dayamu partner Bubono;82 Siwa bornV1 Gononu/Kayu;82 Kuroku died56 old age.

### image_0072.png

Inspected with view_image. 81 Hiboge partner Geono;81 Mubabu bornV1 Kumisu/Nikoyu;81 Anunu died59 old age;80 Komi partner Dedo;80 Kumisu partner clipped.

### image_0073.png

Inspected with view_image. 80 Kumisu partner Nikoyu;79 Mimia partner Yamo;78 Gononu partner Kayu;77 Ibono partner Oyuto;76 Yamama partner clipped.

### image_0074.png

Inspected with view_image. 76 Yamama partner Ahumo;75 Muara bornV1 Nihihi/Rati;75 Tikota bornV1 Dayamu/Kuroku;74 Komi bornV1 Onuti/Nuyoe;74 Yuio bornV1 Mawado/Demu.

### image_0075.png

Inspected with view_image. 74 Yuio overlap;73 Nikoyu bornV1 Bana/Yohati;73 Bana partner Yohati;73 Yosiku bornV1 Hobaho/Suhona;72 Ibono bornV1 Getahu/Ikodo.

### image_0076.png

Inspected with view_image. 72 Ibono overlap;72 Binuyu bornV1 Nihihi/Rati;71 Satua bornV1 Dayamu/Kuroku;71 Dayamu partner Kuroku;71 Oyuto bornV1 Onuti/Nuyoe.

### image_0077.png

Inspected with view_image. 71 Oyuto overlap;70 Koyono bornV1 Mawado/Demu;70 Mimia bornV1 Hobaho/Suhona;70 Sogeru died60 old age;69 Gononu bornV1 Getahu/Ikodo.

### image_0078.png

Inspected with view_image. 69 Gononu overlap;69 Dawabi partner Onuki? visible reads Onuki/Ohuki uncertain;69 Getahu partner Ikodo;69 Totosu died60 old age;69 Muroba bornV1 Nihihi/Rati.

### image_0079.png

Inspected with view_image. 68 Kayu bornV1 Haa/Anunu;68 Hugade partner Inayo;68 Yori died57 old age;68 Gayo died60 old age.

### image_0080.png

Inspected with view_image. 68 Matunu bornV1 Onuti/Nuyoe;68 Bimuo died57 old age;67 Dedo bornV1 Mawado/Demu;67 Yuhomu bornV1 Hobaho/Suhona.

### image_0081.png

Inspected with view_image. 66 Bana bornV1 Nihihi/Rati;66 Mawado partner Demu;66 Hobaho partner Suhona;66 Nihihi partner Rati;66 Tahuku died59 old age.

### image_0082.png

Inspected with view_image. 66 Tahuku overlap;66 Kiyu died57 old age;66 Yohati bornV1 Haa/Anunu;65 Kumisu bornV1 Onuti/Nuyoe;65 Bubono bornV1 Haa/Anunu.

### image_0083.png

Inspected with view_image. 65 Bubono overlap;64 Suemi bornV2 Haritu/Suyuri;64 Sayage bornV1 Onuti/Nuyoe;64 Gakaho died58 old age;64 Tasoe died62 old age.

### image_0084.png

Inspected with view_image. 64 Suemi/Sayage/Gakaho/Tasoe overlap;64 Ahumo bornV2 Gawa/Bisoki.

### image_0085.png

Inspected with view_image. KO85 유이 born1 사투아 무로바;85 무바부 died3 추위;84 티코타 partner 무아라;84 시후아 born1 미미아 야모;84 하아 died60 노환;82 사투아 partner 무로바.

### image_0086.png

Inspected with view_image. KO82 다야무 partner 부보노;82 시와 born1 고노누 카유;82 쿠로쿠 died56 노환;81 히보게 partner 게오노;81 무바부 born1 쿠미수 니코유;81 아누누 died59 노환.

### image_0087.png

Inspected with view_image. KO81 아누누 died59 노환;80 코미 partner 데도;80 쿠미수 partner 니코유;79 미미아 partner 야모;78 고노누 partner 카유;77 이보노 partner 오유토;76 야마마 partner 아후모.

### image_0088.png

Inspected with view_image. KO76 야마마 overlap;75 무아라 born1 니히히 라티;75 티코타 born1 다야무 쿠로쿠;74 코미 born1 오누티 누요에;74 유이오 born1 마와도 데무.

### image_0089.png

Inspected with view_image. KO74 유이오 overlap;73 니코유 born1 바나 요하티;73 바나 partner 요하티;73 요시쿠 born1 호바호 수호나;72 이보노 born1 게타후 이코도;72 비누유 parents below edge.

### image_0090.png

Inspected with view_image. KO72 비누유 born1 니히히 라티;71 사투아 born1 다야무 쿠로쿠;71 다야무 partner 쿠로쿠;71 오유토 born1 오누티 누요에;70 코요노 born1 마와도 데무.

### image_0091.png

Inspected with view_image. KO70 코요노 overlap;70 미미아 born1 호바호 수호나;70 소거루 died60 노환;69 고노누 born1 게타후 이코도;69 다와비 partner 오누키;69 게타후 partner 이코도.

### image_0092.png

Inspected with view_image. KO69 게타후 overlap;69 토토수 died60 노환;69 무로바 born1 니히히 라티;68 카유 born1 하아 아누누;68 후가데 partner 이나요;68 요리 died57 노환.

### image_0093.png

Inspected with view_image. KO68 요리 overlap;68 가요 died60 노환;68 마투누 born1 오누티 누요에;68 비무오 died57 노환;67 데도 born1 마와도 데무;67 유호무 born1 호바호 수호나.

### image_0094.png

Inspected with view_image. KO67 유호무 overlap;66 바나 born1 니히히 라티;66 마와도 partner 데무;66 호바호 partner 수호나;66 니히히 partner 라티;66 타후쿠 died59 노환.

### image_0095.png

Inspected with view_image. KO66 타후쿠 overlap;66 키유 died57 노환;66 요하티 born1 하아 아누누;65 쿠미수 born1 오누티 누요에;65 부보노 born1 하아 아누누;64 수에미 born2 parents clipped.

### image_0096.png

Inspected with view_image. KO64 수에미 born2 하리투 수유리;64 사야게 born1 오누티 누요에;64 가카호 died58 노환;64 타소에 died62 노환;64 아후모 born2 가와 비소키.

### image_0097.png

Inspected with view_image. New Day25 winter4 harshwinter pop74 villages2 shelter12 food407 wood65 cold0 berries40 births59 deaths3. Tiea#23 adult21 V1 (23,33) On the way Target Water carrying6food. Selected ring and blue line toward water.

### image_0098.png

Inspected with view_image. Tiea male spouse Kiyu mother Miyuna father Kumihu children1 Onuti. Day4 bornV1 partial at bottom.

### image_0099.png

Inspected with view_image. Tiea4 bornV1 Miyuna/Kumihu;8 adult;17 partner Kiyu;18 child Ohuti (n/h pixel uncertain);23 quarrel Suyuri ofV2 over berry bush.

### image_0100.png

Inspected with view_image. Duplicate99 complete4/8/17/18/23 facts.

### image_0101.png

Inspected with view_image. KO 티에아#23 adult21 V1 (23,33) 이동중 target물 carrying6food.

### image_0102.png

Inspected with view_image. KO male spouse 키유 mother 미유나 father 쿠미후 child1 오후티; birth line4 clipped at bottom.

### image_0103.png

Inspected with view_image. KO4 born1 미유나 쿠미후;8 adult;17 partner 키유;18 child 오후티;23 berrybush quarrel 수유리(2번마을). Text 다툼 wraps at syllable, no name split observed.

### image_0104.png

Inspected with view_image. Duplicate1034/8/17/18/23 facts.

### image_0105.png

Inspected with view_image. KO zoom map V1(42),V2 from1(32); red dashed line label 사이가 틀어짐 partially obscured by sprite. No life panel.

### image_0106.png

Inspected with view_image. Chronicle25 Yotoro bornV1 Tasoe/Gawa;25 Baho died60 oldage;25 Yamama bornV2 Deyo/Ikimu;25 Rabati quarrel Geono ofV2 berrybush;25 V1 andV2 fell out.

### image_0107.png

Inspected with view_image. 25 villages fellout overlap;24 Yamo bornV2 Birige/Dehona;24 Gawa quarrel Kumihu ofV2 berrybush;24 Dehona quarrel Bisoki ofV1 berrybush;24 Dawabi bornV2 Kuroku/Yori;24 Mawado birth clipped.

### image_0108.png

Inspected with view_image. 24 Mawado bornV1 Wano/Sogeru;24 Sahi died60 oldage;23 Hahima bornV2 Kohua/Godosa;23 Kohua partner Godosa;23 Tasoe quarrel Godosa partial.

### image_0109.png

Inspected with view_image. 23 Tasoe quarrel Godosa ofV2 berrybush;23 Tiea quarrel Suyuri ofV2 berrybush;23 Hoho bornV2 Miyuna/Kumihu;22 Hugade bornV2 Deyo/Ikimu;22 Hobaho birth partial.

### image_0110.png

Inspected with view_image. 22 Hobaho bornV2 Sikino/Bosobu;21 Inayo bornV2 Birige/Dehona;21 Suhona bornV2 Kuroku/Yori;20 Dayamu bornV2 Miyuna/Kumihu;20 Wano partner Sogeru.

### image_0111.png

Inspected with view_image. 20 Wano partner Sogeru overlap;20 Baga bornV1 Tinuyu/Mabito;20 Nihihi bornV2 Deyo/Ikimu;20 Eso bornV1 Dewa/Nobibu;19 Rati bornV2 Sikino/Bosobu.

### image_0112.png

Inspected with view_image. 19 Rati overlap;19 Tasusu bornV1 Simowa/Yubama;19 Batiho died52 oldage;19 Geono bornV2 Birige/Dehona;19 Watitu bornV1 Danora/Rugawa.

### image_0113.png

Inspected with view_image. 19 Watitu overlap;18 Suyuri bornV2 Kuroku/Yori;18 Kuroku partner Yori;18 Haritu bornV1 Bimuo/Gayo;18 Demu bornV2 Miyuna/Kumihu.

### image_0114.png

Inspected with view_image. 18 Demu overlap;18 Ohuti bornV1 Kiyu/Tiea;17 Nuyoe bornV2 Deyo/Ikimu;17 Deyo partner Ikimu;17 Kiyu partner Tiea;17 Yunoni bornV1 Tasoe/Gawa.

### image_0115.png

Inspected with view_image. 17 Yunoni overlap;17 Nieba bornV2 Sikino/Bosobu;17 Gawa bornV1 Tinuyu/Mabito;16 Kohua bornV2 Birige/Dehona;16 Bimuo partner Gayo.

### image_0116.png

Inspected with view_image. 16 Bimuo overlap;16 Tasoe partner Gawa;16 Tinuyu partner Mabito;16 Katago bornV1 Dewa/Nobibu;16 V2 founded by people fromV1.

### image_0117.png

Inspected with view_image. 16 Bisoki bornV1 Simowa/Yubama;16 V1 began to split Dehona leads a group away;15 Godosa bornV1 Miyuna/Kumihu;15 Rabati bornV1 Danora/Rugawa;14 Tanuka birth partial.

### image_0118.png

Inspected with view_image. 14 Tanuka bornV1 Dewa/Nobibu;14 Kuroku bornV1 Sikino/Bosobu;13 Haa bornV1 Birige/Dehona;13 Anunu bornV1 Simowa/Yubama;12 Simowa partner clipped.

### image_0119.png

Inspected with view_image. 12 Simowa partner Yubama;12 Yori bornV1 Miyuna/Kumihu;12 Bimuo bornV1 Danora/Rugawa;11 Danora partner Rugawa;11 Sogeru birth partial.

### image_0120.png

Inspected with view_image. 12 Simowa/Yori/Bimuo11 Danora overlap;11 Sogeru bornV1 Dewa/Nobibu.

### image_0121.png

Inspected with view_image. KO25 요토로 born1 타소에 가와;25 바호 died60 노환;25 야마마 born2 데요 이키무;25 라바티 berrybush quarrel 게오노(2);25 마을1/2 사이가 틀어짐;24 야모 birth parents below.

### image_0122.png

Inspected with view_image. KO24 야모 born2 비리게 데호나;24 가와 berrybush quarrel 쿠미후(2);24 데호나 berrybush quarrel 비소키(1);24 다와비 born2 쿠로쿠 요리;24 마와도 born1 와노 소거루.

### image_0123.png

Inspected with view_image. KO24 마와도 overlap;24 사히 died60 노환;23 하니마 (EN looked Hahima, Korean clearly 하니마) born2 코후아 고도사;23 코후아 partner 고도사;23 타소에 berrybush quarrel 고도사2;23 티에아 berrybush quarrel 수유리 village below.

### image_0124.png

Inspected with view_image. KO23 티에아 quarrel 수유리2;23 호호 born2 미유나 쿠미후;22 후가데 born2 데요 이키무;22 호바호 born2 시키노 보소부;21 이나요 birth parents clipped.

### image_0125.png

Inspected with view_image. KO21 이나요 born2 비리게 데호나;21 수호나 born2 쿠로쿠 요리;20 다야무 born2 미유나 쿠미후;20 와노 partner 소거루;20 바가 born1 티누유 마비토.

### image_0126.png

Inspected with view_image. KO20 니히히 born2 데요 이키무;20 에소 born1 데와 노비부;19 라티 born2 시키노 보소부;19 타수수 born1 시모와 유바마.

### image_0127.png

Inspected with view_image. KO19 바티호 died52 노환;19 게오노 born2 비리게 데호나;19 와티투 born1 다노라 루가와;18 수유리 born2 쿠로쿠 요리;18 쿠로쿠 partner 요리.

### image_0128.png

Inspected with view_image. KO18 쿠로쿠 overlap;18 하리투 born1 비무오 가요;18 데무 born2 미유나 쿠미후;18 오후티 born1 키유 티에아;17 누요에 born2 데요 이키무.

### image_0129.png

Inspected with view_image. KO17 누요에 overlap;17 데요 partner 이키무;17 키유 partner 티에아;17 유노니 born1 타소에 가와;17 니에바 born2 시키노 보소부;17 가와 born1 티누유 마비토.

### image_0130.png

Inspected with view_image. KO17 가와 overlap;16 코후아 born2 비리게 데호나;16 비무오 partner 가요;16 타소에 partner 가와;16 티누유 partner 마비토;16 카타고 born1 데와 노비부.

### image_0131.png

Inspected with view_image. KO16 카타고 overlap;16 2번마을 세워짐1번마을에서 갈라져나옴;16 비소키 born1 시모와 유바마;16 1번마을 갈라지기 시작 이끈이 데호나;15 고도사 born1 미유나 쿠미후.

### image_0132.png

Inspected with view_image. KO15 라바티 born1 다노라 루가와;14 타누카 born1 데와 노비부;14 쿠로쿠 born1 시키노 보소부;13 하아 born1 비리게 데호나;13 아누누 parents clipped.

### image_0133.png

Inspected with view_image. KO13 아누누 born1 시모와 유바마;12 시모와 partner 유바마;12 요리 born1 미유나 쿠미후;12 비무오 born1 다노라 루가와;11 다노라 partner 루가와.

### image_0134.png

Inspected with view_image. KO12 시모와/요리/비무오11 다노라 overlap;11 소거루 born1 데와 노비부.

### image_0135.png

Inspected with view_image. Crowd1 EN Day30 10:00 spring2 pop29 food135 wood12 berries30 births11 deaths7. Baho#1 elder46 V1 (23,18) Sitting at the hearth targetPlace(23,18) carrying0.

### image_0136.png

Inspected with view_image. Crowd1 KO 바호#1 노인46 1번마을(23,18) 화덕가에 둘러앉음 target장소(23,18),food0. Same scene135.

### image_0137.png

Inspected with view_image. Crowd2 EN Day60 10:00 spring4 pop92 villages2 shelter14 food489 wood84 berries104 births98 deaths33. Simowa#29 elder57 V1(41,33) Resting by the fire targetPlace(41,33),food0.

### image_0138.png

Inspected with view_image. Crowd2 KO 시모와#29 노인57 1번마을(41,33) 불가에서 쉬는 중 target장소(41,33),food0. Same scene137.

### image_0139.png

Inspected with view_image. Crowd3 EN Day30 15:00 spring2 pop28 shelter4 food161 wood46 berries40 births11 deaths8. Wano#30 adult9 V1(24,21) Making things targetPlace(24,21),food0.

### image_0140.png

Inspected with view_image. Crowd3 KO 와노#30 어른9 1번마을(24,21) 무언가 만드는 중 target장소(24,21),food0. Same scene139.

