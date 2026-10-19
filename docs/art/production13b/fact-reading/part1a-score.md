# Part 1a fact and name reading score: images 0001–0065

No incorrect visible non-name assertion found in images 0001–0065; visual-only fields are explicitly separated from independent data verification. This is not the full 119-image or later-batch verdict.

Frozen answers SHA-256: `e6e2f47ceba07220fb6b11375dd6a221b7ba090dd65d21bbb79b7b6ea5171311`. All65 PNG/private-map/reference-image hashes matched. The answers remain unchanged.

## Exact counts

- Non-name assertions: 802; independent-data correct665; visible-only correct127; incorrect0; explicit abstentions8; motion inferences2.
- Scored visible assertions:792; excluding10 abstention/inference statements. This denominator includes event bundles, scalar fields, named relationship clauses, and artifact observations. It is not a count of unique simulation events.
- Names:424 written occurrences;411 full correct;2 correct partial/uncertain;11 incorrect. Partial uncertain readings are image0010 Simowa and image0017 Nuyoe.
- Scope is65images only. Do not use this subset as the final full fact-reading verdict.

## Limitations and scoring rules

- Display day=floor((tick+clock_offset_ticks)/1440)+1; death age=floor(value/1440).
- Event assertions bundle day/kind/village/age/cause into one row; separate events remain separate rows.
- All original per-image answer text is retained verbatim.
- CORRECT requires visual visibility plus independent data where exposed; offscreen portions are explicitly withheld.
- Current-state scalar facts each count once. Explicit repeated HUD count sets expand to their scalar components; named day is retained.
- Each semicolon-separated named relationship clause is a separate assertion, including compound parent pairs or children lists; explicit comma-separated spouse/mother/father clauses split. These are redundantly tested rather than being exempted as names.
- Every written proper-name token counts as one occurrence, including repeated names, partial/uncertain readings, and surname-free two/three-syllable names. Correct partial uncertain occurrences are separate from full correct names.
- Artifact observations count once per image including the answer preamble. Reader abstentions and motion inference do not enter the scored denominator.
- CORRECT_VISUAL_ONLY preserves visibly correct assertions without claiming independent simulation-data corroboration; notably construction0 and berry counts lack fields in reference_worlds.
- Name reading uses exact spelling; expected/read confusion pairs are aggregated separately and do not change non-name event outcomes.

The127 visible-only assertions include UI/visual observations and construction/berry HUD values. These values are legible and agree with the reader, but independent reference_worlds contains no construction-progress or terrain berry inventory. They are not silently promoted to independently verified data facts.

## Name confusion pairs

| Expected | Read | Occurrences |
|---|---|---:|
| Demu | Oemu | 2 |
| Tanuka | Taruka | 3 |
| Dehona | Oehona | 2 |
| Sogaru | Sogeru | 1 |
| 소가루 | 소거루 | 1 |
| Inayo | Irayo | 2 |

## Image evidence and original reader statements

### image_0001.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_en_page_000.png`; SHA-256 `45f0a5520b2afb32840189faa8b53ee8e0119232106741002580b895a73687aa`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/0` and `/reference_worlds/0`.

Facts: Day 60 10:00 daytime, spring day 4; population 29, shelter 4, construction 0, food 174, firewood 20, cold 0, berries 48, births 32, deaths 28. Selected #25 is elder, age 58 days, Village 1, position (23,18), resting by the fire, target same place, carrying 0 food. Picture: seated gray-haired figure near village fire area; exact selected sprite obscured by cluster, resting supported by panel.
Names: Ikimu. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0257 | state | Selected #25 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0258 | state | Elder | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0259 | state | Age 58 days | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0260 | state | Village 1 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0261 | state | Position (23,18) | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0262 | state | Resting by fire | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0263 | state | Target place (23,18) | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0264 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0388 | hud | HUD day = 60 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0389 | hud | HUD time = 10:00 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0390 | hud | HUD daytime = True | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0391 | hud | HUD season = spring | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0392 | hud | HUD season_day = 4 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0393 | hud | HUD population = 29 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0394 | hud | HUD shelter = 4 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0395 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0396 | hud | HUD food = 174 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0397 | hud | HUD wood = 20 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0398 | hud | HUD cold = 0 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0399 | hud | HUD berries = 48 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0400 | hud | HUD births = 32 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0401 | hud | HUD deaths = 28 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0483 | visual | Seated gray-haired figure near village fire area | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0527 | abstention | Exact selected sprite obscured/uncertain | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0537 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0001 | Ikimu | Ikimu | CORRECT |

### image_0002.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_en_page_001.png`; SHA-256 `b421057ad712c37df7e329058bce855fa649684f25e857694f8e2bbdee967b8e`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/1` and `/reference_worlds/0`.

Facts: Same visible day-60 HUD counts. Family male, spouse present, mother and father deceased, children 3. Temperament bars curiosity/caution/warmth/persistence have no numeric labels.
Names: Bimuo spouse; Nitiso mother; Baho father; Totosu visible child. Artifacts: none observed; later children offscreen.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0265 | state | Male | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0266 | state | Spouse present | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0267 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0268 | state | Father deceased | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0269 | state | Children 3 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0402 | hud | HUD population = 29 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0403 | hud | HUD shelter = 4 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0404 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0405 | hud | HUD food = 174 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0406 | hud | HUD wood = 20 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0407 | hud | HUD cold = 0 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0408 | hud | HUD berries = 48 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0409 | hud | HUD births = 32 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0410 | hud | HUD deaths = 28 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0484 | visual | Four temperament bars labelled curiosity/caution/warmth/persistence with no numeric labels | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0485 | visual | Later children offscreen | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0538 | artifact_observation | none observed; later children offscreen. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0602 | named_relationship | Bimuo spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0603 | named_relationship | Nitiso mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0604 | named_relationship | Baho father | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0605 | named_relationship | Totosu visible child. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0798 | hud | HUD day 60 | CORRECT | live/reference-data.json#/reference_worlds/0/clock |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0002 | Bimuo | Bimuo | CORRECT |
| P1A-N0003 | Nitiso | Nitiso | CORRECT |
| P1A-N0004 | Baho | Baho | CORRECT |
| P1A-N0005 | Totosu | Totosu | CORRECT |

### image_0003.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_en_page_002.png`; SHA-256 `73c55244c6cbc56cc457266acf901c7ac478be05794109905a531b341b28b0a2`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/2` and `/reference_worlds/0`.

Facts: Day 1 born in Village 1, parents by village reckoning; day 5 came of age; day 6 partnered; day 30 lost partner and partnered again.
Names: Totosu, Yori, Gawa at top; parents Nitiso and Baho; partner Sikino; new partner Dewa. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0001 | event | Day 1: birth (person #25); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/26 |
| P1A-C0002 | event | Day 5: came of age (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/36 |
| P1A-C0003 | event | Day 6: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/39 |
| P1A-C0004 | event | Day 30: lost partner (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/89 |
| P1A-C0005 | event | Day 30: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0486 | visual | Parents by village reckoning qualifier visible | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0539 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0606 | named_relationship | parents Nitiso and Baho | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0607 | named_relationship | partner Sikino | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0608 | named_relationship | new partner Dewa. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0006 | Totosu | Totosu | CORRECT |
| P1A-N0007 | Yori | Yori | CORRECT |
| P1A-N0008 | Gawa | Gawa | CORRECT |
| P1A-N0009 | Nitiso | Nitiso | CORRECT |
| P1A-N0010 | Baho | Baho | CORRECT |
| P1A-N0011 | Sikino | Sikino | CORRECT |
| P1A-N0012 | Dewa | Dewa | CORRECT |

### image_0004.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_en_page_003.png`; SHA-256 `99deff5a1264bb869330e974d88a285ae8822cb9a67640881a09e779e437e1a2`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/3` and `/reference_worlds/0`.

Facts: Day 30 partnered and child born; day 32 child born; day 47 partner lost, new partnership, became elder; day 49 child born.
Names: Dewa day-30 partner/day-47 lost; Totosu day-30 child; Yori day-32 child; Bimuo day-47 partner; Gawa day-49 child. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0006 | event | Day 30: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0007 | event | Day 30: birth (person #36) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/97 |
| P1A-C0008 | event | Day 32: birth (person #40) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/109 |
| P1A-C0009 | event | Day 47: lost partner (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/153 |
| P1A-C0010 | event | Day 47: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/155 |
| P1A-C0011 | event | Day 47: became elder (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/156 |
| P1A-C0012 | event | Day 49: birth (person #50) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/161 |
| P1A-C0540 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0609 | named_relationship | Dewa day-30 partner/day-47 lost | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0610 | named_relationship | Totosu day-30 child | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0611 | named_relationship | Yori day-32 child | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0612 | named_relationship | Bimuo day-47 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0613 | named_relationship | Gawa day-49 child. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0013 | Dewa | Dewa | CORRECT |
| P1A-N0014 | Totosu | Totosu | CORRECT |
| P1A-N0015 | Yori | Yori | CORRECT |
| P1A-N0016 | Bimuo | Bimuo | CORRECT |
| P1A-N0017 | Gawa | Gawa | CORRECT |

### image_0005.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_ko_page_000.png`; SHA-256 `9dd9c27e0f9bf2efd68d6eabb8e001629a887563e36ed695a69e0b21ca92ae45`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/4` and `/reference_worlds/0`.

Facts: Korean HUD day 60 10:00, spring day 4, population 29, shelter 4, construction 0, food 174, wood 20, cold 0, fruit 48, births 32, deaths 28. #25 elder age 58 days, Village 1, (23,18), resting by fire, target (23,18), carrying 0 food. Picture gray-haired seated figures near huts/fire; selected exact individual uncertain.
Names: 이키무. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0270 | state | Selected #25 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0271 | state | Elder | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0272 | state | Age 58 days | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0273 | state | Village 1 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0274 | state | Position (23,18) | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0275 | state | Resting by fire | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0276 | state | Target place (23,18) | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0277 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0411 | hud | HUD day = 60 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0412 | hud | HUD time = 10:00 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0413 | hud | HUD season = spring | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0414 | hud | HUD season_day = 4 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0415 | hud | HUD population = 29 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0416 | hud | HUD shelter = 4 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0417 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0418 | hud | HUD food = 174 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0419 | hud | HUD wood = 20 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0420 | hud | HUD cold = 0 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0421 | hud | HUD berries = 48 | CORRECT_VISUAL_ONLY | live/reference-data.json#/reference_worlds/0 |
| P1A-C0422 | hud | HUD births = 32 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0423 | hud | HUD deaths = 28 | CORRECT | live/reference-data.json#/reference_worlds/0 |
| P1A-C0487 | visual | Gray-haired seated figures near huts/fire | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0528 | abstention | Selected exact individual uncertain | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0541 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0799 | visual | Korean HUD | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0018 | 이키무 | 이키무 | CORRECT |

### image_0006.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_ko_page_001.png`; SHA-256 `271710c29a29a31a98481d774b3ab6e4b3d4b7373f86f38d4564ed2807a235e4`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/5` and `/reference_worlds/0`.

Facts: Male, spouse, deceased mother/father, three children. Temperament curiosity/caution/warmth/persistence.
Names: 비무오 spouse, 니티소 mother, 바호 father; 토토수, 요리, 가와 children. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0278 | state | Male | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0279 | state | Spouse present | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0280 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0281 | state | Father deceased | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0282 | state | Children 3 | CORRECT | /reference_worlds/0/persons/25 |
| P1A-C0488 | visual | Four temperament labels curiosity/caution/warmth/persistence | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0542 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0614 | named_relationship | 비무오 spouse, 니티소 mother, 바호 father | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0615 | named_relationship | 토토수, 요리, 가와 children. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0019 | 비무오 | 비무오 | CORRECT |
| P1A-N0020 | 니티소 | 니티소 | CORRECT |
| P1A-N0021 | 바호 | 바호 | CORRECT |
| P1A-N0022 | 토토수 | 토토수 | CORRECT |
| P1A-N0023 | 요리 | 요리 | CORRECT |
| P1A-N0024 | 가와 | 가와 | CORRECT |

### image_0007.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_ko_page_002.png`; SHA-256 `083e4dd8d127bad51db837c47e97b1e74d4e7bdc4f330334d8ab046dde62038a`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/6` and `/reference_worlds/0`.

Facts: Day 1 born Village 1 with village-designated parents; day 5 adult; day 6 partnership; day 30 partner lost and new partnership and child born; day 32 child born.
Names: 가와 at top; 니티소, 바호 parents; 시키노 day-6/day-30 partner; 데와 day-30 partner; 토토수 day-30 child; 요리 day-32 child. Artifacts: names move intact onto next lines; no internal split observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0013 | event | Day 1: birth (person #25); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/26 |
| P1A-C0014 | event | Day 5: came of age (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/36 |
| P1A-C0015 | event | Day 6: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/39 |
| P1A-C0016 | event | Day 30: lost partner (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/89 |
| P1A-C0017 | event | Day 30: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0018 | event | Day 30: birth (person #36) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/97 |
| P1A-C0019 | event | Day 32: birth (person #40) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/109 |
| P1A-C0489 | visual | Village-designated parents qualifier visible | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0543 | artifact_observation | names move intact onto next lines; no internal split observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0616 | named_relationship | 니티소, 바호 parents | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0617 | named_relationship | 시키노 day-6/day-30 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0618 | named_relationship | 데와 day-30 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0619 | named_relationship | 토토수 day-30 child | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0620 | named_relationship | 요리 day-32 child. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0025 | 가와 | 가와 | CORRECT |
| P1A-N0026 | 니티소 | 니티소 | CORRECT |
| P1A-N0027 | 바호 | 바호 | CORRECT |
| P1A-N0028 | 시키노 | 시키노 | CORRECT |
| P1A-N0029 | 데와 | 데와 | CORRECT |
| P1A-N0030 | 토토수 | 토토수 | CORRECT |
| P1A-N0031 | 요리 | 요리 | CORRECT |

### image_0008.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/living_dead_parent_ko_page_003.png`; SHA-256 `6b83407bb6193d49f4e060abcc2f0e4b0a2b9875de4761585990859391cca15c`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/7` and `/reference_worlds/0`.

Facts: Day 6 partnered; day 30 partner lost/new partnership/child born; day 32 child born; day 47 lost partner, partnered, elder; day 49 child born.
Names: 시키노, 데와, 토토수, 요리, 비무오, 가와. Artifacts: intact names on own lines; no internal split observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0020 | event | Day 6: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/39 |
| P1A-C0021 | event | Day 30: lost partner (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/89 |
| P1A-C0022 | event | Day 30: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0023 | event | Day 30: birth (person #36) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/97 |
| P1A-C0024 | event | Day 32: birth (person #40) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/109 |
| P1A-C0025 | event | Day 47: lost partner (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/153 |
| P1A-C0026 | event | Day 47: partnership (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/155 |
| P1A-C0027 | event | Day 47: became elder (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/156 |
| P1A-C0028 | event | Day 49: birth (person #50) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/161 |
| P1A-C0544 | artifact_observation | intact names on own lines; no internal split observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0032 | 시키노 | 시키노 | CORRECT |
| P1A-N0033 | 데와 | 데와 | CORRECT |
| P1A-N0034 | 토토수 | 토토수 | CORRECT |
| P1A-N0035 | 요리 | 요리 | CORRECT |
| P1A-N0036 | 비무오 | 비무오 | CORRECT |
| P1A-N0037 | 가와 | 가와 | CORRECT |

### image_0009.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_en_page_000.png`; SHA-256 `6efdb039f3a4810f53363e880c0e927a0b3166da525a7505bb4acba6057f6a30`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/8` and `/reference_worlds/0`.

Facts: #0 deceased founder, died day 23, cause old age. Male, deceased spouse, mother/father None. Day 1 event clipped after 'One of the'.
Names: Nitiso; Baho spouse. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0283 | state | Selected #0 | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0284 | state | Deceased | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0285 | state | Founder | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0286 | state | Died day 23 | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0287 | state | Death cause old age | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0288 | state | Male | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0289 | state | Spouse deceased | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0290 | state | Mother None | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0291 | state | Father None | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0490 | visual | Day 1 event clipped after One of the | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0545 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0621 | named_relationship | Baho spouse. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0038 | Nitiso | Nitiso | CORRECT |
| P1A-N0039 | Baho | Baho | CORRECT |

### image_0010.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_en_page_001.png`; SHA-256 `4f291dbeda9d6b6fc56bcfc66f559055c2b7b02a342d1923a3ce73a8fc50fa90`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/9` and `/reference_worlds/0`.

Facts: Day 1 founder of Village 1; child births day 1, twice day 2, day 3; day 4 partnered; day 12 elder. Day 18 child event partially clipped, cannot confirm full sentence.
Names: Ikimu day 1; Timiga and Mabito day 2; Yubama day 3; Baho partner; clipped day-18 name appears Simowa (uncertain). Artifacts: bottom clipping, no split name otherwise.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0029 | event | Day 1: founder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/1 |
| P1A-C0030 | event | Day 1: birth (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/26 |
| P1A-C0031 | event | Day 2: birth (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/27 |
| P1A-C0032 | event | Day 2: birth (person #27) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/28 |
| P1A-C0033 | event | Day 3: birth (person #28) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/29 |
| P1A-C0034 | event | Day 4: partnership (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/32 |
| P1A-C0035 | event | Day 12: became elder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/49 |
| P1A-C0491 | visual | Day 18 child event partially clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0529 | abstention | Full clipped day18 sentence not confirmed | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0546 | artifact_observation | bottom clipping, no split name otherwise. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0622 | named_relationship | Baho partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0040 | Ikimu | Ikimu | CORRECT |
| P1A-N0041 | Timiga | Timiga | CORRECT |
| P1A-N0042 | Mabito | Mabito | CORRECT |
| P1A-N0043 | Yubama | Yubama | CORRECT |
| P1A-N0044 | Baho | Baho | CORRECT |
| P1A-N0045 | Simowa | Simowa | CORRECT_PARTIAL_UNCERTAIN |

### image_0011.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_en_page_002.png`; SHA-256 `e6acacba2e6a9c1c4610a0d10f84f7f0eda9de1b5a4d4369dd3c862b83f5d9c3`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/10` and `/reference_worlds/0`.

Facts: Child births day 2 twice, day 3, day 18; partnership day 4; elder day 12; died day 23 aged 56 days of old age.
Names: Timiga, Mabito, Yubama, Simowa children; Baho partner. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0036 | event | Day 2: birth (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/27 |
| P1A-C0037 | event | Day 2: birth (person #27) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/28 |
| P1A-C0038 | event | Day 3: birth (person #28) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/29 |
| P1A-C0039 | event | Day 18: birth (person #29) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/58 |
| P1A-C0040 | event | Day 4: partnership (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/32 |
| P1A-C0041 | event | Day 12: became elder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/49 |
| P1A-C0042 | event | Day 23: death (person #0); age 56; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/69 |
| P1A-C0547 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0623 | named_relationship | Timiga, Mabito, Yubama, Simowa children | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0624 | named_relationship | Baho partner. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0046 | Timiga | Timiga | CORRECT |
| P1A-N0047 | Mabito | Mabito | CORRECT |
| P1A-N0048 | Yubama | Yubama | CORRECT |
| P1A-N0049 | Simowa | Simowa | CORRECT |
| P1A-N0050 | Baho | Baho | CORRECT |

### image_0012.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_ko_page_000.png`; SHA-256 `7d4c59eaa27131168d3309ae2c96e6a6ad06f85bc476b1169233f51d2dc2e472`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/11` and `/reference_worlds/0`.

Facts: #0 deceased first resident, died day 23 of old age; male, spouse deceased, no mother/father.
Names: 니티소; 바호 spouse. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0292 | state | Selected #0 | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0293 | state | Deceased | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0294 | state | First resident | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0295 | state | Died day 23 | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0296 | state | Death cause old age | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0297 | state | Male | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0298 | state | Spouse deceased | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0299 | state | Mother None | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0300 | state | Father None | CORRECT | /reference_worlds/0/persons/0 |
| P1A-C0548 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0625 | named_relationship | 바호 spouse. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0051 | 니티소 | 니티소 | CORRECT |
| P1A-N0052 | 바호 | 바호 | CORRECT |

### image_0013.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_ko_page_001.png`; SHA-256 `3cf1c0eb1d90a5702b217d8d3522e6f1db44184853e117f715ce78c480da3372`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/12` and `/reference_worlds/0`.

Facts: Day 1 first resident Village 1; child day 1, twice day 2, day 3, day 18; partnered day 4; elder day 12. Day-18 name below viewport.
Names: 이키무, 티미가, 마비토, 유바마 children; 바호 partner. Artifacts: intact names on separate lines, bottom clipping.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0043 | event | Day 1: founder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/1 |
| P1A-C0044 | event | Day 1: birth (person #25) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/26 |
| P1A-C0045 | event | Day 2: birth (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/27 |
| P1A-C0046 | event | Day 2: birth (person #27) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/28 |
| P1A-C0047 | event | Day 3: birth (person #28) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/29 |
| P1A-C0048 | event | Day 18: birth (person #29) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/58 |
| P1A-C0049 | event | Day 4: partnership (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/32 |
| P1A-C0050 | event | Day 12: became elder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/49 |
| P1A-C0492 | visual | Day 18 name below viewport | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0549 | artifact_observation | intact names on separate lines, bottom clipping. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0626 | named_relationship | 이키무, 티미가, 마비토, 유바마 children | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0627 | named_relationship | 바호 partner. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0053 | 이키무 | 이키무 | CORRECT |
| P1A-N0054 | 티미가 | 티미가 | CORRECT |
| P1A-N0055 | 마비토 | 마비토 | CORRECT |
| P1A-N0056 | 유바마 | 유바마 | CORRECT |
| P1A-N0057 | 바호 | 바호 | CORRECT |

### image_0014.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/dead_parent_ko_page_002.png`; SHA-256 `5bab06a5b7c606be7eb055f32a5369597e296ea48c4e2e17b8fd364f89d7bfc0`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/13` and `/reference_worlds/0`.

Facts: Births twice day 2, day 3, day 18; partnership day 4; elder day 12; death day 23 after living 56 days, old age.
Names: 이키무 at top; 티미가, 마비토, 유바마, 시모와 children; 바호 partner. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0051 | event | Day 2: birth (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/27 |
| P1A-C0052 | event | Day 2: birth (person #27) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/28 |
| P1A-C0053 | event | Day 3: birth (person #28) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/29 |
| P1A-C0054 | event | Day 18: birth (person #29) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/58 |
| P1A-C0055 | event | Day 4: partnership (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/32 |
| P1A-C0056 | event | Day 12: became elder (person #0) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/49 |
| P1A-C0057 | event | Day 23: death (person #0); age 56; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/69 |
| P1A-C0550 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0628 | named_relationship | 티미가, 마비토, 유바마, 시모와 children | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0629 | named_relationship | 바호 partner. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0058 | 이키무 | 이키무 | CORRECT |
| P1A-N0059 | 티미가 | 티미가 | CORRECT |
| P1A-N0060 | 마비토 | 마비토 | CORRECT |
| P1A-N0061 | 유바마 | 유바마 | CORRECT |
| P1A-N0062 | 시모와 | 시모와 | CORRECT |
| P1A-N0063 | 바호 | 바호 | CORRECT |

### image_0015.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_000.png`; SHA-256 `3f4e841168150dcf81b3a4f68545a453d497cf3a68e2d0de5d56088bd7c99329`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/14` and `/reference_worlds/0`.

Facts: Chronicle day 59 partnership; day 58 partnership, birth Village 1, death age 54 old age; day 57 birth Village 1.
Names: Katago partnered Kohua; Simowa partnered Nuyoe; Haritu born to Buyobi/Totosu; Yubama died; Oemu born to Tahuku/Yubama. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0058 | event | Day 59: partnership (person #48) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/184 |
| P1A-C0059 | event | Day 58: partnership (person #29) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/183 |
| P1A-C0060 | event | Day 58: birth (person #56); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/182 |
| P1A-C0061 | event | Day 58: death (person #28); age 54; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/180 |
| P1A-C0062 | event | Day 57: birth (person #55); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/178 |
| P1A-C0551 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0630 | named_relationship | Katago partnered Kohua | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0631 | named_relationship | Simowa partnered Nuyoe | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0632 | named_relationship | Haritu born to Buyobi/Totosu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0633 | named_relationship | Yubama died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0634 | named_relationship | Oemu born to Tahuku/Yubama. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0064 | Katago | Katago | CORRECT |
| P1A-N0065 | Kohua | Kohua | CORRECT |
| P1A-N0066 | Simowa | Simowa | CORRECT |
| P1A-N0067 | Nuyoe | Nuyoe | CORRECT |
| P1A-N0068 | Haritu | Haritu | CORRECT |
| P1A-N0069 | Buyobi | Buyobi | CORRECT |
| P1A-N0070 | Totosu | Totosu | CORRECT |
| P1A-N0071 | Yubama | Yubama | CORRECT |
| P1A-N0072 | Oemu | Demu | INCORRECT |
| P1A-N0073 | Tahuku | Tahuku | CORRECT |
| P1A-N0074 | Yubama | Yubama | CORRECT |

### image_0016.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_001.png`; SHA-256 `c295ec60561dab75f3affa0d75e8d41409fdb0c8cb60ffff7e77b1428f1db86f`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/15` and `/reference_worlds/0`.

Facts: Day 57 birth Village 1, death aged 55 old age, another birth Village 1, death aged 3 cold; day 53 birth Village 1.
Names: Oemu parents Tahuku/Yubama; Timiga died; Ohuti parents Gakaho/Wano; Yunoni died; Nuyoe parents Taruka/Rabati. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0063 | event | Day 57: birth (person #55); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/178 |
| P1A-C0064 | event | Day 57: death (person #26); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/177 |
| P1A-C0065 | event | Day 57: birth (person #54); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/176 |
| P1A-C0066 | event | Day 57: death (person #52); age 3; cold | CORRECT | /reference_worlds/0/chronicle_pages/0/*/175 |
| P1A-C0067 | event | Day 53: birth (person #53); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/173 |
| P1A-C0552 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0635 | named_relationship | Oemu parents Tahuku/Yubama | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0636 | named_relationship | Timiga died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0637 | named_relationship | Ohuti parents Gakaho/Wano | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0638 | named_relationship | Yunoni died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0639 | named_relationship | Nuyoe parents Taruka/Rabati. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0075 | Oemu | Demu | INCORRECT |
| P1A-N0076 | Tahuku | Tahuku | CORRECT |
| P1A-N0077 | Yubama | Yubama | CORRECT |
| P1A-N0078 | Timiga | Timiga | CORRECT |
| P1A-N0079 | Ohuti | Ohuti | CORRECT |
| P1A-N0080 | Gakaho | Gakaho | CORRECT |
| P1A-N0081 | Wano | Wano | CORRECT |
| P1A-N0082 | Yunoni | Yunoni | CORRECT |
| P1A-N0083 | Nuyoe | Nuyoe | CORRECT |
| P1A-N0084 | Taruka | Tanuka | INCORRECT |
| P1A-N0085 | Rabati | Rabati | CORRECT |

### image_0017.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_002.png`; SHA-256 `a815c890d91cbf84401d0d7892234dc13bd3a3a6d860974b0ebb9b8e4b57158e`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/16` and `/reference_worlds/0`.

Facts: Day 53 partnership; death aged 62 old age; birth Village 1; death aged 58 old age. Top previous birth line clipped.
Names: Taruka partnered Rabati; Tubido died; Yunoni parents Bisoki/Godosa; Yuruge died; Nuyoe partially visible top. Artifacts: top clipping, none otherwise.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0068 | event | Day 53: partnership (person #44) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/172 |
| P1A-C0069 | event | Day 53: death (person #15); age 62; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/171 |
| P1A-C0070 | event | Day 53: birth (person #52); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0071 | event | Day 53: death (person #8); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/169 |
| P1A-C0493 | visual | Top previous birth line clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0553 | artifact_observation | top clipping, none otherwise. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0640 | named_relationship | Taruka partnered Rabati | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0641 | named_relationship | Tubido died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0642 | named_relationship | Yunoni parents Bisoki/Godosa | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0643 | named_relationship | Yuruge died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0086 | Taruka | Tanuka | INCORRECT |
| P1A-N0087 | Rabati | Rabati | CORRECT |
| P1A-N0088 | Tubido | Tubido | CORRECT |
| P1A-N0089 | Yunoni | Yunoni | CORRECT |
| P1A-N0090 | Bisoki | Bisoki | CORRECT |
| P1A-N0091 | Godosa | Godosa | CORRECT |
| P1A-N0092 | Yuruge | Yuruge | CORRECT |
| P1A-N0093 | Nuyoe | Nuyoe | CORRECT_PARTIAL_UNCERTAIN |

### image_0018.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_003.png`; SHA-256 `6e09417566a32d13ba5776cbc61ebb669fb857ae15acd228289370010a495138`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/17` and `/reference_worlds/0`.

Facts: Day 51 birth Village 1 and death aged 59 old age; day 49 birth Village 1 and death aged 55 old age. Previous event top clipped.
Names: Nieba parents Kuroku/Haa; Nobibu died; Gawa parents Bimuo/Ikimu; Bosobu died. Artifacts: none observed beyond clipping.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0072 | event | Day 51: birth (person #51); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/166 |
| P1A-C0073 | event | Day 51: death (person #14); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/164 |
| P1A-C0074 | event | Day 49: birth (person #50); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/161 |
| P1A-C0075 | event | Day 49: death (person #4); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/159 |
| P1A-C0494 | visual | Previous event top clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0554 | artifact_observation | none observed beyond clipping. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0644 | named_relationship | Nieba parents Kuroku/Haa | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0645 | named_relationship | Nobibu died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0646 | named_relationship | Gawa parents Bimuo/Ikimu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0647 | named_relationship | Bosobu died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0094 | Nieba | Nieba | CORRECT |
| P1A-N0095 | Kuroku | Kuroku | CORRECT |
| P1A-N0096 | Haa | Haa | CORRECT |
| P1A-N0097 | Nobibu | Nobibu | CORRECT |
| P1A-N0098 | Gawa | Gawa | CORRECT |
| P1A-N0099 | Bimuo | Bimuo | CORRECT |
| P1A-N0100 | Ikimu | Ikimu | CORRECT |
| P1A-N0101 | Bosobu | Bosobu | CORRECT |

### image_0019.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_004.png`; SHA-256 `e240db40f79a9f79acb0e9ff13ad28808554dd98989af4781a7552dadf6ce577`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/18` and `/reference_worlds/0`.

Facts: Day 47 partnership, birth Village 1, death age 57 old age; day 46 birth Village 1 and death age 55 (cause line clipped after old).
Names: Bimuo partnered Ikimu; Kohua parents Buyobi/Totosu; Dewa died; Katago parents Tahuku/Yubama; Batiho died. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0076 | event | Day 47: partnership (person #39) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/155 |
| P1A-C0077 | event | Day 47: birth (person #49); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/154 |
| P1A-C0078 | event | Day 47: death (person #12); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/152 |
| P1A-C0079 | event | Day 46: birth (person #48); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/151 |
| P1A-C0080 | event | Day 46: death (person #2); age 55 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/149 |
| P1A-C0495 | visual | Day 46 death cause line clipped after old | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0555 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0648 | named_relationship | Bimuo partnered Ikimu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0649 | named_relationship | Kohua parents Buyobi/Totosu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0650 | named_relationship | Dewa died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0651 | named_relationship | Katago parents Tahuku/Yubama | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0652 | named_relationship | Batiho died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0102 | Bimuo | Bimuo | CORRECT |
| P1A-N0103 | Ikimu | Ikimu | CORRECT |
| P1A-N0104 | Kohua | Kohua | CORRECT |
| P1A-N0105 | Buyobi | Buyobi | CORRECT |
| P1A-N0106 | Totosu | Totosu | CORRECT |
| P1A-N0107 | Dewa | Dewa | CORRECT |
| P1A-N0108 | Katago | Katago | CORRECT |
| P1A-N0109 | Tahuku | Tahuku | CORRECT |
| P1A-N0110 | Yubama | Yubama | CORRECT |
| P1A-N0111 | Batiho | Batiho | CORRECT |

### image_0020.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_005.png`; SHA-256 `08e7cd7db459110588394f68637dfa13bdcc93f2dfbe052da0652d8e09141b8f`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/19` and `/reference_worlds/0`.

Facts: Day 46 death aged 55 old age; partnerships day 45 twice and day 44; day 41 partnership line clipped before partner name.
Names: Batiho died; Bisoki/Godosa, Kuroku/Haa, Tahuku/Yubama partnerships; Buyobi day 41. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0081 | event | Day 46: death (person #2); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/149 |
| P1A-C0082 | event | Day 45: partnership (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/148 |
| P1A-C0083 | event | Day 45: partnership (person #43) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/147 |
| P1A-C0084 | event | Day 44: partnership (person #32) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/145 |
| P1A-C0085 | event | Day 41: partnership (person #35) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/143 |
| P1A-C0496 | visual | Day 41 partnership clipped before partner name | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0556 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0653 | named_relationship | Batiho died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0112 | Batiho | Batiho | CORRECT |
| P1A-N0113 | Bisoki | Bisoki | CORRECT |
| P1A-N0114 | Godosa | Godosa | CORRECT |
| P1A-N0115 | Kuroku | Kuroku | CORRECT |
| P1A-N0116 | Haa | Haa | CORRECT |
| P1A-N0117 | Tahuku | Tahuku | CORRECT |
| P1A-N0118 | Yubama | Yubama | CORRECT |
| P1A-N0119 | Buyobi | Buyobi | CORRECT |

### image_0021.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_006.png`; SHA-256 `41afc87acdff47bbf42ad9eff0faecaffda3a2e277eda5d6b9958ac36ebe14ee`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/20` and `/reference_worlds/0`.

Facts: Day 41 partnership, birth Village 1, death age 56 old age; day 40 birth Village 1, death age 55 with cause clipped after old.
Names: Buyobi/Totosu partners; Bisoki parents Gakaho/Wano; Hubo died; Godosa parents Timiga/Nobibu; Baho died. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0086 | event | Day 41: partnership (person #35) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/143 |
| P1A-C0087 | event | Day 41: birth (person #47); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/140 |
| P1A-C0088 | event | Day 41: death (person #9); age 56; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0089 | event | Day 40: birth (person #46); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/136 |
| P1A-C0090 | event | Day 40: death (person #1); age 55 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/135 |
| P1A-C0497 | visual | Day 40 death cause clipped after old | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0557 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0654 | named_relationship | Buyobi/Totosu partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0655 | named_relationship | Bisoki parents Gakaho/Wano | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0656 | named_relationship | Hubo died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0657 | named_relationship | Godosa parents Timiga/Nobibu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0658 | named_relationship | Baho died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0120 | Buyobi | Buyobi | CORRECT |
| P1A-N0121 | Totosu | Totosu | CORRECT |
| P1A-N0122 | Bisoki | Bisoki | CORRECT |
| P1A-N0123 | Gakaho | Gakaho | CORRECT |
| P1A-N0124 | Wano | Wano | CORRECT |
| P1A-N0125 | Hubo | Hubo | CORRECT |
| P1A-N0126 | Godosa | Godosa | CORRECT |
| P1A-N0127 | Timiga | Timiga | CORRECT |
| P1A-N0128 | Nobibu | Nobibu | CORRECT |
| P1A-N0129 | Baho | Baho | CORRECT |

### image_0022.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_007.png`; SHA-256 `e4beb572f334b12c0c842f0bfe4d13aa0821fd83fc3e4758c49052a814bf5730`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/21` and `/reference_worlds/0`.

Facts: Day 40 death 55 old age; day 37 two Village 1 births, death 58 old age, death 61 cause clipped after old.
Names: Baho died; Rabati parents Gakaho/Wano; Kumihu died; Taruka parents Yuruge/Bosobu; Huesi died. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0091 | event | Day 40: death (person #1); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/135 |
| P1A-C0092 | event | Day 37: birth (person #45); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/132 |
| P1A-C0093 | event | Day 37: death (person #11); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/131 |
| P1A-C0094 | event | Day 37: birth (person #44); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/128 |
| P1A-C0095 | event | Day 37: death (person #18); age 61 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/127 |
| P1A-C0498 | visual | Day 37 death cause clipped after old | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0558 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0659 | named_relationship | Baho died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0660 | named_relationship | Rabati parents Gakaho/Wano | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0661 | named_relationship | Kumihu died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0662 | named_relationship | Taruka parents Yuruge/Bosobu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0663 | named_relationship | Huesi died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0130 | Baho | Baho | CORRECT |
| P1A-N0131 | Rabati | Rabati | CORRECT |
| P1A-N0132 | Gakaho | Gakaho | CORRECT |
| P1A-N0133 | Wano | Wano | CORRECT |
| P1A-N0134 | Kumihu | Kumihu | CORRECT |
| P1A-N0135 | Taruka | Tanuka | INCORRECT |
| P1A-N0136 | Yuruge | Yuruge | CORRECT |
| P1A-N0137 | Bosobu | Bosobu | CORRECT |
| P1A-N0138 | Huesi | Huesi | CORRECT |

### image_0023.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_008.png`; SHA-256 `5b7c54e0eee423970e31143dee09f3fcc040d809f77c8f45d806ed250fbaf034`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/22` and `/reference_worlds/0`.

Facts: Day 37 death 61 old age; day 36 partnership, birth Village 1, death 59 old age; day 35 birth Village 1.
Names: Huesi died; Yuruge/Bosobu partners; Kuroku parents Timiga/Nobibu; Tinuyu died; Haa parents Tubido/Batiho. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0096 | event | Day 37: death (person #18); age 61; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/127 |
| P1A-C0097 | event | Day 36: partnership (person #8) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/125 |
| P1A-C0098 | event | Day 36: birth (person #43); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/121 |
| P1A-C0099 | event | Day 36: death (person #5); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/119 |
| P1A-C0100 | event | Day 35: birth (person #42); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/117 |
| P1A-C0559 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0664 | named_relationship | Huesi died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0665 | named_relationship | Yuruge/Bosobu partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0666 | named_relationship | Kuroku parents Timiga/Nobibu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0667 | named_relationship | Tinuyu died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0668 | named_relationship | Haa parents Tubido/Batiho. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0139 | Huesi | Huesi | CORRECT |
| P1A-N0140 | Yuruge | Yuruge | CORRECT |
| P1A-N0141 | Bosobu | Bosobu | CORRECT |
| P1A-N0142 | Kuroku | Kuroku | CORRECT |
| P1A-N0143 | Timiga | Timiga | CORRECT |
| P1A-N0144 | Nobibu | Nobibu | CORRECT |
| P1A-N0145 | Tinuyu | Tinuyu | CORRECT |
| P1A-N0146 | Haa | Haa | CORRECT |
| P1A-N0147 | Tubido | Tubido | CORRECT |
| P1A-N0148 | Batiho | Batiho | CORRECT |

### image_0024.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_009.png`; SHA-256 `520a5843d7147b8334897660657ffae7ba480f519ff98c13a17a1f33460ec3af`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/23` and `/reference_worlds/0`.

Facts: Day 35 birth Village 1 and death 58 old age; day 33 birth Village 1; day 32 birth Village 1 and death 60 old age.
Names: Haa parents Tubido/Batiho; Tiea died; Anunu parents Gakaho/Wano; Yori parents Dewa/Ikimu; Oehona died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0101 | event | Day 35: birth (person #42); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/117 |
| P1A-C0102 | event | Day 35: death (person #23); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/116 |
| P1A-C0103 | event | Day 33: birth (person #41); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/110 |
| P1A-C0104 | event | Day 32: birth (person #40); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/109 |
| P1A-C0105 | event | Day 32: death (person #3); age 60; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/107 |
| P1A-C0560 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0669 | named_relationship | Haa parents Tubido/Batiho | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0670 | named_relationship | Tiea died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0671 | named_relationship | Anunu parents Gakaho/Wano | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0672 | named_relationship | Yori parents Dewa/Ikimu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0673 | named_relationship | Oehona died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0149 | Haa | Haa | CORRECT |
| P1A-N0150 | Tubido | Tubido | CORRECT |
| P1A-N0151 | Batiho | Batiho | CORRECT |
| P1A-N0152 | Tiea | Tiea | CORRECT |
| P1A-N0153 | Anunu | Anunu | CORRECT |
| P1A-N0154 | Gakaho | Gakaho | CORRECT |
| P1A-N0155 | Wano | Wano | CORRECT |
| P1A-N0156 | Yori | Yori | CORRECT |
| P1A-N0157 | Dewa | Dewa | CORRECT |
| P1A-N0158 | Ikimu | Ikimu | CORRECT |
| P1A-N0159 | Oehona | Dehona | INCORRECT |

### image_0025.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_010.png`; SHA-256 `2d1ee24dfb52a88f81b854068dc57a4314b59e8fe2387bb4bd9380bf79328bae`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/24` and `/reference_worlds/0`.

Facts: Day 32 death 60 old age and birth Village 1; day 31 birth Village 1, deaths 59 and 62 old age.
Names: Oehona died; Bimuo parents Timiga/Nobibu; Sogeru parents Tubido/Batiho; Tasoe and Sahi died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0106 | event | Day 32: death (person #3); age 60; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/107 |
| P1A-C0107 | event | Day 32: birth (person #39); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/106 |
| P1A-C0108 | event | Day 31: birth (person #38); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/104 |
| P1A-C0109 | event | Day 31: death (person #22); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/103 |
| P1A-C0110 | event | Day 31: death (person #17); age 62; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/101 |
| P1A-C0561 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0674 | named_relationship | Oehona died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0675 | named_relationship | Bimuo parents Timiga/Nobibu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0676 | named_relationship | Sogeru parents Tubido/Batiho | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0677 | named_relationship | Tasoe and Sahi died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0160 | Oehona | Dehona | INCORRECT |
| P1A-N0161 | Bimuo | Bimuo | CORRECT |
| P1A-N0162 | Timiga | Timiga | CORRECT |
| P1A-N0163 | Nobibu | Nobibu | CORRECT |
| P1A-N0164 | Sogeru | Sogaru | INCORRECT |
| P1A-N0165 | Tubido | Tubido | CORRECT |
| P1A-N0166 | Batiho | Batiho | CORRECT |
| P1A-N0167 | Tasoe | Tasoe | CORRECT |
| P1A-N0168 | Sahi | Sahi | CORRECT |

### image_0026.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_011.png`; SHA-256 `4645793bc1716d5de1f442b57c38f5e9c7f909ccfee7186575ddb38b3ddb1f9b`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/25` and `/reference_worlds/0`.

Facts: Day 31 deaths 62 and 53 old age, birth Village 1; day 30 birth Village 1 and partnership.
Names: Sahi died; Kiyu parents Gakaho/Wano; Gawa died; Totosu parents Dewa/Ikimu; Dewa partnered Ikimu. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0111 | event | Day 31: death (person #17); age 62; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/101 |
| P1A-C0112 | event | Day 31: birth (person #37); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/100 |
| P1A-C0113 | event | Day 31: death (person #24); age 53; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/98 |
| P1A-C0114 | event | Day 30: birth (person #36); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/97 |
| P1A-C0115 | event | Day 30: partnership (person #12) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0562 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0678 | named_relationship | Sahi died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0679 | named_relationship | Kiyu parents Gakaho/Wano | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0680 | named_relationship | Gawa died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0681 | named_relationship | Totosu parents Dewa/Ikimu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0682 | named_relationship | Dewa partnered Ikimu. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0169 | Sahi | Sahi | CORRECT |
| P1A-N0170 | Kiyu | Kiyu | CORRECT |
| P1A-N0171 | Gakaho | Gakaho | CORRECT |
| P1A-N0172 | Wano | Wano | CORRECT |
| P1A-N0173 | Gawa | Gawa | CORRECT |
| P1A-N0174 | Totosu | Totosu | CORRECT |
| P1A-N0175 | Dewa | Dewa | CORRECT |
| P1A-N0176 | Ikimu | Ikimu | CORRECT |
| P1A-N0177 | Dewa | Dewa | CORRECT |
| P1A-N0178 | Ikimu | Ikimu | CORRECT |

### image_0027.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_012.png`; SHA-256 `161f40637f14b2891aac451cbec0903f1c580a66cde9a48b21708db947be119c`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/26` and `/reference_worlds/0`.

Facts: Day 30 partnership, two deaths aged 57 old age, birth Village 1, death 58 old age.
Names: Dewa/Ikimu; Birige and Etuba died; Buyobi parents Mabito/Gawa; Sikino died. Artifacts: top clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0116 | event | Day 30: partnership (person #12) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0117 | event | Day 30: death (person #16); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/95 |
| P1A-C0118 | event | Day 30: death (person #10); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/93 |
| P1A-C0119 | event | Day 30: birth (person #35); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/92 |
| P1A-C0120 | event | Day 30: death (person #7); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/88 |
| P1A-C0563 | artifact_observation | top clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0683 | named_relationship | Birige and Etuba died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0684 | named_relationship | Buyobi parents Mabito/Gawa | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0685 | named_relationship | Sikino died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0179 | Dewa | Dewa | CORRECT |
| P1A-N0180 | Ikimu | Ikimu | CORRECT |
| P1A-N0181 | Birige | Birige | CORRECT |
| P1A-N0182 | Etuba | Etuba | CORRECT |
| P1A-N0183 | Buyobi | Buyobi | CORRECT |
| P1A-N0184 | Mabito | Mabito | CORRECT |
| P1A-N0185 | Gawa | Gawa | CORRECT |
| P1A-N0186 | Sikino | Sikino | CORRECT |

### image_0028.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_013.png`; SHA-256 `2c66dff3f7a7235e3cbefcc0baedb64c6768fdef254b2835ec3f907c4505543b`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/27` and `/reference_worlds/0`.

Facts: Day 29 birth Village 1, death 52 old age; day 28 partnership and birth Village 1. Prior top death clipped.
Names: Gayo parents Timiga/Nobibu; Timiku died; Gakaho/Wano partners; Deyo parents Tubido/Batiho. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0121 | event | Day 29: birth (person #34); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/87 |
| P1A-C0122 | event | Day 29: death (person #13); age 52; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/85 |
| P1A-C0123 | event | Day 28: partnership (person #31) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/83 |
| P1A-C0124 | event | Day 28: birth (person #33); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/82 |
| P1A-C0499 | visual | Prior top death clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0564 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0686 | named_relationship | Gayo parents Timiga/Nobibu | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0687 | named_relationship | Timiku died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0688 | named_relationship | Gakaho/Wano partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0689 | named_relationship | Deyo parents Tubido/Batiho. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0187 | Gayo | Gayo | CORRECT |
| P1A-N0188 | Timiga | Timiga | CORRECT |
| P1A-N0189 | Nobibu | Nobibu | CORRECT |
| P1A-N0190 | Timiku | Timiku | CORRECT |
| P1A-N0191 | Gakaho | Gakaho | CORRECT |
| P1A-N0192 | Wano | Wano | CORRECT |
| P1A-N0193 | Deyo | Deyo | CORRECT |
| P1A-N0194 | Tubido | Tubido | CORRECT |
| P1A-N0195 | Batiho | Batiho | CORRECT |

### image_0029.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_en_page_014.png`; SHA-256 `6016571adf6f3cbd51eed00847be876105a31c8d5018de92deaceb1613b29f7a`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/28` and `/reference_worlds/0`.

Facts: Day 29 death 52 old age; day 28 partnership, birth Village 1, death 59 old age; day 25 partnership.
Names: Timiku died; Gakaho/Wano partners; Deyo parents Tubido/Batiho; Miyuna died; Timiga/Nobibu partners. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0125 | event | Day 29: death (person #13); age 52; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/85 |
| P1A-C0126 | event | Day 28: partnership (person #31) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/83 |
| P1A-C0127 | event | Day 28: birth (person #33); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/82 |
| P1A-C0128 | event | Day 28: death (person #6); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/80 |
| P1A-C0129 | event | Day 25: partnership (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/78 |
| P1A-C0565 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0690 | named_relationship | Timiku died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0691 | named_relationship | Gakaho/Wano partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0692 | named_relationship | Deyo parents Tubido/Batiho | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0693 | named_relationship | Miyuna died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0694 | named_relationship | Timiga/Nobibu partners. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0196 | Timiku | Timiku | CORRECT |
| P1A-N0197 | Gakaho | Gakaho | CORRECT |
| P1A-N0198 | Wano | Wano | CORRECT |
| P1A-N0199 | Deyo | Deyo | CORRECT |
| P1A-N0200 | Tubido | Tubido | CORRECT |
| P1A-N0201 | Batiho | Batiho | CORRECT |
| P1A-N0202 | Miyuna | Miyuna | CORRECT |
| P1A-N0203 | Timiga | Timiga | CORRECT |
| P1A-N0204 | Nobibu | Nobibu | CORRECT |

### image_0030.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_000.png`; SHA-256 `8240f3afc527dd0135481602fa88e84635f517874eaad888da5bdced39a1a184`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/29` and `/reference_worlds/0`.

Facts: Day 59 partnership; day 58 partnership, birth Village 1, death 54 old age; day 57 birth Village 1, death 55 old age.
Names: 카타고/코후아; 시모와/누요에; 하리투 parents 부요비/토토수; 유바마 died; 데무 parents 타후쿠/유바마; 티미가 died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0130 | event | Day 59: partnership (person #48) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/184 |
| P1A-C0131 | event | Day 58: partnership (person #29) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/183 |
| P1A-C0132 | event | Day 58: birth (person #56); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/182 |
| P1A-C0133 | event | Day 58: death (person #28); age 54; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/180 |
| P1A-C0134 | event | Day 57: birth (person #55); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/178 |
| P1A-C0135 | event | Day 57: death (person #26); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/177 |
| P1A-C0566 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0695 | named_relationship | 하리투 parents 부요비/토토수 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0696 | named_relationship | 유바마 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0697 | named_relationship | 데무 parents 타후쿠/유바마 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0698 | named_relationship | 티미가 died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0205 | 카타고 | 카타고 | CORRECT |
| P1A-N0206 | 코후아 | 코후아 | CORRECT |
| P1A-N0207 | 시모와 | 시모와 | CORRECT |
| P1A-N0208 | 누요에 | 누요에 | CORRECT |
| P1A-N0209 | 하리투 | 하리투 | CORRECT |
| P1A-N0210 | 부요비 | 부요비 | CORRECT |
| P1A-N0211 | 토토수 | 토토수 | CORRECT |
| P1A-N0212 | 유바마 | 유바마 | CORRECT |
| P1A-N0213 | 데무 | 데무 | CORRECT |
| P1A-N0214 | 타후쿠 | 타후쿠 | CORRECT |
| P1A-N0215 | 유바마 | 유바마 | CORRECT |
| P1A-N0216 | 티미가 | 티미가 | CORRECT |

### image_0031.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_001.png`; SHA-256 `3263877391feb0eb473b750672bb906f6bc8e22f3f6e06cdd0dd6243fc276f1a`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/30` and `/reference_worlds/0`.

Facts: Top clipped day-57 death 55 old age; day 57 birth Village 1 and death 3 cold; day 53 birth Village 1, partnership, death 62 old age, birth Village 1 with parents clipped.
Names: 티미가; 오후티 parents 가카호/와노; 유노니 died; 누요에 parents 타누카/라바티; 타누카/라바티 partners; 투비도 died; 유노니 born. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0136 | event | Day 57: death (person #26); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/177 |
| P1A-C0137 | event | Day 57: birth (person #54); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/176 |
| P1A-C0138 | event | Day 57: death (person #52); age 3; cold | CORRECT | /reference_worlds/0/chronicle_pages/0/*/175 |
| P1A-C0139 | event | Day 53: birth (person #53); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/173 |
| P1A-C0140 | event | Day 53: partnership (person #44) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/172 |
| P1A-C0141 | event | Day 53: death (person #15); age 62; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/171 |
| P1A-C0142 | event | Day 53: birth (person #52); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0500 | visual | Top day57 death clipped; bottom parents clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0567 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0699 | named_relationship | 오후티 parents 가카호/와노 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0700 | named_relationship | 유노니 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0701 | named_relationship | 누요에 parents 타누카/라바티 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0702 | named_relationship | 타누카/라바티 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0703 | named_relationship | 투비도 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0704 | named_relationship | 유노니 born. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0217 | 티미가 | 티미가 | CORRECT |
| P1A-N0218 | 오후티 | 오후티 | CORRECT |
| P1A-N0219 | 가카호 | 가카호 | CORRECT |
| P1A-N0220 | 와노 | 와노 | CORRECT |
| P1A-N0221 | 유노니 | 유노니 | CORRECT |
| P1A-N0222 | 누요에 | 누요에 | CORRECT |
| P1A-N0223 | 타누카 | 타누카 | CORRECT |
| P1A-N0224 | 라바티 | 라바티 | CORRECT |
| P1A-N0225 | 타누카 | 타누카 | CORRECT |
| P1A-N0226 | 라바티 | 라바티 | CORRECT |
| P1A-N0227 | 투비도 | 투비도 | CORRECT |
| P1A-N0228 | 유노니 | 유노니 | CORRECT |

### image_0032.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_002.png`; SHA-256 `8270596254b1b035e7970e412a18ba63f9f4715f8ded1abc4cf26d20815ea38f`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/31` and `/reference_worlds/0`.

Facts: Day 53 birth Village 1, death 58 old age; day 51 birth Village 1, death 59 old age; day 49 birth Village 1. Prior top event clipped.
Names: 유노니 parents 비소키/고도사; 유루게 died; 니에바 parents 쿠로쿠/하아; 노비부 died; 가와 parents 비무오/이키무. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0143 | event | Day 53: birth (person #52); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0144 | event | Day 53: death (person #8); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/169 |
| P1A-C0145 | event | Day 51: birth (person #51); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/166 |
| P1A-C0146 | event | Day 51: death (person #14); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/164 |
| P1A-C0147 | event | Day 49: birth (person #50); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/161 |
| P1A-C0501 | visual | Prior top event clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0568 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0705 | named_relationship | 유노니 parents 비소키/고도사 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0706 | named_relationship | 유루게 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0707 | named_relationship | 니에바 parents 쿠로쿠/하아 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0708 | named_relationship | 노비부 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0709 | named_relationship | 가와 parents 비무오/이키무. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0229 | 유노니 | 유노니 | CORRECT |
| P1A-N0230 | 비소키 | 비소키 | CORRECT |
| P1A-N0231 | 고도사 | 고도사 | CORRECT |
| P1A-N0232 | 유루게 | 유루게 | CORRECT |
| P1A-N0233 | 니에바 | 니에바 | CORRECT |
| P1A-N0234 | 쿠로쿠 | 쿠로쿠 | CORRECT |
| P1A-N0235 | 하아 | 하아 | CORRECT |
| P1A-N0236 | 노비부 | 노비부 | CORRECT |
| P1A-N0237 | 가와 | 가와 | CORRECT |
| P1A-N0238 | 비무오 | 비무오 | CORRECT |
| P1A-N0239 | 이키무 | 이키무 | CORRECT |

### image_0033.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_003.png`; SHA-256 `04517b3fb615ca3ffd933f5c81a2485bfb2c01cec108d06f86fa46dc543bde25`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/32` and `/reference_worlds/0`.

Facts: Day 49 death 55 old age; day 47 partnership, Village 1 birth, death 57 old age; day 46 Village 1 birth.
Names: 보소부 died; 비무오/이키무 partners; 코후아 parents 부요비/토토수; 데와 died; 카타고 parents 타후쿠/유바마. Artifacts: top clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0148 | event | Day 49: death (person #4); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/159 |
| P1A-C0149 | event | Day 47: partnership (person #39) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/155 |
| P1A-C0150 | event | Day 47: birth (person #49); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/154 |
| P1A-C0151 | event | Day 47: death (person #12); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/152 |
| P1A-C0152 | event | Day 46: birth (person #48); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/151 |
| P1A-C0569 | artifact_observation | top clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0710 | named_relationship | 보소부 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0711 | named_relationship | 비무오/이키무 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0712 | named_relationship | 코후아 parents 부요비/토토수 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0713 | named_relationship | 데와 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0714 | named_relationship | 카타고 parents 타후쿠/유바마. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0240 | 보소부 | 보소부 | CORRECT |
| P1A-N0241 | 비무오 | 비무오 | CORRECT |
| P1A-N0242 | 이키무 | 이키무 | CORRECT |
| P1A-N0243 | 코후아 | 코후아 | CORRECT |
| P1A-N0244 | 부요비 | 부요비 | CORRECT |
| P1A-N0245 | 토토수 | 토토수 | CORRECT |
| P1A-N0246 | 데와 | 데와 | CORRECT |
| P1A-N0247 | 카타고 | 카타고 | CORRECT |
| P1A-N0248 | 타후쿠 | 타후쿠 | CORRECT |
| P1A-N0249 | 유바마 | 유바마 | CORRECT |

### image_0034.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_004.png`; SHA-256 `0058a5847a37fe04bf1feadc53338859454a93117d9184313d9abe735b2e7b85`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/33` and `/reference_worlds/0`.

Facts: Day 46 death 55 old age; day 45 two partnerships; day 44 partnership; day 41 partnership and Village 1 birth.
Names: 바티호 died; 비소키/고도사, 쿠로쿠/하아, 타후쿠/유바마, 부요비/토토수 partners; 비소키 parents 가카호/와노. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0153 | event | Day 46: death (person #2); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/149 |
| P1A-C0154 | event | Day 45: partnership (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/148 |
| P1A-C0155 | event | Day 45: partnership (person #43) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/147 |
| P1A-C0156 | event | Day 44: partnership (person #32) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/145 |
| P1A-C0157 | event | Day 41: partnership (person #35) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/143 |
| P1A-C0158 | event | Day 41: birth (person #47); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/140 |
| P1A-C0570 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0715 | named_relationship | 바티호 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0716 | named_relationship | 비소키/고도사, 쿠로쿠/하아, 타후쿠/유바마, 부요비/토토수 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0717 | named_relationship | 비소키 parents 가카호/와노. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0250 | 바티호 | 바티호 | CORRECT |
| P1A-N0251 | 비소키 | 비소키 | CORRECT |
| P1A-N0252 | 고도사 | 고도사 | CORRECT |
| P1A-N0253 | 쿠로쿠 | 쿠로쿠 | CORRECT |
| P1A-N0254 | 하아 | 하아 | CORRECT |
| P1A-N0255 | 타후쿠 | 타후쿠 | CORRECT |
| P1A-N0256 | 유바마 | 유바마 | CORRECT |
| P1A-N0257 | 부요비 | 부요비 | CORRECT |
| P1A-N0258 | 토토수 | 토토수 | CORRECT |
| P1A-N0259 | 비소키 | 비소키 | CORRECT |
| P1A-N0260 | 가카호 | 가카호 | CORRECT |
| P1A-N0261 | 와노 | 와노 | CORRECT |

### image_0035.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_005.png`; SHA-256 `37e3be07d70f92d4df0e0194c8f9fc6e301d02eec3c9dfcde3fff48dc8383d68`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/34` and `/reference_worlds/0`.

Facts: Day 41 death 56 old age; day 40 Village 1 birth and death 55 old age; day 37 Village 1 birth and death 58 old age. Prior top birth clipped.
Names: 후보 died; 고도사 parents 티미가/노비부; 바호 died; 라바티 parents 가카호/와노; 쿠미후 died. Artifacts: clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0159 | event | Day 41: death (person #9); age 56; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0160 | event | Day 40: birth (person #46); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/136 |
| P1A-C0161 | event | Day 40: death (person #1); age 55; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/135 |
| P1A-C0162 | event | Day 37: birth (person #45); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/132 |
| P1A-C0163 | event | Day 37: death (person #11); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/131 |
| P1A-C0502 | visual | Prior top birth clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0571 | artifact_observation | clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0718 | named_relationship | 후보 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0719 | named_relationship | 고도사 parents 티미가/노비부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0720 | named_relationship | 바호 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0721 | named_relationship | 라바티 parents 가카호/와노 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0722 | named_relationship | 쿠미후 died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0262 | 후보 | 후보 | CORRECT |
| P1A-N0263 | 고도사 | 고도사 | CORRECT |
| P1A-N0264 | 티미가 | 티미가 | CORRECT |
| P1A-N0265 | 노비부 | 노비부 | CORRECT |
| P1A-N0266 | 바호 | 바호 | CORRECT |
| P1A-N0267 | 라바티 | 라바티 | CORRECT |
| P1A-N0268 | 가카호 | 가카호 | CORRECT |
| P1A-N0269 | 와노 | 와노 | CORRECT |
| P1A-N0270 | 쿠미후 | 쿠미후 | CORRECT |

### image_0036.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_006.png`; SHA-256 `cc6649236b0771bb7e73206848765922841e44007845ec7ce1eb0b4fc82121c4`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/35` and `/reference_worlds/0`.

Facts: Day 37 death 58 old age, Village 1 birth, death 61 old age; day 36 partnership, Village 1 birth, death 59 old age.
Names: 쿠미후 died; 타누카 parents 유루게/보소부; 후에시 died; 유루게/보소부 partners; 쿠로쿠 parents 티미가/노비부; 티누유 died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0164 | event | Day 37: death (person #11); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/131 |
| P1A-C0165 | event | Day 37: birth (person #44); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/128 |
| P1A-C0166 | event | Day 37: death (person #18); age 61; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/127 |
| P1A-C0167 | event | Day 36: partnership (person #8) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/125 |
| P1A-C0168 | event | Day 36: birth (person #43); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/121 |
| P1A-C0169 | event | Day 36: death (person #5); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/119 |
| P1A-C0572 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0723 | named_relationship | 쿠미후 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0724 | named_relationship | 타누카 parents 유루게/보소부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0725 | named_relationship | 후에시 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0726 | named_relationship | 유루게/보소부 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0727 | named_relationship | 쿠로쿠 parents 티미가/노비부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0728 | named_relationship | 티누유 died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0271 | 쿠미후 | 쿠미후 | CORRECT |
| P1A-N0272 | 타누카 | 타누카 | CORRECT |
| P1A-N0273 | 유루게 | 유루게 | CORRECT |
| P1A-N0274 | 보소부 | 보소부 | CORRECT |
| P1A-N0275 | 후에시 | 후에시 | CORRECT |
| P1A-N0276 | 유루게 | 유루게 | CORRECT |
| P1A-N0277 | 보소부 | 보소부 | CORRECT |
| P1A-N0278 | 쿠로쿠 | 쿠로쿠 | CORRECT |
| P1A-N0279 | 티미가 | 티미가 | CORRECT |
| P1A-N0280 | 노비부 | 노비부 | CORRECT |
| P1A-N0281 | 티누유 | 티누유 | CORRECT |

### image_0037.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_007.png`; SHA-256 `3be94a1b78f22eb22d3e0debcd2ce47212f313da2dce7a61e2e1ccb1062f1495`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/36` and `/reference_worlds/0`.

Facts: Day 36 death 59 old age; day 35 Village 1 birth and death 58 old age; day 33 Village 1 birth; day 32 Village 1 birth and death 60 old age.
Names: 티누유 died; 하아 parents 투비도/바티호; 티에아 died; 아누누 parents 가카호/와노; 요리 parents 데와/이키무; 데호나 died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0170 | event | Day 36: death (person #5); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/119 |
| P1A-C0171 | event | Day 35: birth (person #42); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/117 |
| P1A-C0172 | event | Day 35: death (person #23); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/116 |
| P1A-C0173 | event | Day 33: birth (person #41); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/110 |
| P1A-C0174 | event | Day 32: birth (person #40); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/109 |
| P1A-C0175 | event | Day 32: death (person #3); age 60; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/107 |
| P1A-C0573 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0729 | named_relationship | 티누유 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0730 | named_relationship | 하아 parents 투비도/바티호 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0731 | named_relationship | 티에아 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0732 | named_relationship | 아누누 parents 가카호/와노 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0733 | named_relationship | 요리 parents 데와/이키무 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0734 | named_relationship | 데호나 died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0282 | 티누유 | 티누유 | CORRECT |
| P1A-N0283 | 하아 | 하아 | CORRECT |
| P1A-N0284 | 투비도 | 투비도 | CORRECT |
| P1A-N0285 | 바티호 | 바티호 | CORRECT |
| P1A-N0286 | 티에아 | 티에아 | CORRECT |
| P1A-N0287 | 아누누 | 아누누 | CORRECT |
| P1A-N0288 | 가카호 | 가카호 | CORRECT |
| P1A-N0289 | 와노 | 와노 | CORRECT |
| P1A-N0290 | 요리 | 요리 | CORRECT |
| P1A-N0291 | 데와 | 데와 | CORRECT |
| P1A-N0292 | 이키무 | 이키무 | CORRECT |
| P1A-N0293 | 데호나 | 데호나 | CORRECT |

### image_0038.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_008.png`; SHA-256 `54eb8e990582742bcc0d2a50b03458c1abf6cc25218eaada447b6d8271055101`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/37` and `/reference_worlds/0`.

Facts: Day 32 death 60 old age and Village 1 birth; day 31 Village 1 birth, deaths 59 and 62 old age, another Village 1 birth.
Names: 데호나 died; 비무오 parents 티미가/노비부; 소거루 parents 투비도/바티호; 타소에, 사히 died; 키유 parents 가카호/와노. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0176 | event | Day 32: death (person #3); age 60; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/107 |
| P1A-C0177 | event | Day 32: birth (person #39); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/106 |
| P1A-C0178 | event | Day 31: birth (person #38); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/104 |
| P1A-C0179 | event | Day 31: death (person #22); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/103 |
| P1A-C0180 | event | Day 31: death (person #17); age 62; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/101 |
| P1A-C0181 | event | Day 31: birth (person #37); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/100 |
| P1A-C0574 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0735 | named_relationship | 데호나 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0736 | named_relationship | 비무오 parents 티미가/노비부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0737 | named_relationship | 소거루 parents 투비도/바티호 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0738 | named_relationship | 타소에, 사히 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0739 | named_relationship | 키유 parents 가카호/와노. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0294 | 데호나 | 데호나 | CORRECT |
| P1A-N0295 | 비무오 | 비무오 | CORRECT |
| P1A-N0296 | 티미가 | 티미가 | CORRECT |
| P1A-N0297 | 노비부 | 노비부 | CORRECT |
| P1A-N0298 | 소거루 | 소가루 | INCORRECT |
| P1A-N0299 | 투비도 | 투비도 | CORRECT |
| P1A-N0300 | 바티호 | 바티호 | CORRECT |
| P1A-N0301 | 타소에 | 타소에 | CORRECT |
| P1A-N0302 | 사히 | 사히 | CORRECT |
| P1A-N0303 | 키유 | 키유 | CORRECT |
| P1A-N0304 | 가카호 | 가카호 | CORRECT |
| P1A-N0305 | 와노 | 와노 | CORRECT |

### image_0039.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_009.png`; SHA-256 `231aff459d02213f1a9ccd0545b085039c5b985ada53419e3b1cb92e816c5b61`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/38` and `/reference_worlds/0`.

Facts: Day 31 Village 1 birth and death 53 old age; day 30 Village 1 birth, partnership, two deaths 57 old age.
Names: 키유 parents 가카호/와노; 가와 died; 토토수 parents 데와/이키무; 데와/이키무 partners; 비리게, 에투바 died. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0182 | event | Day 31: birth (person #37); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/100 |
| P1A-C0183 | event | Day 31: death (person #24); age 53; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/98 |
| P1A-C0184 | event | Day 30: birth (person #36); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/97 |
| P1A-C0185 | event | Day 30: partnership (person #12) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/96 |
| P1A-C0186 | event | Day 30: death (person #16); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/95 |
| P1A-C0187 | event | Day 30: death (person #10); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/93 |
| P1A-C0575 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0740 | named_relationship | 키유 parents 가카호/와노 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0741 | named_relationship | 가와 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0742 | named_relationship | 토토수 parents 데와/이키무 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0743 | named_relationship | 데와/이키무 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0744 | named_relationship | 비리게, 에투바 died. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0306 | 키유 | 키유 | CORRECT |
| P1A-N0307 | 가카호 | 가카호 | CORRECT |
| P1A-N0308 | 와노 | 와노 | CORRECT |
| P1A-N0309 | 가와 | 가와 | CORRECT |
| P1A-N0310 | 토토수 | 토토수 | CORRECT |
| P1A-N0311 | 데와 | 데와 | CORRECT |
| P1A-N0312 | 이키무 | 이키무 | CORRECT |
| P1A-N0313 | 데와 | 데와 | CORRECT |
| P1A-N0314 | 이키무 | 이키무 | CORRECT |
| P1A-N0315 | 비리게 | 비리게 | CORRECT |
| P1A-N0316 | 에투바 | 에투바 | CORRECT |

### image_0040.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_010.png`; SHA-256 `d0f512ce6b3c51be39d0e7f31458c5ebccb235b56aa31a248b5b0358f017c1dd`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/39` and `/reference_worlds/0`.

Facts: Day 30 death 57 old age, Village 1 birth, death 58 old age; day 29 Village 1 birth and death 52 old age; day 28 partnership.
Names: 에투바 died; 부요비 parents 마비토/가와; 시키노 died; 가요 parents 티미가/노비부; 티미쿠 died; 가카호/와노 partners. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0188 | event | Day 30: death (person #10); age 57; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/93 |
| P1A-C0189 | event | Day 30: birth (person #35); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/92 |
| P1A-C0190 | event | Day 30: death (person #7); age 58; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/88 |
| P1A-C0191 | event | Day 29: birth (person #34); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/87 |
| P1A-C0192 | event | Day 29: death (person #13); age 52; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/85 |
| P1A-C0193 | event | Day 28: partnership (person #31) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/83 |
| P1A-C0576 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0745 | named_relationship | 에투바 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0746 | named_relationship | 부요비 parents 마비토/가와 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0747 | named_relationship | 시키노 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0748 | named_relationship | 가요 parents 티미가/노비부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0749 | named_relationship | 티미쿠 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0750 | named_relationship | 가카호/와노 partners. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0317 | 에투바 | 에투바 | CORRECT |
| P1A-N0318 | 부요비 | 부요비 | CORRECT |
| P1A-N0319 | 마비토 | 마비토 | CORRECT |
| P1A-N0320 | 가와 | 가와 | CORRECT |
| P1A-N0321 | 시키노 | 시키노 | CORRECT |
| P1A-N0322 | 가요 | 가요 | CORRECT |
| P1A-N0323 | 티미가 | 티미가 | CORRECT |
| P1A-N0324 | 노비부 | 노비부 | CORRECT |
| P1A-N0325 | 티미쿠 | 티미쿠 | CORRECT |
| P1A-N0326 | 가카호 | 가카호 | CORRECT |
| P1A-N0327 | 와노 | 와노 | CORRECT |

### image_0041.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/chronicle_ko_page_011.png`; SHA-256 `9fa9586c52b03195094bf1df829a333e1e2812a0bcf170697d4ceb40b9ca4957`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/live/reference-data.json` at `/images/40` and `/reference_worlds/0`.

Facts: Day 29 Village 1 birth and death 52 old age; day 28 partnership, Village 1 birth, death 59 old age; day 25 partnership.
Names: 가요 parents 티미가/노비부; 티미쿠 died; 가카호/와노 partners; 데요 parents 투비도/바티호; 미유나 died; 티미가/노비부 partners. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0194 | event | Day 29: birth (person #34); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/87 |
| P1A-C0195 | event | Day 29: death (person #13); age 52; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/85 |
| P1A-C0196 | event | Day 28: partnership (person #31) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/83 |
| P1A-C0197 | event | Day 28: birth (person #33); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/82 |
| P1A-C0198 | event | Day 28: death (person #6); age 59; old age | CORRECT | /reference_worlds/0/chronicle_pages/0/*/80 |
| P1A-C0199 | event | Day 25: partnership (person #26) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/78 |
| P1A-C0577 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0751 | named_relationship | 가요 parents 티미가/노비부 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0752 | named_relationship | 티미쿠 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0753 | named_relationship | 가카호/와노 partners | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0754 | named_relationship | 데요 parents 투비도/바티호 | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0755 | named_relationship | 미유나 died | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0756 | named_relationship | 티미가/노비부 partners. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0328 | 가요 | 가요 | CORRECT |
| P1A-N0329 | 티미가 | 티미가 | CORRECT |
| P1A-N0330 | 노비부 | 노비부 | CORRECT |
| P1A-N0331 | 티미쿠 | 티미쿠 | CORRECT |
| P1A-N0332 | 가카호 | 가카호 | CORRECT |
| P1A-N0333 | 와노 | 와노 | CORRECT |
| P1A-N0334 | 데요 | 데요 | CORRECT |
| P1A-N0335 | 투비도 | 투비도 | CORRECT |
| P1A-N0336 | 바티호 | 바티호 | CORRECT |
| P1A-N0337 | 미유나 | 미유나 | CORRECT |
| P1A-N0338 | 티미가 | 티미가 | CORRECT |
| P1A-N0339 | 노비부 | 노비부 | CORRECT |

### image_0042.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_en_page_000.png`; SHA-256 `47d2a8d870a34930c73505459964963322f523ae7d2ef7e2deb5ecf2a9d70f59`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/0` and `/reference_worlds/0`.

Facts: Day 85 10:00 daytime spring day 1, population 65, villages 2, shelter 10, construction 0, food 369, wood 12, cold 0, berries 54, births 88, deaths 45. #45 elder age 55, Village 2 from 1, position/target (3,62), resting by fire, carrying 0. Picture selected person's exact location cannot identify confidently from crowd; portrait seated gray-haired person.
Names: Rabati. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0301 | state | Selected #45 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0302 | state | Elder | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0303 | state | Age 55 days | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0304 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0305 | state | Position (3,62) | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0306 | state | Resting by fire | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0307 | state | Target place (3,62) | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0308 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0424 | hud | HUD day = 85 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0425 | hud | HUD time = 10:00 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0426 | hud | HUD daytime = True | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0427 | hud | HUD season = spring | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0428 | hud | HUD season_day = 1 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0429 | hud | HUD population = 65 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0430 | hud | HUD villages = 2 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0431 | hud | HUD shelter = 10 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0432 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0433 | hud | HUD food = 369 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0434 | hud | HUD wood = 12 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0435 | hud | HUD cold = 0 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0436 | hud | HUD berries = 54 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0437 | hud | HUD births = 88 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0438 | hud | HUD deaths = 45 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0503 | visual | Seated gray-haired portrait | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0530 | abstention | Cannot confidently identify exact selected map sprite | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0578 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0340 | Rabati | Rabati | CORRECT |

### image_0043.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_en_page_001.png`; SHA-256 `6c5a16489b38f8db1104f3eb1c3d2e02350e31c87bdf2e81da51261da4dbf3bc`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/1` and `/reference_worlds/0`.

Facts: Male, spouse present, mother/father deceased, three children, carrying 0 food. Temperament bars without numbers.
Names: Godosa spouse, Yubama mother, Gawa father, Yunoni visible child. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0309 | state | Male | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0310 | state | Spouse present | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0311 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0312 | state | Father deceased | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0313 | state | Children 3 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0314 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0504 | visual | Temperament bars without numeric labels | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0579 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0757 | named_relationship | Godosa spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0758 | named_relationship | Yubama mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0759 | named_relationship | Gawa father, Yunoni visible child. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0341 | Godosa | Godosa | CORRECT |
| P1A-N0342 | Yubama | Yubama | CORRECT |
| P1A-N0343 | Gawa | Gawa | CORRECT |
| P1A-N0344 | Yunoni | Yunoni | CORRECT |

### image_0044.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_en_page_002.png`; SHA-256 `5774c52b01f3814c707ec10c6239644117cdc5bad3ba6b99ec067af8ecbdca0c`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/2` and `/reference_worlds/0`.

Facts: Born day 30 Village 1; adult day 34; partnered day 36; child born days 39 and 56; day 60 leaving event partly clipped.
Names: Yunoni, Baga, Dawabi children listed; Yubama/Gawa parents; Godosa partner; Yunoni day39, Baga day56 births. Artifacts: bottom clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0200 | event | Day 30: birth (person #45); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/106 |
| P1A-C0201 | event | Day 34: came of age (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/118 |
| P1A-C0202 | event | Day 36: partnership (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/128 |
| P1A-C0203 | event | Day 39: birth (person #52) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0204 | event | Day 56: birth (person #64) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/208 |
| P1A-C0505 | visual | Day 60 leaving event partly clipped | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0580 | artifact_observation | bottom clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0760 | named_relationship | Yunoni, Baga, Dawabi children listed | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0761 | named_relationship | Yubama/Gawa parents | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0762 | named_relationship | Godosa partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0763 | named_relationship | Yunoni day39, Baga day56 births. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0345 | Yunoni | Yunoni | CORRECT |
| P1A-N0346 | Baga | Baga | CORRECT |
| P1A-N0347 | Dawabi | Dawabi | CORRECT |
| P1A-N0348 | Yubama | Yubama | CORRECT |
| P1A-N0349 | Gawa | Gawa | CORRECT |
| P1A-N0350 | Godosa | Godosa | CORRECT |
| P1A-N0351 | Yunoni | Yunoni | CORRECT |
| P1A-N0352 | Baga | Baga | CORRECT |

### image_0045.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_en_page_003.png`; SHA-256 `3d86012d5245f17ec7cd6f455685e64c03061063e87997d263808794c3b429c4`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/3` and `/reference_worlds/0`.

Facts: Children born days 39,56,62; day 60 left Village 1 in a named group, went with household; same day settled new Village 2 from Village 1; elder day 76.
Names: Yunoni day39; Baga day56; Rabati group; Dawabi day62. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0205 | event | Day 39: birth (person #52) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0206 | event | Day 56: birth (person #64) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/208 |
| P1A-C0207 | event | Day 60: departed (person #45); left Village 1, leader #45, followed household | CORRECT | /reference_worlds/0/chronicle_pages/0/*/212 |
| P1A-C0208 | event | Day 60: settled (person #45); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/231 |
| P1A-C0209 | event | Day 62: birth (person #73) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/254 |
| P1A-C0210 | event | Day 76: became elder (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/354 |
| P1A-C0581 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0764 | named_relationship | Rabati group | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0353 | Yunoni | Yunoni | CORRECT |
| P1A-N0354 | Baga | Baga | CORRECT |
| P1A-N0355 | Rabati | Rabati | CORRECT |
| P1A-N0356 | Dawabi | Dawabi | CORRECT |

### image_0046.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_ko_page_000.png`; SHA-256 `85be20e1bae8c87748912b58b252daf61a85cdc5c30c4d5b62c232fb52ce9bc6`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/4` and `/reference_worlds/0`.

Facts: Korean day85 HUD population65 villages2 shelter10 construction0 food369 wood12 cold0 berries54 births88 deaths45. #45 elder age55, Village2 from1, position/target(3,62), resting by fire, carrying0. Picture exact map identity uncertain, seated portrait.
Names: 라바티. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0315 | state | Selected #45 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0316 | state | Elder | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0317 | state | Age 55 days | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0318 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0319 | state | Position (3,62) | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0320 | state | Resting by fire | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0321 | state | Target place (3,62) | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0322 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0439 | hud | HUD day = 85 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0440 | hud | HUD population = 65 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0441 | hud | HUD villages = 2 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0442 | hud | HUD shelter = 10 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0443 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0444 | hud | HUD food = 369 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0445 | hud | HUD wood = 12 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0446 | hud | HUD cold = 0 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0447 | hud | HUD berries = 54 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0448 | hud | HUD births = 88 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0449 | hud | HUD deaths = 45 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0506 | visual | Seated portrait | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0531 | abstention | Exact map identity uncertain | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0582 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0800 | visual | Korean HUD | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0357 | 라바티 | 라바티 | CORRECT |

### image_0047.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_ko_page_001.png`; SHA-256 `5e2fd3293a8e974e67c8a7eb6e4254261d52df7e03779d155754c50f02d5009b`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/5` and `/reference_worlds/0`.

Facts: Male, spouse, deceased mother/father, children3. Four temperament bars.
Names: 고도사 spouse, 유바마 mother, 가와 father; 유노니, 바가, 다와비 children. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0323 | state | Male | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0324 | state | Spouse present | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0325 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0326 | state | Father deceased | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0327 | state | Children 3 | CORRECT | /reference_worlds/0/persons/45 |
| P1A-C0507 | visual | Four temperament bars | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0583 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0765 | named_relationship | 고도사 spouse, 유바마 mother, 가와 father | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0766 | named_relationship | 유노니, 바가, 다와비 children. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0358 | 고도사 | 고도사 | CORRECT |
| P1A-N0359 | 유바마 | 유바마 | CORRECT |
| P1A-N0360 | 가와 | 가와 | CORRECT |
| P1A-N0361 | 유노니 | 유노니 | CORRECT |
| P1A-N0362 | 바가 | 바가 | CORRECT |
| P1A-N0363 | 다와비 | 다와비 | CORRECT |

### image_0048.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_ko_page_002.png`; SHA-256 `3523c1245593f588cbe63f4849057c830d8ef6b41c7babcc3f1c43b8e5789cf0`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/6` and `/reference_worlds/0`.

Facts: Day30 born Village1; day34 adult; day36 partnered; births39,56; day60 left Village1, named leader, followed family; same day settled Village2 from1.
Names: 다와비 at top; 유바마/가와 parents; 고도사 partner; 유노니 day39, 바가 day56; 라바티 leader. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0211 | event | Day 30: birth (person #45); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/106 |
| P1A-C0212 | event | Day 34: came of age (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/118 |
| P1A-C0213 | event | Day 36: partnership (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/128 |
| P1A-C0214 | event | Day 39: birth (person #52) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0215 | event | Day 56: birth (person #64) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/208 |
| P1A-C0216 | event | Day 60: departed (person #45); left Village 1, leader #45, followed household | CORRECT | /reference_worlds/0/chronicle_pages/0/*/212 |
| P1A-C0217 | event | Day 60: settled (person #45); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/231 |
| P1A-C0584 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0767 | named_relationship | 유바마/가와 parents | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0768 | named_relationship | 고도사 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0769 | named_relationship | 라바티 leader. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0364 | 다와비 | 다와비 | CORRECT |
| P1A-N0365 | 유바마 | 유바마 | CORRECT |
| P1A-N0366 | 가와 | 가와 | CORRECT |
| P1A-N0367 | 고도사 | 고도사 | CORRECT |
| P1A-N0368 | 유노니 | 유노니 | CORRECT |
| P1A-N0369 | 바가 | 바가 | CORRECT |
| P1A-N0370 | 라바티 | 라바티 | CORRECT |

### image_0049.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_2_ko_page_003.png`; SHA-256 `d37eef76cf9e3b7134b2668121fab024e6dfe37d695f2be4344e392cfe1c2352`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/7` and `/reference_worlds/0`.

Facts: Adult34; partnered36; births39,56,62; day60 left Village1 led by named person and followed family; settled Village2 from1; elder76.
Names: 고도사 partner; 유노니, 바가, 다와비 children; 라바티 leader. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0218 | event | Day 34: came of age (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/118 |
| P1A-C0219 | event | Day 36: partnership (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/128 |
| P1A-C0220 | event | Day 39: birth (person #52) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/139 |
| P1A-C0221 | event | Day 56: birth (person #64) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/208 |
| P1A-C0222 | event | Day 60: departed (person #45); left Village 1, leader #45, followed household | CORRECT | /reference_worlds/0/chronicle_pages/0/*/212 |
| P1A-C0223 | event | Day 60: settled (person #45); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/231 |
| P1A-C0224 | event | Day 62: birth (person #73) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/254 |
| P1A-C0225 | event | Day 76: became elder (person #45) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/354 |
| P1A-C0585 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0770 | named_relationship | 고도사 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0771 | named_relationship | 유노니, 바가, 다와비 children | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0772 | named_relationship | 라바티 leader. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0371 | 고도사 | 고도사 | CORRECT |
| P1A-N0372 | 유노니 | 유노니 | CORRECT |
| P1A-N0373 | 바가 | 바가 | CORRECT |
| P1A-N0374 | 다와비 | 다와비 | CORRECT |
| P1A-N0375 | 라바티 | 라바티 | CORRECT |

### image_0050.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_en_page_000.png`; SHA-256 `e1ad9592b5fd522e026aa5083feb3b1d82b7d6384e561ee4995f85f3b1c12b20`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/8` and `/reference_worlds/0`.

Facts: #47 elder54 days, Village2 from1, position/target(6,63), sitting at hearth, carrying0 food. Day85 counts same visible as image42. Picture seated gray-haired side-facing portrait, exact map sprite uncertain in crowd.
Names: Bisoki. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0328 | state | Selected #47 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0329 | state | Elder | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0330 | state | Age 54 days | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0331 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0332 | state | Position (6,63) | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0333 | state | Sitting at hearth | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0334 | state | Target place (6,63) | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0335 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0450 | hud | HUD day = 85 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0451 | hud | HUD population = 65 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0452 | hud | HUD villages = 2 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0453 | hud | HUD shelter = 10 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0454 | hud | HUD construction = 0 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0455 | hud | HUD food = 369 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0456 | hud | HUD wood = 12 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0457 | hud | HUD cold = 0 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0458 | hud | HUD berries = 54 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0459 | hud | HUD births = 88 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0460 | hud | HUD deaths = 45 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0508 | visual | Seated gray-haired side-facing portrait | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0532 | abstention | Exact map sprite uncertain in crowd | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0586 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0376 | Bisoki | Bisoki | CORRECT |

### image_0051.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_en_page_001.png`; SHA-256 `bde023c52d2b8b77660637c3121531851cd679105cec891e2b96f72cfb77798b`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/9` and `/reference_worlds/0`.

Facts: Male, spouse, deceased mother/father, three children, carrying0.
Names: Gawa spouse, Deyo mother, Gakaho father, Geono child visible. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0336 | state | Male | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0337 | state | Spouse present | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0338 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0339 | state | Father deceased | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0340 | state | Children 3 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0341 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0587 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0773 | named_relationship | Gawa spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0774 | named_relationship | Deyo mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0775 | named_relationship | Gakaho father, Geono child visible. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0377 | Gawa | Gawa | CORRECT |
| P1A-N0378 | Deyo | Deyo | CORRECT |
| P1A-N0379 | Gakaho | Gakaho | CORRECT |
| P1A-N0380 | Geono | Geono | CORRECT |

### image_0052.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_en_page_002.png`; SHA-256 `e306bb66a33c7284a112f7f3325514ff678f32a29edb5cde8a67c396d1b30b38`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/10` and `/reference_worlds/0`.

Facts: Born31 Village1; adult35; partnered47; child48; day60 left Village1 in named group, among first to go.
Names: Geono, Hugade, Ahumo listed; Deyo/Gakaho parents; Gawa partner; Geono child48; Rabati group leader. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0226 | event | Day 31: birth (person #47); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/114 |
| P1A-C0227 | event | Day 35: came of age (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/120 |
| P1A-C0228 | event | Day 47: partnership (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/162 |
| P1A-C0229 | event | Day 48: birth (person #59) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0230 | event | Day 60: departed (person #47); left Village 1, leader #45, among first to leave | CORRECT | /reference_worlds/0/chronicle_pages/0/*/214 |
| P1A-C0588 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0776 | named_relationship | Deyo/Gakaho parents | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0777 | named_relationship | Gawa partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0778 | named_relationship | Rabati group leader. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0381 | Geono | Geono | CORRECT |
| P1A-N0382 | Hugade | Hugade | CORRECT |
| P1A-N0383 | Ahumo | Ahumo | CORRECT |
| P1A-N0384 | Deyo | Deyo | CORRECT |
| P1A-N0385 | Gakaho | Gakaho | CORRECT |
| P1A-N0386 | Gawa | Gawa | CORRECT |
| P1A-N0387 | Geono | Geono | CORRECT |
| P1A-N0388 | Rabati | Rabati | CORRECT |

### image_0053.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_en_page_003.png`; SHA-256 `7388cf807bc5b2b9dbe0ef6a93be8cc0cc44fbc7017a9eece8e24f893746a8ae`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/11` and `/reference_worlds/0`.

Facts: Child48; day60 left Village1 in named group among first to go, settled new Village2 from1; children61,64; elder77.
Names: Geono child48; Rabati group; Hugade child61, Ahumo child64. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0231 | event | Day 48: birth (person #59) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0232 | event | Day 60: departed (person #47); left Village 1, leader #45, among first to leave | CORRECT | /reference_worlds/0/chronicle_pages/0/*/214 |
| P1A-C0233 | event | Day 60: settled (person #47); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/233 |
| P1A-C0234 | event | Day 61: birth (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/249 |
| P1A-C0235 | event | Day 64: birth (person #81) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/273 |
| P1A-C0236 | event | Day 77: became elder (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/359 |
| P1A-C0589 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0779 | named_relationship | Rabati group | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0389 | Geono | Geono | CORRECT |
| P1A-N0390 | Rabati | Rabati | CORRECT |
| P1A-N0391 | Hugade | Hugade | CORRECT |
| P1A-N0392 | Ahumo | Ahumo | CORRECT |

### image_0054.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_ko_page_000.png`; SHA-256 `88e3bc9842cd41922e36ce1c26e0edbca5c1b25b9ea200fc7e39b11594787308`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/12` and `/reference_worlds/0`.

Facts: #47 elder54, Village2 from1, (6,63), sitting around hearth, target same place, food0. Picture seated side-facing gray-haired portrait; exact map person uncertain.
Names: 비소키. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0342 | state | Selected #47 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0343 | state | Elder | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0344 | state | Age 54 days | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0345 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0346 | state | Position (6,63) | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0347 | state | Sitting around hearth | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0348 | state | Target place (6,63) | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0349 | state | Carrying 0 food | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0509 | visual | Seated side-facing gray-haired portrait | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0533 | abstention | Exact map person uncertain | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0590 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0393 | 비소키 | 비소키 | CORRECT |

### image_0055.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_ko_page_001.png`; SHA-256 `66401fa7ae935e6c1bfebd24fa0ecc98001a573910aa1c577bcf0a22c54fe0a9`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/13` and `/reference_worlds/0`.

Facts: Male, spouse, deceased mother/father, children3.
Names: 가와 spouse; 데요 mother; 가카호 father; 게오노, 후가데, 아후모 children. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0350 | state | Male | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0351 | state | Spouse present | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0352 | state | Mother deceased | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0353 | state | Father deceased | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0354 | state | Children 3 | CORRECT | /reference_worlds/0/persons/47 |
| P1A-C0591 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0780 | named_relationship | 가와 spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0781 | named_relationship | 데요 mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0782 | named_relationship | 가카호 father | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0783 | named_relationship | 게오노, 후가데, 아후모 children. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0394 | 가와 | 가와 | CORRECT |
| P1A-N0395 | 데요 | 데요 | CORRECT |
| P1A-N0396 | 가카호 | 가카호 | CORRECT |
| P1A-N0397 | 게오노 | 게오노 | CORRECT |
| P1A-N0398 | 후가데 | 후가데 | CORRECT |
| P1A-N0399 | 아후모 | 아후모 | CORRECT |

### image_0056.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_ko_page_002.png`; SHA-256 `732e86819d2cfabd1116328990fe92f21abf57d3dec69fb644f51b47fd2a4b5d`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/14` and `/reference_worlds/0`.

Facts: Born31 Village1; adult35; partnership47; child48; day60 left Village1 led by named leader, chose to leave first; settled Village2 from1; birth61 clipped before name.
Names: 아후모 at top; 데요/가카호 parents; 가와 partner; 게오노 child48; 라바티 leader. Artifacts: bottom clipping only.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0237 | event | Day 31: birth (person #47); Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/114 |
| P1A-C0238 | event | Day 35: came of age (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/120 |
| P1A-C0239 | event | Day 47: partnership (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/162 |
| P1A-C0240 | event | Day 48: birth (person #59) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0241 | event | Day 60: departed (person #47); left Village 1, leader #45, among first to leave | CORRECT | /reference_worlds/0/chronicle_pages/0/*/214 |
| P1A-C0242 | event | Day 60: settled (person #47); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/233 |
| P1A-C0243 | event | Day 61: birth (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/249 |
| P1A-C0510 | visual | Day61 birth clipped before name | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0592 | artifact_observation | bottom clipping only. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0784 | named_relationship | 데요/가카호 parents | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0785 | named_relationship | 가와 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0786 | named_relationship | 라바티 leader. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0400 | 아후모 | 아후모 | CORRECT |
| P1A-N0401 | 데요 | 데요 | CORRECT |
| P1A-N0402 | 가카호 | 가카호 | CORRECT |
| P1A-N0403 | 가와 | 가와 | CORRECT |
| P1A-N0404 | 게오노 | 게오노 | CORRECT |
| P1A-N0405 | 라바티 | 라바티 | CORRECT |

### image_0057.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/depart_cause_1_ko_page_003.png`; SHA-256 `69b33ca33f046756a44e7380f9174c0d334a6403274659991bfdc9ed99100097`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/15` and `/reference_worlds/0`.

Facts: Partnership47; child48; day60 left Village1 led by leader, chose to leave first, settled Village2 from1; births61,64; elder77.
Names: 가와 partner; 게오노 child48; 라바티 leader; 후가데 child61; 아후모 child64. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0244 | event | Day 47: partnership (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/162 |
| P1A-C0245 | event | Day 48: birth (person #59) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/170 |
| P1A-C0246 | event | Day 60: departed (person #47); left Village 1, leader #45, among first to leave | CORRECT | /reference_worlds/0/chronicle_pages/0/*/214 |
| P1A-C0247 | event | Day 60: settled (person #47); new Village 2 from Village 1 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/233 |
| P1A-C0248 | event | Day 61: birth (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/249 |
| P1A-C0249 | event | Day 64: birth (person #81) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/273 |
| P1A-C0250 | event | Day 77: became elder (person #47) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/359 |
| P1A-C0593 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0787 | named_relationship | 가와 partner | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0788 | named_relationship | 라바티 leader | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0406 | 가와 | 가와 | CORRECT |
| P1A-N0407 | 게오노 | 게오노 | CORRECT |
| P1A-N0408 | 라바티 | 라바티 | CORRECT |
| P1A-N0409 | 후가데 | 후가데 | CORRECT |
| P1A-N0410 | 아후모 | 아후모 | CORRECT |

### image_0058.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_en_page_000.png`; SHA-256 `04f22f8a572ece820c419e27da0d702807c2d56a452f3ed6f7dfb2c3a4419fbb`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/16` and `/reference_worlds/0`.

Facts: #69 carries food ×10 to Village1; adult24, Village2 from1, position(21,48), on way, target Village1. Picture: selected person bottom center with yellow foot ring holding red food bundle, pale route line extends toward village. Activity visibly transport/walking (motion inferred from still frame), panel confirms delivery.
Names: Hugade. Artifacts: multiplication sign ×10 is meaningful quantity, no malformed percent/joiner/name split.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0355 | state | Selected #69 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0356 | state | Carrying food 10 to Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0357 | state | Adult | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0358 | state | Age 24 days | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0359 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0360 | state | Position (21,48) | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0361 | state | On the way | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0362 | state | Target Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0363 | state | Panel confirms delivery | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0511 | visual | Selected person bottom center with yellow foot ring and red food bundle | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0512 | visual | Pale route line extends toward village | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0535 | inference | Motion/walking inferred from still frame | NOT_SCORED_INFERENCE | PNG visible content |
| P1A-C0594 | artifact_observation | multiplication sign ×10 is meaningful quantity, no malformed percent/joiner/name split. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0411 | Hugade | Hugade | CORRECT |

### image_0059.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_en_page_001.png`; SHA-256 `32d2f521246a2af51531f39e9fdc43b676a241cdafd8aeb61fd82c77ff1ae56a`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/17` and `/reference_worlds/0`.

Facts: Carries10 food, female, spouse, mother/father listed alive without deceased marker, children0. Selected person still ringed with food bundle.
Names: Irayo spouse, Gawa mother, Bisoki father. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0364 | state | Carrying food 10 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0365 | state | Female | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0366 | state | Spouse present | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0367 | state | Mother alive | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0368 | state | Father alive | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0369 | state | Children 0 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0513 | visual | Selected person ringed with food bundle | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0595 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0789 | named_relationship | Irayo spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0790 | named_relationship | Gawa mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0791 | named_relationship | Bisoki father. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0412 | Irayo | Inayo | INCORRECT |
| P1A-N0413 | Gawa | Gawa | CORRECT |
| P1A-N0414 | Bisoki | Bisoki | CORRECT |

### image_0060.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_en_page_002.png`; SHA-256 `a46336a360ffc1c49b605817f0a9a819e8a5a8a2075dcf9d09e18eace0033c73`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/18` and `/reference_worlds/0`.

Facts: Female children0; born61 Village2; adult65; partnered68; day85 set out Village2 to Village1 with food×10 aid. Selected person carries bundle on visible route.
Names: Irayo spouse; Gawa/Bisoki parents. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0251 | event | Day 61: birth (person #69); Village 2 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/249 |
| P1A-C0252 | event | Day 65: came of age (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/283 |
| P1A-C0253 | event | Day 68: partnership (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/312 |
| P1A-C0370 | state | Female | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0371 | state | Children 0 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0481 | event | Day 85 departed Village 2 for Village 1 with food 10, aid | CORRECT | multi/reference-data.json#/reference_worlds/0/chronicle_pages/0/*/389 |
| P1A-C0514 | visual | Selected person carries bundle on visible route | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0596 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0792 | named_relationship | Irayo spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0793 | named_relationship | Gawa/Bisoki parents. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0415 | Irayo | Inayo | INCORRECT |
| P1A-N0416 | Gawa | Gawa | CORRECT |
| P1A-N0417 | Bisoki | Bisoki | CORRECT |

### image_0061.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_ko_page_000.png`; SHA-256 `b4d7ac293cf0217a2882f312e89c204ecd3de363a703aa1f35f13033fe062fec`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/19` and `/reference_worlds/0`.

Facts: #69 food10 to Village1, adult24, Village2 from1, position(21,48), moving, target Village1. Picture ringed person carrying food with route line.
Names: 후가데. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0372 | state | Selected #69 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0373 | state | Food 10 to Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0374 | state | Adult | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0375 | state | Age 24 days | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0376 | state | Village 2 with parent Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0377 | state | Position (21,48) | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0378 | state | Moving (panel state) | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0379 | state | Target Village 1 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0515 | visual | Ringed person carrying food with route line | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0597 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0418 | 후가데 | 후가데 | CORRECT |

### image_0062.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_ko_page_001.png`; SHA-256 `f64360a5d89323652a34de40e25f0867127dda5b7676964209c3dcdcac46b329`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/20` and `/reference_worlds/0`.

Facts: Carrying10 food; female, spouse, parents listed, children0. Picture ringed courier holding food.
Names: 이나요 spouse; 가와 mother; 비소키 father. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0380 | state | Carrying food 10 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0381 | state | Female | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0382 | state | Spouse present | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0383 | state | Mother listed | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0384 | state | Father listed | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0385 | state | Children 0 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0516 | visual | Ringed courier holding food | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0598 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0794 | named_relationship | 이나요 spouse | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0795 | named_relationship | 가와 mother | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |
| P1A-C0796 | named_relationship | 비소키 father. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0419 | 이나요 | 이나요 | CORRECT |
| P1A-N0420 | 가와 | 가와 | CORRECT |
| P1A-N0421 | 비소키 | 비소키 | CORRECT |

### image_0063.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_life_ko_page_002.png`; SHA-256 `919c0acc3dd9b6a0eef27ec1c784cf81674c0493266bf0f85d98567687dd069d`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/21` and `/reference_worlds/0`.

Facts: Female children0; born61 Village2; adult65; partnered68; day85 departed Village2 to Village1 carrying food10 aid. Picture ringed food carrier.
Names: 이나요 spouse, 가와/비소키 parents. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0254 | event | Day 61: birth (person #69); Village 2 | CORRECT | /reference_worlds/0/chronicle_pages/0/*/249 |
| P1A-C0255 | event | Day 65: came of age (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/283 |
| P1A-C0256 | event | Day 68: partnership (person #69) | CORRECT | /reference_worlds/0/chronicle_pages/0/*/312 |
| P1A-C0386 | state | Female | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0387 | state | Children 0 | CORRECT | /reference_worlds/0/persons/69 |
| P1A-C0482 | event | Day 85 departed Village 2 for Village 1 with food 10, aid | CORRECT | multi/reference-data.json#/reference_worlds/0/chronicle_pages/0/*/389 |
| P1A-C0517 | visual | Ringed food carrier | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0599 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0797 | named_relationship | 이나요 spouse, 가와/비소키 parents. | CORRECT | /reference_worlds/0/persons and exact chronicle indices retained in JSON |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|
| P1A-N0422 | 이나요 | 이나요 | CORRECT |
| P1A-N0423 | 가와 | 가와 | CORRECT |
| P1A-N0424 | 비소키 | 비소키 | CORRECT |

### image_0064.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_road_z4.png`; SHA-256 `b8281c8faa48743307505203a314396601208dcfbe7144995a853f0e78eac3b9`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/22` and `/reference_worlds/0`.

Facts: Korean HUD day85, population65 villages2 shelter10 food369 wood12 cold0 fruit54 births88 deaths45. Village1(41) label. Picture centered selected person with yellow ring and red bundle; thin line connects toward village, transport inferred. No personal details panel.
Names: none. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0461 | hud | HUD day = 85 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0462 | hud | HUD population = 65 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0463 | hud | HUD villages = 2 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0464 | hud | HUD shelter = 10 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0465 | hud | HUD food = 369 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0466 | hud | HUD wood = 12 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0467 | hud | HUD cold = 0 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0468 | hud | HUD berries = 54 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0469 | hud | HUD births = 88 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0470 | hud | HUD deaths = 45 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0518 | visual | Village1 label shows 41 | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0519 | visual | Centered selected person with yellow ring and red bundle | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0520 | visual | Thin line connects toward village | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0521 | visual | No personal details panel | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0536 | inference | Transport inferred from bundle and line | NOT_SCORED_INFERENCE | PNG visible content |
| P1A-C0600 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0801 | visual | Korean HUD | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|

### image_0065.png

Source: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/porter_road_z1.png`; SHA-256 `615c61d098a91d70c7a4aa920bc97d2bf37d7e948b4816b98e85447704c6e652`.
Reference: `/Users/rexxa/work/art-production13b-evidence/life-attempt1/multi/reference-data.json` at `/images/23` and `/reference_worlds/0`.

Facts: Zoomed-out map Village1(41), Village2 from1(24), day85 counts as prior. Picture two settlement clusters and connecting green line; selected tiny ringed person below first village, activity unreadable at this scale alone.
Names: none. Artifacts: none observed.

| Claim ID | Category | Assertion | Verdict | Reference |
|---|---|---|---|---|
| P1A-C0471 | hud | HUD day = 85 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0472 | hud | HUD population = 65 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0473 | hud | HUD villages = 2 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0474 | hud | HUD shelter = 10 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0475 | hud | HUD food = 369 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0476 | hud | HUD wood = 12 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0477 | hud | HUD cold = 0 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0478 | hud | HUD berries = 54 | CORRECT_VISUAL_ONLY | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0479 | hud | HUD births = 88 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0480 | hud | HUD deaths = 45 | CORRECT | multi/reference-data.json#/reference_worlds/0 |
| P1A-C0522 | visual | Village1 label shows41 | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0523 | visual | Village2 from1 label shows24 | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0524 | visual | Two settlement clusters | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0525 | visual | Connecting green line | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0526 | visual | Tiny ringed person below first village | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0534 | abstention | Activity unreadable at this scale alone | NOT_SCORED_ABSTENTION | PNG visible content |
| P1A-C0601 | artifact_observation | none observed. | CORRECT_VISUAL_ONLY | PNG visible content |
| P1A-C0802 | visual | Zoomed-out map | CORRECT_VISUAL_ONLY | PNG visible content |

| Name ID | Reader | Expected | Verdict |
|---|---|---|---|

## Verification

Frozen answer hash rechecked after scoring. Every65 original PNG hashes to the private-map and reference-data image entry. Exact event fields, person records, roles, source hashes and every claim/name occurrence are retained in `part1a-score.json`. No Godot, commits, source changes, blind-reader contact, or reader-answer edits were performed.
