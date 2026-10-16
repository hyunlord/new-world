# Pass 17 Stage B — FAIL

Mode: evaluation.

| Rule | Verdict |
|---|---|
| F1 | FAIL |
| F2 | PASS |
| F3 | PASS |
| F4 | PASS |
| F5 | PASS |
| F6 | PASS |
| F7 | PASS |
| F8 | PASS |

| Metric | Candidate | Baseline | Estimate | 95% CI | Valid bootstrap draws | Bar | Verdict |
|---|---:|---:|---:|---|---:|---|---|
| F1 | undefined | undefined | 0.495301 | [0.454451, 0.532409] | 2000/2000 | pooled living mutually partnered adult share at elapsed day60 >= 0.50 | FAIL |
| F2 | undefined | undefined | 0.967101 | [0.932075, 0.99375] | 2000/2000 | couple-parent births from elapsed simulation day20 inclusive >= 0.90 | PASS |
| F3 | undefined | undefined | 0.108624 | [0.0862283, 0.130078] | 2000/2000 | mean four-axis sibling Pearson minus unrelated Pearson >= 0.06; lower CI > 0 | PASS |
| F3-siblings | undefined | undefined | 0.12118 | [0.0969236, 0.142339] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-ha | undefined | undefined | 0.112353 | [0.0771057, 0.144321] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-ns | undefined | undefined | 0.14423 | [0.108634, 0.179695] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-p | undefined | undefined | 0.102016 | [0.0579897, 0.148218] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-rd | undefined | undefined | 0.126121 | [0.0853351, 0.165427] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated | undefined | undefined | 0.0125563 | [0.00511908, 0.0179032] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-ha | undefined | undefined | 0.00012083 | [-0.012945, 0.0144567] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-ns | undefined | undefined | 0.0231976 | [0.00388838, 0.0414886] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-p | undefined | undefined | 0.00779574 | [-0.00700439, 0.0198295] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-rd | undefined | undefined | 0.0191112 | [0.0030208, 0.0324138] | 2000/2000 | descriptive reference | PASS |
| F4 | undefined | undefined | 0.996182 | [0.993906, 0.998185] | 2000/2000 | same-shelter share of living mutual bedded couple-nights at20:00 >= 0.90 | PASS |
| F5 | undefined | undefined | 0.0769231 | [0.0389592, 0.122581] | 2000/2000 | split couples / parent couples at completed fission departures <= 0.10 | PASS |
| F6-alive | 57.2188 | 56.8438 | 0.00659703 | [-0.0101409, 0.0308809] | 2000/2000 | relative change within +/-0.10 | PASS |
| F6-deaths | 1.3125 | 1.53125 | -0.21875 | [-0.96875, 0.5] | 2000/2000 | non-old-age deaths difference <= +1.5/world | PASS |
| F6-food | 465.167 | 462.688 | 0.00535819 | [-0.00309528, 0.0145654] | 2000/2000 | noon food relative change within +/-0.15 | PASS |
| F6-shelters | 6.125 | 6.0625 | 1.01031 | [0.98907, 1.04] | 2000/2000 | completed shelters ratio >= 0.90 | PASS |
| F6-social | 1.75939 | 1.72919 | 1.01746 | [1.00624, 1.02833] | 2000/2000 | pooled adult social rate ratio >= 0.90 | PASS |
| F6-wood | 115.629 | 114.286 | 0.0117441 | [-0.00276679, 0.0315035] | 2000/2000 | noon wood relative change within +/-0.15 | PASS |

## Methods

- bootstrap: 2,000 whole-world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded with valid count; F6 keeps seed pairs together
- F1: Pooled living nonelder adults at exact elapsed day60 (after tick86399); partner must be alive and mutual; dead partner pointers do not count
- F2: Pooled births with fallback=false from one-based elapsed simulation day20; simulation_day=tick//1440+1, tick>=27360; display-clock offset does not change inclusion; all daily births/fallbacks retained
- F3: Pooled Pearson for each temperament axis, then mean four axes for siblings minus unrelated; each unique unordered pair contributes both orientations; bar>=0.06 and lower95%CI>0; exact per-world integer sufficient statistics preserve whole-world bootstrap
- F3_eligibility: Full siblings have both known matching mother/father, including fallback births; first shared hourly snapshot fixes sibling life-stage pair. Unrelated controls exclude direct parent-child or any shared known parent across either slot, and use first shared hourly snapshot with a stage-pair category present among that world's eligible siblings. Unassigned settlement=null observations cannot form same-settlement pairs; their counts are reported. Unknown founder kinship is not inferred. Empty worlds remain in resampling; pair IDs/ticks/stages reported
- F4: 20:00 snapshots; each mutual living couple counted once if both have beds; same bed tile means shared shelter; bedless person-nights separately reported
- F5: Only completed fissions; pooled split couples / mutual living couples in parent settlements at departure; raw membership and household partitions verify numerator; zero denominator ERROR
- F6: Identical R2 definitions: equal-world alive/death/shelter means, noon settlement-summed stores averaged within worlds then across worlds; primary social rate pools adult completions/adult-days, mean world rate sensitivity
- shelters: Final minus initial complete Shelter count, including daughter-camp boot shelters
- errors: Missing/malformed observations or proof ERROR; undefined fractions/correlations ERROR, never artificial zero; measured bar misses FAIL with numeric estimates; absent F7/F8 tuning-only PENDING
- evidence: F7 schema1 stageb requires I1-I5 plus v19/v20 pins and verifies every file hash; F8 schema1 stageb re-verifies complete raw performance evidence with unchanged timing bars

## Eligibility and sensitivity

```json
{
  "F1": {
    "numerator": 527.0,
    "denominator": 1064.0,
    "worlds": 32,
    "zero_denominator_worlds": 0
  },
  "F2": {
    "numerator": 2969.0,
    "denominator": 3070.0,
    "worlds": 32,
    "zero_denominator_worlds": 0
  },
  "F3": {
    "sibling_unordered_pairs": 6783,
    "unrelated_unordered_pairs": 78583,
    "sibling_worlds": 32,
    "unrelated_worlds": 32,
    "zero_sibling_worlds": 0,
    "zero_unrelated_worlds": 0,
    "excluded_close_kin_pairs": 7695,
    "orientations_per_pair": 2
  },
  "F4": {
    "numerator": 38875.0,
    "denominator": 39024.0,
    "worlds": 32,
    "zero_denominator_worlds": 0
  },
  "F5": {
    "numerator": 18.0,
    "denominator": 234.0,
    "worlds": 32,
    "zero_denominator_worlds": 14
  },
  "F6-shelters": {
    "definition": "final minus initial complete Shelter count, including daughter-camp boot shelters"
  },
  "F6-social": {
    "candidate_mean_world_rate": 1.7319725672232409,
    "baseline_mean_world_rate": 1.7048127222704061,
    "candidate_sensitivity_worlds": 32,
    "baseline_sensitivity_worlds": 32
  }
}
```

## game_v21

32 worlds × 120 days; seeds [18000, 18001, 18002, 18003, 18004, 18005, 18006, 18007, 18008, 18009, 18010, 18011, 18012, 18013, 18014, 18015, 18016, 18017, 18018, 18019, 18020, 18021, 18022, 18023, 18024, 18025, 18026, 18027, 18028, 18029, 18030, 18031].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 129640 | 0 | 0.062203 | 0 | 0.937797 | 0 |
| 0 | child | 13723 | 0 | 0.00371639 | 0 | 0.996284 | 0 |
| 0 | elder | 31747 | 0 | 0.00790626 | 0 | 0.992094 | 0 |
| 60 | adult | 129681 | 1.54225e-05 | 0.0295649 | 0 | 0.97042 | 0 |
| 60 | child | 13746 | 0 | 0.000509239 | 0 | 0.999491 | 0 |
| 60 | elder | 31759 | 0 | 0.00365251 | 0 | 0.996347 | 0 |
| 120 | adult | 129713 | 0.00011564 | 0.0178856 | 0 | 0.981999 | 0 |
| 120 | child | 13745 | 0 | 0 | 0 | 1 | 0 |
| 120 | elder | 31715 | 0.000157654 | 0.00681066 | 0 | 0.993032 | 0 |
| 180 | adult | 129725 | 0.000616689 | 0.0894893 | 0 | 0.909894 | 0 |
| 180 | child | 13748 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 31686 | 0.000504955 | 0.0980244 | 0 | 0.901471 | 0 |
| 240 | adult | 129714 | 0.000547358 | 0.117458 | 0 | 0.881994 | 0 |
| 240 | child | 13745 | 0 | 0.000145507 | 0 | 0.999854 | 0 |
| 240 | elder | 31634 | 0.000505785 | 0.127426 | 0 | 0.872068 | 0 |
| 300 | adult | 129691 | 0.00383219 | 0.167606 | 0 | 0.828562 | 0 |
| 300 | child | 13740 | 0.00014556 | 0.00123726 | 0 | 0.998617 | 0 |
| 300 | elder | 31583 | 0.00446443 | 0.180034 | 0 | 0.815502 | 0 |
| 360 | adult | 129691 | 0.797133 | 0.154698 | 0 | 0.0481683 | 0 |
| 360 | child | 13746 | 0.975629 | 0.0176779 | 0 | 0.00669286 | 0 |
| 360 | elder | 31554 | 0.791786 | 0.158237 | 0 | 0.0499778 | 0 |
| 420 | adult | 129751 | 0.156376 | 0.832325 | 0.00901727 | 0.000246626 | 0.00203467 |
| 420 | child | 13755 | 0.814104 | 0.171792 | 0 | 0.000218103 | 0.0138859 |
| 420 | elder | 31590 | 0.505445 | 0.467173 | 0.0182653 | 0.000443178 | 0.00867363 |
| 480 | adult | 129309 | 0.0593385 | 0.630791 | 0.00437711 | 0.0373524 | 0.268141 |
| 480 | child | 13585 | 0.00993743 | 0.0357748 | 0 | 0.0195804 | 0.934707 |
| 480 | elder | 31318 | 0.00842966 | 0.328405 | 0.00421483 | 0.0200843 | 0.638866 |
| 540 | adult | 129347 | 0.0961909 | 0.568339 | 0.00533449 | 0.000819501 | 0.329316 |
| 540 | child | 13596 | 0.117314 | 0.0369226 | 0 | 0 | 0.845763 |
| 540 | elder | 31573 | 0.111171 | 0.279353 | 0.00240712 | 0.000158363 | 0.606911 |
| 600 | adult | 129357 | 0.103357 | 0.590946 | 0.0017471 | 0.0299945 | 0.273955 |
| 600 | child | 13611 | 0.348395 | 0.0368819 | 0 | 0.0101389 | 0.604585 |
| 600 | elder | 31587 | 0.214994 | 0.2363 | 0.00189952 | 0.0168424 | 0.529965 |
| 660 | adult | 129385 | 0.134451 | 0.476725 | 0.0051397 | 0.000533292 | 0.383151 |
| 660 | child | 13617 | 0.174708 | 0.0436219 | 0 | 0 | 0.78167 |
| 660 | elder | 31558 | 0.152354 | 0.0960137 | 0.0015527 | 0.000158438 | 0.749921 |
| 720 | adult | 129398 | 0.152012 | 0.281473 | 0.00120558 | 0.0413453 | 0.523965 |
| 720 | child | 13623 | 0.252881 | 0.0399325 | 0 | 0.0109374 | 0.696249 |
| 720 | elder | 31534 | 0.189922 | 0.0970064 | 0.00120505 | 0.0188685 | 0.692998 |
| 780 | adult | 129435 | 0.151744 | 0.333642 | 0.00372388 | 0.000185421 | 0.510704 |
| 780 | child | 13644 | 0.251173 | 0.0417033 | 0 | 0 | 0.707124 |
| 780 | elder | 31545 | 0.203994 | 0.0895229 | 0.000982723 | 0.000126803 | 0.705373 |
| 840 | adult | 129465 | 0.126552 | 0.219727 | 0.00121268 | 0.0302939 | 0.622214 |
| 840 | child | 13648 | 0.228458 | 0.0431565 | 0 | 0.0103312 | 0.718054 |
| 840 | elder | 31509 | 0.190834 | 0.0891174 | 0.000761687 | 0.01955 | 0.699737 |
| 900 | adult | 129477 | 0.126733 | 0.320188 | 0.00352958 | 0.000177638 | 0.549372 |
| 900 | child | 13655 | 0.231783 | 0.0585134 | 0 | 7.32332e-05 | 0.70963 |
| 900 | elder | 31480 | 0.201302 | 0.0839898 | 0.000984752 | 0.000127065 | 0.713596 |
| 960 | adult | 129476 | 0.118532 | 0.223238 | 0.000656492 | 0.0306775 | 0.626896 |
| 960 | child | 13658 | 0.236199 | 0.0462732 | 0 | 0.0112754 | 0.706253 |
| 960 | elder | 31463 | 0.19337 | 0.0913454 | 0.000540317 | 0.018339 | 0.696405 |
| 1020 | adult | 129480 | 0.11462 | 0.313531 | 0.00339821 | 0.000115848 | 0.568335 |
| 1020 | child | 13661 | 0.239953 | 0.0450187 | 0 | 0 | 0.715028 |
| 1020 | elder | 31424 | 0.198829 | 0.095373 | 0.000986507 | 0.000127291 | 0.704684 |
| 1080 | adult | 129489 | 0.744658 | 0.225031 | 0.000679594 | 0.0296319 | 0 |
| 1080 | child | 13665 | 0.941017 | 0.0455177 | 0 | 0.0134651 | 0 |
| 1080 | elder | 31386 | 0.885936 | 0.0937998 | 0.000446059 | 0.0198178 | 0 |
| 1140 | adult | 129533 | 0.593069 | 0.398732 | 0.00812148 | 7.72004e-05 | 0 |
| 1140 | child | 13671 | 0.959988 | 0.0399386 | 0 | 7.31475e-05 | 0 |
| 1140 | elder | 31401 | 0.906054 | 0.0915894 | 0.00229292 | 6.36922e-05 | 0 |
| 1200 | adult | 129572 | 0.709806 | 0.255271 | 0.00236934 | 0.0325533 | 0 |
| 1200 | child | 13679 | 0.9489 | 0.0406462 | 0 | 0.010454 | 0 |
| 1200 | elder | 31547 | 0.88487 | 0.0939234 | 0.00187023 | 0.0193362 | 0 |
| 1260 | adult | 129609 | 0.349389 | 0.568479 | 0.00156625 | 0.0805654 | 0 |
| 1260 | child | 13693 | 0.48156 | 0.331118 | 0 | 0.187322 | 0 |
| 1260 | elder | 31790 | 0.475307 | 0.389368 | 0.000471846 | 0.134854 | 0 |
| 1320 | adult | 129600 | 0 | 0.359344 | 0 | 0.640656 | 0 |
| 1320 | child | 13709 | 0 | 0.0533226 | 0 | 0.946677 | 0 |
| 1320 | elder | 31800 | 0 | 0.103679 | 0 | 0.896321 | 0 |
| 1380 | adult | 129618 | 1.543e-05 | 0.162763 | 0 | 0.837222 | 0 |
| 1380 | child | 13719 | 0 | 0.0139223 | 0 | 0.986078 | 0 |
| 1380 | elder | 31776 | 0 | 0.0278827 | 0 | 0.972117 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 4446114,
    "adult_minutes": 4443605,
    "all_life_stages_share": 0.1361832704152739,
    "adult_share": 0.2546777117821379,
    "all_life_stages_starts": 141933
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 2936079,
    "adult_minutes": 2933965,
    "all_life_stages_share": 0.08993130639871288,
    "adult_share": 0.16815524616811806,
    "all_life_stages_starts": 89979
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 3583334,
    "adult_minutes": 3580997,
    "all_life_stages_share": 0.10975655215098962,
    "adult_share": 0.20523879189502678,
    "all_life_stages_starts": 112713
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 9986733,
    "adult_minutes": 4967502,
    "all_life_stages_share": 0.3058909332293638,
    "adult_share": 0.2847039830572685,
    "all_life_stages_starts": 250106
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 1495591,
    "adult_minutes": 1494025,
    "all_life_stages_share": 0.04580954819954007,
    "adult_share": 0.08562751827520866,
    "all_life_stages_starts": 94818
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 4990752,
    "adult_minutes": 27860,
    "all_life_stages_share": 0.15286538518615786,
    "adult_share": 0.001596748822240132,
    "all_life_stages_starts": 68160
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 5209416,
    "adult_minutes": 0,
    "all_life_stages_share": 0.1595630044199619,
    "adult_share": 0.0,
    "all_life_stages_starts": 93711
  }
]
```

### health_by_world

```json
[
  {
    "alive": 77.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 676.4166666666666,
    "wood": 166.55,
    "seed": 18000,
    "old_age_deaths": 107,
    "social_completed": 10415.0,
    "adult_days": 5932.75,
    "social_rate": 1.7555096708946105,
    "no_place_redraws": 2525,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 330.85833333333335,
    "wood": 83.51666666666667,
    "seed": 18001,
    "old_age_deaths": 66,
    "social_completed": 4906.0,
    "adult_days": 2930.3333333333335,
    "social_rate": 1.6742122625412352,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 344.8666666666667,
    "wood": 91.78333333333333,
    "seed": 18002,
    "old_age_deaths": 62,
    "social_completed": 4729.0,
    "adult_days": 2796.25,
    "social_rate": 1.6911935628073314,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 31.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 315.025,
    "wood": 94.15,
    "seed": 18003,
    "old_age_deaths": 58,
    "social_completed": 4716.0,
    "adult_days": 2712.875,
    "social_rate": 1.738377182877943,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 58.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 562.0916666666667,
    "wood": 151.325,
    "seed": 18004,
    "old_age_deaths": 89,
    "social_completed": 7294.0,
    "adult_days": 4493.625,
    "social_rate": 1.6231884057971016,
    "no_place_redraws": 2533,
    "no_place_failures": 0
  },
  {
    "alive": 26.0,
    "deaths": 6.0,
    "shelters": 1.0,
    "food": 265.71666666666664,
    "wood": 76.25,
    "seed": 18005,
    "old_age_deaths": 52,
    "social_completed": 3567.0,
    "adult_days": 2231.9166666666665,
    "social_rate": 1.5981779486988017,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 2.0,
    "shelters": 2.0,
    "food": 280.725,
    "wood": 87.48333333333333,
    "seed": 18006,
    "old_age_deaths": 64,
    "social_completed": 4932.0,
    "adult_days": 2785.0,
    "social_rate": 1.770915619389587,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 87.0,
    "deaths": 0.0,
    "shelters": 10.0,
    "food": 741.9833333333333,
    "wood": 149.7,
    "seed": 18007,
    "old_age_deaths": 124,
    "social_completed": 12178.0,
    "adult_days": 6378.416666666667,
    "social_rate": 1.9092512509635358,
    "no_place_redraws": 3353,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 0.0,
    "shelters": 6.0,
    "food": 535.8333333333334,
    "wood": 148.23333333333332,
    "seed": 18008,
    "old_age_deaths": 90,
    "social_completed": 8215.0,
    "adult_days": 4607.75,
    "social_rate": 1.7828658238836743,
    "no_place_redraws": 2404,
    "no_place_failures": 0
  },
  {
    "alive": 67.0,
    "deaths": 1.0,
    "shelters": 7.0,
    "food": 368.18333333333334,
    "wood": 94.23333333333333,
    "seed": 18009,
    "old_age_deaths": 61,
    "social_completed": 5707.0,
    "adult_days": 3536.0,
    "social_rate": 1.6139705882352942,
    "no_place_redraws": 959,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 195.09166666666667,
    "wood": 79.925,
    "seed": 18010,
    "old_age_deaths": 45,
    "social_completed": 3183.0,
    "adult_days": 2014.5833333333333,
    "social_rate": 1.5799793174767323,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 277.375,
    "wood": 90.45833333333333,
    "seed": 18011,
    "old_age_deaths": 64,
    "social_completed": 4956.0,
    "adult_days": 2968.25,
    "social_rate": 1.6696706813779163,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 79.0,
    "deaths": 18.0,
    "shelters": 9.0,
    "food": 598.65,
    "wood": 112.5,
    "seed": 18012,
    "old_age_deaths": 76,
    "social_completed": 9495.0,
    "adult_days": 5469.833333333333,
    "social_rate": 1.735884700935434,
    "no_place_redraws": 1659,
    "no_place_failures": 0
  },
  {
    "alive": 29.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 299.3833333333333,
    "wood": 77.8,
    "seed": 18013,
    "old_age_deaths": 55,
    "social_completed": 4208.0,
    "adult_days": 2504.1666666666665,
    "social_rate": 1.6803993344425958,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 93.0,
    "deaths": 0.0,
    "shelters": 12.0,
    "food": 684.1833333333333,
    "wood": 179.75833333333333,
    "seed": 18014,
    "old_age_deaths": 95,
    "social_completed": 10631.0,
    "adult_days": 5676.625,
    "social_rate": 1.8727677096866535,
    "no_place_redraws": 3049,
    "no_place_failures": 0
  },
  {
    "alive": 35.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 367.525,
    "wood": 86.74166666666666,
    "seed": 18015,
    "old_age_deaths": 68,
    "social_completed": 5212.0,
    "adult_days": 3051.5833333333335,
    "social_rate": 1.7079658100985826,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 73.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 506.325,
    "wood": 131.11666666666667,
    "seed": 18016,
    "old_age_deaths": 68,
    "social_completed": 7857.0,
    "adult_days": 4619.625,
    "social_rate": 1.7007874015748032,
    "no_place_redraws": 1814,
    "no_place_failures": 0
  },
  {
    "alive": 81.0,
    "deaths": 1.0,
    "shelters": 9.0,
    "food": 703.7416666666667,
    "wood": 145.95,
    "seed": 18017,
    "old_age_deaths": 115,
    "social_completed": 11867.0,
    "adult_days": 6360.333333333333,
    "social_rate": 1.865782715790577,
    "no_place_redraws": 2623,
    "no_place_failures": 0
  },
  {
    "alive": 122.0,
    "deaths": 1.0,
    "shelters": 16.0,
    "food": 831.4666666666667,
    "wood": 185.05,
    "seed": 18018,
    "old_age_deaths": 116,
    "social_completed": 13181.0,
    "adult_days": 7499.958333333333,
    "social_rate": 1.7574764304246135,
    "no_place_redraws": 6195,
    "no_place_failures": 9
  },
  {
    "alive": 89.0,
    "deaths": 0.0,
    "shelters": 13.0,
    "food": 649.0,
    "wood": 130.76666666666668,
    "seed": 18019,
    "old_age_deaths": 94,
    "social_completed": 9988.0,
    "adult_days": 5633.416666666667,
    "social_rate": 1.7729915237940266,
    "no_place_redraws": 1970,
    "no_place_failures": 0
  },
  {
    "alive": 24.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 255.7,
    "wood": 75.975,
    "seed": 18020,
    "old_age_deaths": 48,
    "social_completed": 3299.0,
    "adult_days": 2129.5,
    "social_rate": 1.5491899506926508,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 66.0,
    "deaths": 2.0,
    "shelters": 8.0,
    "food": 520.7583333333333,
    "wood": 132.475,
    "seed": 18021,
    "old_age_deaths": 75,
    "social_completed": 7620.0,
    "adult_days": 4463.458333333333,
    "social_rate": 1.7071964003995408,
    "no_place_redraws": 2119,
    "no_place_failures": 0
  },
  {
    "alive": 70.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 572.4916666666667,
    "wood": 130.24166666666667,
    "seed": 18022,
    "old_age_deaths": 75,
    "social_completed": 8139.0,
    "adult_days": 4783.416666666667,
    "social_rate": 1.7015034581279072,
    "no_place_redraws": 2433,
    "no_place_failures": 0
  },
  {
    "alive": 97.0,
    "deaths": 8.0,
    "shelters": 12.0,
    "food": 723.3666666666667,
    "wood": 171.95833333333334,
    "seed": 18023,
    "old_age_deaths": 110,
    "social_completed": 12205.0,
    "adult_days": 6914.666666666667,
    "social_rate": 1.7650887003470883,
    "no_place_redraws": 3328,
    "no_place_failures": 0
  },
  {
    "alive": 90.0,
    "deaths": 1.0,
    "shelters": 13.0,
    "food": 661.0166666666667,
    "wood": 137.73333333333332,
    "seed": 18024,
    "old_age_deaths": 101,
    "social_completed": 9252.0,
    "adult_days": 5469.041666666667,
    "social_rate": 1.6917040614976724,
    "no_place_redraws": 3539,
    "no_place_failures": 0
  },
  {
    "alive": 24.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 263.09166666666664,
    "wood": 78.0,
    "seed": 18025,
    "old_age_deaths": 48,
    "social_completed": 3397.0,
    "adult_days": 2088.6666666666665,
    "social_rate": 1.626396425151612,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 92.0,
    "deaths": 1.0,
    "shelters": 12.0,
    "food": 547.1416666666667,
    "wood": 143.25833333333333,
    "seed": 18026,
    "old_age_deaths": 77,
    "social_completed": 8302.0,
    "adult_days": 4747.416666666667,
    "social_rate": 1.7487405431023888,
    "no_place_redraws": 2356,
    "no_place_failures": 0
  },
  {
    "alive": 64.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 480.325,
    "wood": 89.075,
    "seed": 18027,
    "old_age_deaths": 79,
    "social_completed": 6379.0,
    "adult_days": 3856.2083333333335,
    "social_rate": 1.6542156047066958,
    "no_place_redraws": 115,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 254.74166666666667,
    "wood": 79.89166666666667,
    "seed": 18028,
    "old_age_deaths": 45,
    "social_completed": 3049.0,
    "adult_days": 2026.5,
    "social_rate": 1.5045645201085616,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 42.0,
    "deaths": 3.0,
    "shelters": 6.0,
    "food": 397.89166666666665,
    "wood": 89.84166666666667,
    "seed": 18029,
    "old_age_deaths": 71,
    "social_completed": 6000.0,
    "adult_days": 3315.0,
    "social_rate": 1.8099547511312217,
    "no_place_redraws": 37,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 349.35,
    "wood": 82.58333333333333,
    "seed": 18030,
    "old_age_deaths": 56,
    "social_completed": 4504.0,
    "adult_days": 2839.2083333333335,
    "social_rate": 1.5863576994760862,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 31.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 245.69166666666666,
    "wood": 82.84166666666667,
    "seed": 18031,
    "old_age_deaths": 61,
    "social_completed": 4631.0,
    "adult_days": 2711.7916666666665,
    "social_rate": 1.7077270562205185,
    "no_place_redraws": 0,
    "no_place_failures": 0
  }
]
```

## game_v22

32 worlds × 120 days; seeds [18000, 18001, 18002, 18003, 18004, 18005, 18006, 18007, 18008, 18009, 18010, 18011, 18012, 18013, 18014, 18015, 18016, 18017, 18018, 18019, 18020, 18021, 18022, 18023, 18024, 18025, 18026, 18027, 18028, 18029, 18030, 18031].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 130976 | 0 | 0.0627443 | 0 | 0.937256 | 0 |
| 0 | child | 13710 | 0 | 0.00437637 | 0 | 0.995624 | 0 |
| 0 | elder | 31628 | 0 | 0.00749336 | 0 | 0.992507 | 0 |
| 60 | adult | 131005 | 5.34331e-05 | 0.0299454 | 0 | 0.970001 | 0 |
| 60 | child | 13719 | 0 | 0.000728916 | 0 | 0.999271 | 0 |
| 60 | elder | 31616 | 0 | 0.00230896 | 0 | 0.997691 | 0 |
| 120 | adult | 131032 | 5.34221e-05 | 0.0169348 | 0 | 0.983012 | 0 |
| 120 | child | 13731 | 0 | 0.000145656 | 0 | 0.999854 | 0 |
| 120 | elder | 31598 | 3.16476e-05 | 0.00623457 | 0 | 0.993734 | 0 |
| 180 | adult | 131016 | 0.000366367 | 0.0866001 | 0 | 0.913034 | 0 |
| 180 | child | 13730 | 0 | 0.000145666 | 0 | 0.999854 | 0 |
| 180 | elder | 31567 | 6.33573e-05 | 0.096905 | 0 | 0.903032 | 0 |
| 240 | adult | 131016 | 0.00022898 | 0.11407 | 0 | 0.885701 | 0 |
| 240 | child | 13724 | 0 | 0.00029146 | 0 | 0.999709 | 0 |
| 240 | elder | 31516 | 9.51897e-05 | 0.12692 | 0 | 0.872985 | 0 |
| 300 | adult | 130996 | 0.00338178 | 0.163654 | 0 | 0.832964 | 0 |
| 300 | child | 13713 | 0.000145847 | 0.000583388 | 0 | 0.999271 | 0 |
| 300 | elder | 31489 | 0.0031122 | 0.180031 | 0 | 0.816857 | 0 |
| 360 | adult | 130999 | 0.801243 | 0.150536 | 0 | 0.0482217 | 0 |
| 360 | child | 13714 | 0.973968 | 0.0218025 | 0 | 0.00422925 | 0 |
| 360 | elder | 31464 | 0.796784 | 0.154558 | 0 | 0.0486588 | 0 |
| 420 | adult | 131054 | 0.163101 | 0.825003 | 0.00948464 | 0.000236544 | 0.00217468 |
| 420 | child | 13722 | 0.823204 | 0.166302 | 0 | 0.000145751 | 0.0103483 |
| 420 | elder | 31497 | 0.534591 | 0.440359 | 0.016446 | 0.000190494 | 0.0084135 |
| 480 | adult | 130600 | 0.0594334 | 0.618292 | 0.00418836 | 0.0390429 | 0.279043 |
| 480 | child | 13583 | 0.0103806 | 0.0352647 | 0 | 0.0147243 | 0.93963 |
| 480 | elder | 31180 | 0.00875561 | 0.305548 | 0.00420141 | 0.0184092 | 0.663085 |
| 540 | adult | 130656 | 0.0991612 | 0.559821 | 0.00500551 | 0.000734754 | 0.335277 |
| 540 | child | 13592 | 0.140377 | 0.0371542 | 0 | 0 | 0.822469 |
| 540 | elder | 31439 | 0.116034 | 0.264989 | 0.00254461 | 9.54229e-05 | 0.616336 |
| 600 | adult | 130701 | 0.101698 | 0.587142 | 0.00195102 | 0.0307113 | 0.278498 |
| 600 | child | 13595 | 0.341964 | 0.036484 | 0 | 0.0113277 | 0.610224 |
| 600 | elder | 31433 | 0.221042 | 0.226036 | 0.00130436 | 0.0163841 | 0.535234 |
| 660 | adult | 130721 | 0.133835 | 0.474797 | 0.00445223 | 0.000497242 | 0.386418 |
| 660 | child | 13603 | 0.183268 | 0.044843 | 0 | 7.35132e-05 | 0.771815 |
| 660 | elder | 31388 | 0.151969 | 0.0951319 | 0.00146553 | 3.18593e-05 | 0.751402 |
| 720 | adult | 130745 | 0.152235 | 0.278328 | 0.00113197 | 0.0406746 | 0.52763 |
| 720 | child | 13620 | 0.25815 | 0.0406755 | 0 | 0.010793 | 0.690382 |
| 720 | elder | 31400 | 0.19121 | 0.093535 | 0.00127389 | 0.0202866 | 0.693694 |
| 780 | adult | 130767 | 0.15224 | 0.328041 | 0.00372418 | 0.000267652 | 0.515726 |
| 780 | child | 13628 | 0.256531 | 0.040138 | 0 | 0 | 0.703331 |
| 780 | elder | 31389 | 0.209564 | 0.0876103 | 0.00124247 | 0 | 0.701583 |
| 840 | adult | 130792 | 0.125268 | 0.21434 | 0.00128448 | 0.0301624 | 0.628945 |
| 840 | child | 13641 | 0.231068 | 0.0448647 | 0 | 0.0113628 | 0.712704 |
| 840 | elder | 31359 | 0.193437 | 0.0874071 | 0.00051022 | 0.0186868 | 0.699959 |
| 900 | adult | 130791 | 0.12812 | 0.314922 | 0.00344825 | 7.64579e-05 | 0.553433 |
| 900 | child | 13642 | 0.230978 | 0.0608415 | 0 | 0.000366515 | 0.707814 |
| 900 | elder | 31337 | 0.202795 | 0.0832881 | 0.00124454 | 3.19112e-05 | 0.71264 |
| 960 | adult | 130798 | 0.1191 | 0.219032 | 0.000963317 | 0.0305815 | 0.630323 |
| 960 | child | 13644 | 0.233949 | 0.0454412 | 0 | 0.00982117 | 0.710789 |
| 960 | elder | 31301 | 0.19616 | 0.0860995 | 0.00089454 | 0.0190729 | 0.697773 |
| 1020 | adult | 130800 | 0.115222 | 0.310329 | 0.00315749 | 2.29358e-05 | 0.571269 |
| 1020 | child | 13648 | 0.242453 | 0.0426436 | 0 | 0 | 0.714903 |
| 1020 | elder | 31263 | 0.199373 | 0.0935931 | 0.00150337 | 0 | 0.70553 |
| 1080 | adult | 130811 | 0.749364 | 0.220272 | 0.000756817 | 0.0296076 | 0 |
| 1080 | child | 13651 | 0.947769 | 0.0416087 | 0 | 0.0106219 | 0 |
| 1080 | elder | 31236 | 0.887566 | 0.0912409 | 0.000800359 | 0.0203931 | 0 |
| 1140 | adult | 130842 | 0.600992 | 0.391434 | 0.00756638 | 7.64281e-06 | 0 |
| 1140 | child | 13656 | 0.958992 | 0.0410076 | 0 | 0 | 0 |
| 1140 | elder | 31244 | 0.908654 | 0.0891371 | 0.00220842 | 0 | 0 |
| 1200 | adult | 130859 | 0.712935 | 0.252539 | 0.00276634 | 0.0317594 | 0 |
| 1200 | child | 13665 | 0.947969 | 0.0414929 | 0 | 0.0105379 | 0 |
| 1200 | elder | 31387 | 0.887852 | 0.0912161 | 0.0018479 | 0.0190843 | 0 |
| 1260 | adult | 130915 | 0.349311 | 0.566742 | 0.00149715 | 0.0824504 | 0 |
| 1260 | child | 13685 | 0.474096 | 0.324808 | 0 | 0.201096 | 0 |
| 1260 | elder | 31630 | 0.474802 | 0.389725 | 0.000442618 | 0.13503 | 0 |
| 1320 | adult | 130950 | 0 | 0.356373 | 0 | 0.643627 | 0 |
| 1320 | child | 13691 | 0 | 0.0549996 | 0 | 0.945 | 0 |
| 1320 | elder | 31635 | 0 | 0.0963806 | 0 | 0.903619 | 0 |
| 1380 | adult | 130951 | 3.05458e-05 | 0.159502 | 0 | 0.840467 | 0 |
| 1380 | child | 13700 | 0 | 0.0153285 | 0 | 0.984672 | 0 |
| 1380 | elder | 31627 | 0 | 0.0269706 | 0 | 0.973029 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 4457458,
    "adult_minutes": 4454394,
    "all_life_stages_share": 0.13595900115021026,
    "adult_share": 0.25290043303807047,
    "all_life_stages_starts": 144326
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 2921153,
    "adult_minutes": 2919380,
    "all_life_stages_share": 0.08909944728294471,
    "adult_share": 0.16574925033633806,
    "all_life_stages_starts": 90092
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 3559958,
    "adult_minutes": 3557498,
    "all_life_stages_share": 0.10858393591520105,
    "adult_share": 0.20197871691010486,
    "all_life_stages_starts": 113467
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 10104784,
    "adult_minutes": 5091286,
    "all_life_stages_share": 0.30821071998404165,
    "adult_share": 0.2890602928525554,
    "all_life_stages_starts": 257563
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 1564464,
    "adult_minutes": 1562425,
    "all_life_stages_share": 0.04771844463267238,
    "adult_share": 0.08870745584910254,
    "all_life_stages_starts": 98836
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 4987537,
    "adult_minutes": 28249,
    "all_life_stages_share": 0.15212718745072107,
    "adult_share": 0.0016038510138286942,
    "all_life_stages_starts": 68764
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 5189956,
    "adult_minutes": 0,
    "all_life_stages_share": 0.1583012635842089,
    "adult_share": 0.0,
    "all_life_stages_starts": 93790
  }
]
```

### health_by_world

```json
[
  {
    "alive": 78.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 688.05,
    "wood": 161.00833333333333,
    "seed": 18000,
    "old_age_deaths": 107,
    "social_completed": 10681.0,
    "adult_days": 5992.708333333333,
    "social_rate": 1.7823326959847037,
    "no_place_redraws": 2449,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 333.3666666666667,
    "wood": 85.94166666666666,
    "seed": 18001,
    "old_age_deaths": 66,
    "social_completed": 5060.0,
    "adult_days": 2935.9583333333335,
    "social_rate": 1.7234577012048875,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 345.2,
    "wood": 91.88333333333334,
    "seed": 18002,
    "old_age_deaths": 60,
    "social_completed": 4955.0,
    "adult_days": 2771.4583333333335,
    "social_rate": 1.7878673983312035,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 31.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 317.4583333333333,
    "wood": 92.33333333333333,
    "seed": 18003,
    "old_age_deaths": 55,
    "social_completed": 4507.0,
    "adult_days": 2700.0416666666665,
    "social_rate": 1.6692334994830327,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 58.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 559.9666666666667,
    "wood": 151.1,
    "seed": 18004,
    "old_age_deaths": 90,
    "social_completed": 7585.0,
    "adult_days": 4485.75,
    "social_rate": 1.6909101042189154,
    "no_place_redraws": 2500,
    "no_place_failures": 0
  },
  {
    "alive": 26.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 267.6666666666667,
    "wood": 78.50833333333334,
    "seed": 18005,
    "old_age_deaths": 51,
    "social_completed": 3852.0,
    "adult_days": 2278.5416666666665,
    "social_rate": 1.6905549967998539,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 284.1,
    "wood": 85.025,
    "seed": 18006,
    "old_age_deaths": 64,
    "social_completed": 4944.0,
    "adult_days": 2783.0,
    "social_rate": 1.776500179662235,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 87.0,
    "deaths": 0.0,
    "shelters": 10.0,
    "food": 740.3666666666667,
    "wood": 150.19166666666666,
    "seed": 18007,
    "old_age_deaths": 120,
    "social_completed": 11761.0,
    "adult_days": 6385.958333333333,
    "social_rate": 1.8416969522976845,
    "no_place_redraws": 3301,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 2.0,
    "shelters": 6.0,
    "food": 517.1416666666667,
    "wood": 147.56666666666666,
    "seed": 18008,
    "old_age_deaths": 90,
    "social_completed": 8120.0,
    "adult_days": 4486.791666666667,
    "social_rate": 1.8097564146615528,
    "no_place_redraws": 2266,
    "no_place_failures": 0
  },
  {
    "alive": 67.0,
    "deaths": 1.0,
    "shelters": 7.0,
    "food": 370.10833333333335,
    "wood": 97.49166666666666,
    "seed": 18009,
    "old_age_deaths": 64,
    "social_completed": 5832.0,
    "adult_days": 3516.1666666666665,
    "social_rate": 1.6586244489737878,
    "no_place_redraws": 896,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 194.225,
    "wood": 77.075,
    "seed": 18010,
    "old_age_deaths": 46,
    "social_completed": 3131.0,
    "adult_days": 1992.5833333333333,
    "social_rate": 1.5713270043076408,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 294.51666666666665,
    "wood": 91.225,
    "seed": 18011,
    "old_age_deaths": 65,
    "social_completed": 5013.0,
    "adult_days": 2996.7083333333335,
    "social_rate": 1.6728354722542789,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 79.0,
    "deaths": 13.0,
    "shelters": 9.0,
    "food": 569.2583333333333,
    "wood": 115.48333333333333,
    "seed": 18012,
    "old_age_deaths": 81,
    "social_completed": 9570.0,
    "adult_days": 5442.333333333333,
    "social_rate": 1.7584369449378332,
    "no_place_redraws": 1684,
    "no_place_failures": 0
  },
  {
    "alive": 29.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 302.48333333333335,
    "wood": 77.66666666666667,
    "seed": 18013,
    "old_age_deaths": 53,
    "social_completed": 4279.0,
    "adult_days": 2509.125,
    "social_rate": 1.7053753798634983,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 92.0,
    "deaths": 0.0,
    "shelters": 12.0,
    "food": 695.8666666666667,
    "wood": 177.01666666666668,
    "seed": 18014,
    "old_age_deaths": 94,
    "social_completed": 10591.0,
    "adult_days": 5734.541666666667,
    "social_rate": 1.8468782015418261,
    "no_place_redraws": 3063,
    "no_place_failures": 0
  },
  {
    "alive": 35.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 364.06666666666666,
    "wood": 87.6,
    "seed": 18015,
    "old_age_deaths": 67,
    "social_completed": 5206.0,
    "adult_days": 3076.6666666666665,
    "social_rate": 1.6920910075839655,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 73.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 511.1166666666667,
    "wood": 134.60833333333332,
    "seed": 18016,
    "old_age_deaths": 69,
    "social_completed": 7876.0,
    "adult_days": 4673.0,
    "social_rate": 1.6854269206077466,
    "no_place_redraws": 1985,
    "no_place_failures": 0
  },
  {
    "alive": 80.0,
    "deaths": 1.0,
    "shelters": 9.0,
    "food": 714.25,
    "wood": 164.66666666666666,
    "seed": 18017,
    "old_age_deaths": 114,
    "social_completed": 12946.0,
    "adult_days": 6832.125,
    "social_rate": 1.8948716541339627,
    "no_place_redraws": 3346,
    "no_place_failures": 1
  },
  {
    "alive": 121.0,
    "deaths": 0.0,
    "shelters": 16.0,
    "food": 841.5416666666666,
    "wood": 185.70833333333334,
    "seed": 18018,
    "old_age_deaths": 122,
    "social_completed": 14116.0,
    "adult_days": 7593.25,
    "social_rate": 1.8590195239192704,
    "no_place_redraws": 5966,
    "no_place_failures": 3
  },
  {
    "alive": 85.0,
    "deaths": 5.0,
    "shelters": 13.0,
    "food": 653.0666666666667,
    "wood": 126.725,
    "seed": 18019,
    "old_age_deaths": 91,
    "social_completed": 10377.0,
    "adult_days": 5619.25,
    "social_rate": 1.8466877252302354,
    "no_place_redraws": 1977,
    "no_place_failures": 1
  },
  {
    "alive": 24.0,
    "deaths": 2.0,
    "shelters": 1.0,
    "food": 254.86666666666667,
    "wood": 75.18333333333334,
    "seed": 18020,
    "old_age_deaths": 48,
    "social_completed": 3300.0,
    "adult_days": 2107.5,
    "social_rate": 1.5658362989323844,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 67.0,
    "deaths": 2.0,
    "shelters": 8.0,
    "food": 521.5666666666667,
    "wood": 136.325,
    "seed": 18021,
    "old_age_deaths": 74,
    "social_completed": 7569.0,
    "adult_days": 4475.875,
    "social_rate": 1.691065992682995,
    "no_place_redraws": 2176,
    "no_place_failures": 0
  },
  {
    "alive": 72.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 572.2166666666667,
    "wood": 130.20833333333334,
    "seed": 18022,
    "old_age_deaths": 71,
    "social_completed": 8296.0,
    "adult_days": 4777.791666666667,
    "social_rate": 1.7363670454446352,
    "no_place_redraws": 2282,
    "no_place_failures": 0
  },
  {
    "alive": 97.0,
    "deaths": 2.0,
    "shelters": 12.0,
    "food": 732.8083333333333,
    "wood": 173.96666666666667,
    "seed": 18023,
    "old_age_deaths": 112,
    "social_completed": 12537.0,
    "adult_days": 6887.75,
    "social_rate": 1.8201880149540852,
    "no_place_redraws": 3196,
    "no_place_failures": 0
  },
  {
    "alive": 90.0,
    "deaths": 3.0,
    "shelters": 13.0,
    "food": 671.2416666666667,
    "wood": 137.7,
    "seed": 18024,
    "old_age_deaths": 96,
    "social_completed": 9866.0,
    "adult_days": 5457.666666666667,
    "social_rate": 1.8077322421059059,
    "no_place_redraws": 3325,
    "no_place_failures": 0
  },
  {
    "alive": 24.0,
    "deaths": 2.0,
    "shelters": 1.0,
    "food": 264.28333333333336,
    "wood": 78.85,
    "seed": 18025,
    "old_age_deaths": 48,
    "social_completed": 3611.0,
    "adult_days": 2109.2916666666665,
    "social_rate": 1.7119491140390732,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 86.0,
    "deaths": 1.0,
    "shelters": 11.0,
    "food": 533.0,
    "wood": 141.21666666666667,
    "seed": 18026,
    "old_age_deaths": 68,
    "social_completed": 8386.0,
    "adult_days": 4737.416666666667,
    "social_rate": 1.770163063554328,
    "no_place_redraws": 2408,
    "no_place_failures": 0
  },
  {
    "alive": 81.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 523.575,
    "wood": 112.28333333333333,
    "seed": 18027,
    "old_age_deaths": 80,
    "social_completed": 8015.0,
    "adult_days": 4605.875,
    "social_rate": 1.7401688061443266,
    "no_place_redraws": 1095,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 257.125,
    "wood": 79.875,
    "seed": 18028,
    "old_age_deaths": 45,
    "social_completed": 3058.0,
    "adult_days": 2045.375,
    "social_rate": 1.495080364236387,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 46.0,
    "deaths": 3.0,
    "shelters": 7.0,
    "food": 394.9,
    "wood": 91.39166666666667,
    "seed": 18029,
    "old_age_deaths": 68,
    "social_completed": 6259.0,
    "adult_days": 3298.2916666666665,
    "social_rate": 1.8976490354855418,
    "no_place_redraws": 42,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 349.3333333333333,
    "wood": 81.79166666666667,
    "seed": 18030,
    "old_age_deaths": 54,
    "social_completed": 4463.0,
    "adult_days": 2869.2916666666665,
    "social_rate": 1.5554361558456646,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 31.0,
    "deaths": 2.0,
    "shelters": 2.0,
    "food": 246.60833333333332,
    "wood": 82.5,
    "seed": 18031,
    "old_age_deaths": 61,
    "social_completed": 4483.0,
    "adult_days": 2688.2916666666665,
    "social_rate": 1.6676017917202686,
    "no_place_redraws": 0,
    "no_place_failures": 0
  }
]
```

## Family observations

Full pair IDs, first snapshots, life-stage categories and file hashes are retained in the JSON report.

```json
{
  "births_by_day": [
    {
      "day": 1,
      "births": 30,
      "couple_births": 0,
      "fallback_births": 30
    },
    {
      "day": 2,
      "births": 55,
      "couple_births": 0,
      "fallback_births": 55
    },
    {
      "day": 3,
      "births": 48,
      "couple_births": 3,
      "fallback_births": 45
    },
    {
      "day": 4,
      "births": 44,
      "couple_births": 14,
      "fallback_births": 30
    },
    {
      "day": 5,
      "births": 40,
      "couple_births": 24,
      "fallback_births": 16
    },
    {
      "day": 6,
      "births": 31,
      "couple_births": 25,
      "fallback_births": 6
    },
    {
      "day": 7,
      "births": 29,
      "couple_births": 27,
      "fallback_births": 2
    },
    {
      "day": 8,
      "births": 26,
      "couple_births": 26,
      "fallback_births": 0
    },
    {
      "day": 9,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 10,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 11,
      "births": 15,
      "couple_births": 15,
      "fallback_births": 0
    },
    {
      "day": 12,
      "births": 11,
      "couple_births": 11,
      "fallback_births": 0
    },
    {
      "day": 13,
      "births": 7,
      "couple_births": 6,
      "fallback_births": 1
    },
    {
      "day": 14,
      "births": 8,
      "couple_births": 7,
      "fallback_births": 1
    },
    {
      "day": 15,
      "births": 8,
      "couple_births": 8,
      "fallback_births": 0
    },
    {
      "day": 16,
      "births": 8,
      "couple_births": 8,
      "fallback_births": 0
    },
    {
      "day": 17,
      "births": 12,
      "couple_births": 12,
      "fallback_births": 0
    },
    {
      "day": 18,
      "births": 11,
      "couple_births": 11,
      "fallback_births": 0
    },
    {
      "day": 19,
      "births": 13,
      "couple_births": 13,
      "fallback_births": 0
    },
    {
      "day": 20,
      "births": 22,
      "couple_births": 22,
      "fallback_births": 0
    },
    {
      "day": 21,
      "births": 18,
      "couple_births": 18,
      "fallback_births": 0
    },
    {
      "day": 22,
      "births": 16,
      "couple_births": 16,
      "fallback_births": 0
    },
    {
      "day": 23,
      "births": 26,
      "couple_births": 26,
      "fallback_births": 0
    },
    {
      "day": 24,
      "births": 14,
      "couple_births": 14,
      "fallback_births": 0
    },
    {
      "day": 25,
      "births": 23,
      "couple_births": 23,
      "fallback_births": 0
    },
    {
      "day": 26,
      "births": 22,
      "couple_births": 22,
      "fallback_births": 0
    },
    {
      "day": 27,
      "births": 23,
      "couple_births": 23,
      "fallback_births": 0
    },
    {
      "day": 28,
      "births": 30,
      "couple_births": 29,
      "fallback_births": 1
    },
    {
      "day": 29,
      "births": 29,
      "couple_births": 27,
      "fallback_births": 2
    },
    {
      "day": 30,
      "births": 24,
      "couple_births": 22,
      "fallback_births": 2
    },
    {
      "day": 31,
      "births": 31,
      "couple_births": 29,
      "fallback_births": 2
    },
    {
      "day": 32,
      "births": 33,
      "couple_births": 31,
      "fallback_births": 2
    },
    {
      "day": 33,
      "births": 43,
      "couple_births": 40,
      "fallback_births": 3
    },
    {
      "day": 34,
      "births": 41,
      "couple_births": 36,
      "fallback_births": 5
    },
    {
      "day": 35,
      "births": 42,
      "couple_births": 36,
      "fallback_births": 6
    },
    {
      "day": 36,
      "births": 48,
      "couple_births": 42,
      "fallback_births": 6
    },
    {
      "day": 37,
      "births": 49,
      "couple_births": 43,
      "fallback_births": 6
    },
    {
      "day": 38,
      "births": 43,
      "couple_births": 38,
      "fallback_births": 5
    },
    {
      "day": 39,
      "births": 52,
      "couple_births": 48,
      "fallback_births": 4
    },
    {
      "day": 40,
      "births": 45,
      "couple_births": 41,
      "fallback_births": 4
    },
    {
      "day": 41,
      "births": 36,
      "couple_births": 32,
      "fallback_births": 4
    },
    {
      "day": 42,
      "births": 32,
      "couple_births": 29,
      "fallback_births": 3
    },
    {
      "day": 43,
      "births": 25,
      "couple_births": 23,
      "fallback_births": 2
    },
    {
      "day": 44,
      "births": 23,
      "couple_births": 21,
      "fallback_births": 2
    },
    {
      "day": 45,
      "births": 32,
      "couple_births": 30,
      "fallback_births": 2
    },
    {
      "day": 46,
      "births": 28,
      "couple_births": 26,
      "fallback_births": 2
    },
    {
      "day": 47,
      "births": 32,
      "couple_births": 32,
      "fallback_births": 0
    },
    {
      "day": 48,
      "births": 25,
      "couple_births": 25,
      "fallback_births": 0
    },
    {
      "day": 49,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 50,
      "births": 30,
      "couple_births": 30,
      "fallback_births": 0
    },
    {
      "day": 51,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 52,
      "births": 15,
      "couple_births": 15,
      "fallback_births": 0
    },
    {
      "day": 53,
      "births": 8,
      "couple_births": 8,
      "fallback_births": 0
    },
    {
      "day": 54,
      "births": 10,
      "couple_births": 10,
      "fallback_births": 0
    },
    {
      "day": 55,
      "births": 8,
      "couple_births": 8,
      "fallback_births": 0
    },
    {
      "day": 56,
      "births": 10,
      "couple_births": 10,
      "fallback_births": 0
    },
    {
      "day": 57,
      "births": 15,
      "couple_births": 15,
      "fallback_births": 0
    },
    {
      "day": 58,
      "births": 18,
      "couple_births": 18,
      "fallback_births": 0
    },
    {
      "day": 59,
      "births": 31,
      "couple_births": 31,
      "fallback_births": 0
    },
    {
      "day": 60,
      "births": 36,
      "couple_births": 36,
      "fallback_births": 0
    },
    {
      "day": 61,
      "births": 43,
      "couple_births": 42,
      "fallback_births": 1
    },
    {
      "day": 62,
      "births": 50,
      "couple_births": 48,
      "fallback_births": 2
    },
    {
      "day": 63,
      "births": 48,
      "couple_births": 46,
      "fallback_births": 2
    },
    {
      "day": 64,
      "births": 56,
      "couple_births": 54,
      "fallback_births": 2
    },
    {
      "day": 65,
      "births": 48,
      "couple_births": 46,
      "fallback_births": 2
    },
    {
      "day": 66,
      "births": 46,
      "couple_births": 44,
      "fallback_births": 2
    },
    {
      "day": 67,
      "births": 40,
      "couple_births": 38,
      "fallback_births": 2
    },
    {
      "day": 68,
      "births": 43,
      "couple_births": 41,
      "fallback_births": 2
    },
    {
      "day": 69,
      "births": 32,
      "couple_births": 30,
      "fallback_births": 2
    },
    {
      "day": 70,
      "births": 27,
      "couple_births": 25,
      "fallback_births": 2
    },
    {
      "day": 71,
      "births": 26,
      "couple_births": 24,
      "fallback_births": 2
    },
    {
      "day": 72,
      "births": 27,
      "couple_births": 26,
      "fallback_births": 1
    },
    {
      "day": 73,
      "births": 24,
      "couple_births": 24,
      "fallback_births": 0
    },
    {
      "day": 74,
      "births": 26,
      "couple_births": 26,
      "fallback_births": 0
    },
    {
      "day": 75,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 76,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 77,
      "births": 17,
      "couple_births": 17,
      "fallback_births": 0
    },
    {
      "day": 78,
      "births": 17,
      "couple_births": 17,
      "fallback_births": 0
    },
    {
      "day": 79,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 80,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 81,
      "births": 21,
      "couple_births": 21,
      "fallback_births": 0
    },
    {
      "day": 82,
      "births": 22,
      "couple_births": 22,
      "fallback_births": 0
    },
    {
      "day": 83,
      "births": 24,
      "couple_births": 24,
      "fallback_births": 0
    },
    {
      "day": 84,
      "births": 21,
      "couple_births": 21,
      "fallback_births": 0
    },
    {
      "day": 85,
      "births": 28,
      "couple_births": 28,
      "fallback_births": 0
    },
    {
      "day": 86,
      "births": 32,
      "couple_births": 32,
      "fallback_births": 0
    },
    {
      "day": 87,
      "births": 29,
      "couple_births": 28,
      "fallback_births": 1
    },
    {
      "day": 88,
      "births": 35,
      "couple_births": 34,
      "fallback_births": 1
    },
    {
      "day": 89,
      "births": 31,
      "couple_births": 29,
      "fallback_births": 2
    },
    {
      "day": 90,
      "births": 27,
      "couple_births": 25,
      "fallback_births": 2
    },
    {
      "day": 91,
      "births": 39,
      "couple_births": 39,
      "fallback_births": 0
    },
    {
      "day": 92,
      "births": 47,
      "couple_births": 46,
      "fallback_births": 1
    },
    {
      "day": 93,
      "births": 42,
      "couple_births": 39,
      "fallback_births": 3
    },
    {
      "day": 94,
      "births": 53,
      "couple_births": 52,
      "fallback_births": 1
    },
    {
      "day": 95,
      "births": 50,
      "couple_births": 50,
      "fallback_births": 0
    },
    {
      "day": 96,
      "births": 55,
      "couple_births": 55,
      "fallback_births": 0
    },
    {
      "day": 97,
      "births": 48,
      "couple_births": 48,
      "fallback_births": 0
    },
    {
      "day": 98,
      "births": 41,
      "couple_births": 41,
      "fallback_births": 0
    },
    {
      "day": 99,
      "births": 43,
      "couple_births": 43,
      "fallback_births": 0
    },
    {
      "day": 100,
      "births": 50,
      "couple_births": 50,
      "fallback_births": 0
    },
    {
      "day": 101,
      "births": 45,
      "couple_births": 45,
      "fallback_births": 0
    },
    {
      "day": 102,
      "births": 42,
      "couple_births": 42,
      "fallback_births": 0
    },
    {
      "day": 103,
      "births": 44,
      "couple_births": 44,
      "fallback_births": 0
    },
    {
      "day": 104,
      "births": 29,
      "couple_births": 29,
      "fallback_births": 0
    },
    {
      "day": 105,
      "births": 32,
      "couple_births": 32,
      "fallback_births": 0
    },
    {
      "day": 106,
      "births": 21,
      "couple_births": 21,
      "fallback_births": 0
    },
    {
      "day": 107,
      "births": 16,
      "couple_births": 16,
      "fallback_births": 0
    },
    {
      "day": 108,
      "births": 14,
      "couple_births": 14,
      "fallback_births": 0
    },
    {
      "day": 109,
      "births": 16,
      "couple_births": 16,
      "fallback_births": 0
    },
    {
      "day": 110,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 111,
      "births": 21,
      "couple_births": 20,
      "fallback_births": 1
    },
    {
      "day": 112,
      "births": 23,
      "couple_births": 23,
      "fallback_births": 0
    },
    {
      "day": 113,
      "births": 15,
      "couple_births": 15,
      "fallback_births": 0
    },
    {
      "day": 114,
      "births": 22,
      "couple_births": 22,
      "fallback_births": 0
    },
    {
      "day": 115,
      "births": 25,
      "couple_births": 25,
      "fallback_births": 0
    },
    {
      "day": 116,
      "births": 25,
      "couple_births": 25,
      "fallback_births": 0
    },
    {
      "day": 117,
      "births": 28,
      "couple_births": 28,
      "fallback_births": 0
    },
    {
      "day": 118,
      "births": 42,
      "couple_births": 42,
      "fallback_births": 0
    },
    {
      "day": 119,
      "births": 37,
      "couple_births": 35,
      "fallback_births": 2
    },
    {
      "day": 120,
      "births": 42,
      "couple_births": 40,
      "fallback_births": 2
    }
  ],
  "worlds": [
    {
      "seed": 18000,
      "day60_partnered_adults": 32.0,
      "day60_adults": 57.0,
      "births_from_day20": 140.0,
      "couple_births_from_day20": 140.0,
      "all_births": 164,
      "all_fallback_births": 11,
      "same_shelter_couple_nights": 1731.0,
      "bedded_couple_nights": 1738.0,
      "all_living_couple_nights": 1738,
      "bedless_person_nights": 18,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 10.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 31,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 377,
      "unrelated_unordered_pairs": 3910,
      "excluded_close_kin_pairs": 367,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18001,
      "day60_partnered_adults": 19.0,
      "day60_adults": 29.0,
      "births_from_day20": 66.0,
      "couple_births_from_day20": 66.0,
      "all_births": 72,
      "all_fallback_births": 4,
      "same_shelter_couple_nights": 967.0,
      "bedded_couple_nights": 967.0,
      "all_living_couple_nights": 967,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 72,
      "unrelated_unordered_pairs": 1604,
      "excluded_close_kin_pairs": 157,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18002,
      "day60_partnered_adults": 16.0,
      "day60_adults": 23.0,
      "births_from_day20": 60.0,
      "couple_births_from_day20": 60.0,
      "all_births": 70,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 700.0,
      "bedded_couple_nights": 700.0,
      "all_living_couple_nights": 700,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 105,
      "unrelated_unordered_pairs": 1479,
      "excluded_close_kin_pairs": 140,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18003,
      "day60_partnered_adults": 11.0,
      "day60_adults": 20.0,
      "births_from_day20": 53.0,
      "couple_births_from_day20": 53.0,
      "all_births": 67,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 855.0,
      "bedded_couple_nights": 855.0,
      "all_living_couple_nights": 855,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 68,
      "unrelated_unordered_pairs": 1350,
      "excluded_close_kin_pairs": 163,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18004,
      "day60_partnered_adults": 11.0,
      "day60_adults": 54.0,
      "births_from_day20": 114.0,
      "couple_births_from_day20": 93.0,
      "all_births": 119,
      "all_fallback_births": 25,
      "same_shelter_couple_nights": 1044.0,
      "bedded_couple_nights": 1053.0,
      "all_living_couple_nights": 1053,
      "bedless_person_nights": 5,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 7.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 23,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 347,
      "unrelated_unordered_pairs": 1840,
      "excluded_close_kin_pairs": 301,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18005,
      "day60_partnered_adults": 10.0,
      "day60_adults": 25.0,
      "births_from_day20": 51.0,
      "couple_births_from_day20": 50.0,
      "all_births": 53,
      "all_fallback_births": 3,
      "same_shelter_couple_nights": 653.0,
      "bedded_couple_nights": 653.0,
      "all_living_couple_nights": 653,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 98,
      "unrelated_unordered_pairs": 769,
      "excluded_close_kin_pairs": 124,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18006,
      "day60_partnered_adults": 18.0,
      "day60_adults": 30.0,
      "births_from_day20": 63.0,
      "couple_births_from_day20": 63.0,
      "all_births": 67,
      "all_fallback_births": 2,
      "same_shelter_couple_nights": 938.0,
      "bedded_couple_nights": 938.0,
      "all_living_couple_nights": 938,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 49,
      "unrelated_unordered_pairs": 1393,
      "excluded_close_kin_pairs": 144,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18007,
      "day60_partnered_adults": 32.0,
      "day60_adults": 67.0,
      "births_from_day20": 158.0,
      "couple_births_from_day20": 158.0,
      "all_births": 180,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 2011.0,
      "bedded_couple_nights": 2020.0,
      "all_living_couple_nights": 2020,
      "bedless_person_nights": 8,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 15.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 27,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 343,
      "unrelated_unordered_pairs": 4938,
      "excluded_close_kin_pairs": 378,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18008,
      "day60_partnered_adults": 28.0,
      "day60_adults": 60.0,
      "births_from_day20": 124.0,
      "couple_births_from_day20": 83.0,
      "all_births": 126,
      "all_fallback_births": 43,
      "same_shelter_couple_nights": 931.0,
      "bedded_couple_nights": 935.0,
      "all_living_couple_nights": 935,
      "bedless_person_nights": 10,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 2.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 20,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 672,
      "unrelated_unordered_pairs": 1487,
      "excluded_close_kin_pairs": 329,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18009,
      "day60_partnered_adults": 11.0,
      "day60_adults": 27.0,
      "births_from_day20": 100.0,
      "couple_births_from_day20": 99.0,
      "all_births": 105,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 815.0,
      "bedded_couple_nights": 827.0,
      "all_living_couple_nights": 827,
      "bedless_person_nights": 3,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 7.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 21,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 312,
      "unrelated_unordered_pairs": 1821,
      "excluded_close_kin_pairs": 227,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18010,
      "day60_partnered_adults": 10.0,
      "day60_adults": 20.0,
      "births_from_day20": 46.0,
      "couple_births_from_day20": 46.0,
      "all_births": 50,
      "all_fallback_births": 3,
      "same_shelter_couple_nights": 636.0,
      "bedded_couple_nights": 636.0,
      "all_living_couple_nights": 636,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 53,
      "unrelated_unordered_pairs": 673,
      "excluded_close_kin_pairs": 108,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18011,
      "day60_partnered_adults": 17.0,
      "day60_adults": 24.0,
      "births_from_day20": 62.0,
      "couple_births_from_day20": 62.0,
      "all_births": 74,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 968.0,
      "bedded_couple_nights": 968.0,
      "all_living_couple_nights": 968,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 71,
      "unrelated_unordered_pairs": 1752,
      "excluded_close_kin_pairs": 166,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18012,
      "day60_partnered_adults": 12.0,
      "day60_adults": 22.0,
      "births_from_day20": 124.0,
      "couple_births_from_day20": 102.0,
      "all_births": 154,
      "all_fallback_births": 31,
      "same_shelter_couple_nights": 1747.0,
      "bedded_couple_nights": 1787.0,
      "all_living_couple_nights": 1787,
      "bedless_person_nights": 3,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 16.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 30,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 285,
      "unrelated_unordered_pairs": 4129,
      "excluded_close_kin_pairs": 379,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18013,
      "day60_partnered_adults": 8.0,
      "day60_adults": 20.0,
      "births_from_day20": 53.0,
      "couple_births_from_day20": 53.0,
      "all_births": 62,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 774.0,
      "bedded_couple_nights": 774.0,
      "all_living_couple_nights": 774,
      "bedless_person_nights": 11,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 61,
      "unrelated_unordered_pairs": 1176,
      "excluded_close_kin_pairs": 142,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18014,
      "day60_partnered_adults": 27.0,
      "day60_adults": 47.0,
      "births_from_day20": 143.0,
      "couple_births_from_day20": 143.0,
      "all_births": 166,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 1766.0,
      "bedded_couple_nights": 1766.0,
      "all_living_couple_nights": 1766,
      "bedless_person_nights": 0,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 19.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 51,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 504,
      "unrelated_unordered_pairs": 3157,
      "excluded_close_kin_pairs": 335,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18015,
      "day60_partnered_adults": 12.0,
      "day60_adults": 28.0,
      "births_from_day20": 65.0,
      "couple_births_from_day20": 65.0,
      "all_births": 74,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 943.0,
      "bedded_couple_nights": 943.0,
      "all_living_couple_nights": 943,
      "bedless_person_nights": 2,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 69,
      "unrelated_unordered_pairs": 1792,
      "excluded_close_kin_pairs": 165,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18016,
      "day60_partnered_adults": 9.0,
      "day60_adults": 27.0,
      "births_from_day20": 107.0,
      "couple_births_from_day20": 107.0,
      "all_births": 116,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 1445.0,
      "bedded_couple_nights": 1445.0,
      "all_living_couple_nights": 1445,
      "bedless_person_nights": 10,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 6.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 26,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 198,
      "unrelated_unordered_pairs": 2620,
      "excluded_close_kin_pairs": 240,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18017,
      "day60_partnered_adults": 26.0,
      "day60_adults": 52.0,
      "births_from_day20": 118.0,
      "couple_births_from_day20": 118.0,
      "all_births": 168,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 2030.0,
      "bedded_couple_nights": 2046.0,
      "all_living_couple_nights": 2046,
      "bedless_person_nights": 0,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 5.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 36,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 397,
      "unrelated_unordered_pairs": 4697,
      "excluded_close_kin_pairs": 339,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18018,
      "day60_partnered_adults": 33.0,
      "day60_adults": 70.0,
      "births_from_day20": 205.0,
      "couple_births_from_day20": 205.0,
      "all_births": 215,
      "all_fallback_births": 4,
      "same_shelter_couple_nights": 2304.0,
      "bedded_couple_nights": 2324.0,
      "all_living_couple_nights": 2324,
      "bedless_person_nights": 11,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 19.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 61,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 390,
      "unrelated_unordered_pairs": 5493,
      "excluded_close_kin_pairs": 441,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18019,
      "day60_partnered_adults": 12.0,
      "day60_adults": 24.0,
      "births_from_day20": 129.0,
      "couple_births_from_day20": 125.0,
      "all_births": 157,
      "all_fallback_births": 15,
      "same_shelter_couple_nights": 1710.0,
      "bedded_couple_nights": 1710.0,
      "all_living_couple_nights": 1710,
      "bedless_person_nights": 3,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 24.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 61,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 275,
      "unrelated_unordered_pairs": 4051,
      "excluded_close_kin_pairs": 374,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18020,
      "day60_partnered_adults": 14.0,
      "day60_adults": 23.0,
      "births_from_day20": 49.0,
      "couple_births_from_day20": 49.0,
      "all_births": 50,
      "all_fallback_births": 0,
      "same_shelter_couple_nights": 647.0,
      "bedded_couple_nights": 647.0,
      "all_living_couple_nights": 647,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 128,
      "unrelated_unordered_pairs": 589,
      "excluded_close_kin_pairs": 100,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18021,
      "day60_partnered_adults": 15.0,
      "day60_adults": 24.0,
      "births_from_day20": 106.0,
      "couple_births_from_day20": 106.0,
      "all_births": 121,
      "all_fallback_births": 4,
      "same_shelter_couple_nights": 1296.0,
      "bedded_couple_nights": 1296.0,
      "all_living_couple_nights": 1296,
      "bedless_person_nights": 5,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 10.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 24,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 191,
      "unrelated_unordered_pairs": 2656,
      "excluded_close_kin_pairs": 271,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18022,
      "day60_partnered_adults": 13.0,
      "day60_adults": 22.0,
      "births_from_day20": 101.0,
      "couple_births_from_day20": 100.0,
      "all_births": 122,
      "all_fallback_births": 10,
      "same_shelter_couple_nights": 1346.0,
      "bedded_couple_nights": 1349.0,
      "all_living_couple_nights": 1349,
      "bedless_person_nights": 15,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 10.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 25,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 228,
      "unrelated_unordered_pairs": 2886,
      "excluded_close_kin_pairs": 246,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18023,
      "day60_partnered_adults": 27.0,
      "day60_adults": 55.0,
      "births_from_day20": 166.0,
      "couple_births_from_day20": 156.0,
      "all_births": 189,
      "all_fallback_births": 15,
      "same_shelter_couple_nights": 2020.0,
      "bedded_couple_nights": 2020.0,
      "all_living_couple_nights": 2020,
      "bedless_person_nights": 21,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 19.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 64,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 321,
      "unrelated_unordered_pairs": 5468,
      "excluded_close_kin_pairs": 423,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ],
        [
          "child",
          "elder"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18024,
      "day60_partnered_adults": 24.0,
      "day60_adults": 50.0,
      "births_from_day20": 147.0,
      "couple_births_from_day20": 147.0,
      "all_births": 170,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 1750.0,
      "bedded_couple_nights": 1756.0,
      "all_living_couple_nights": 1759,
      "bedless_person_nights": 31,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 23.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 61,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 294,
      "unrelated_unordered_pairs": 3832,
      "excluded_close_kin_pairs": 402,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18025,
      "day60_partnered_adults": 12.0,
      "day60_adults": 23.0,
      "births_from_day20": 50.0,
      "couple_births_from_day20": 50.0,
      "all_births": 51,
      "all_fallback_births": 1,
      "same_shelter_couple_nights": 683.0,
      "bedded_couple_nights": 683.0,
      "all_living_couple_nights": 683,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 45,
      "unrelated_unordered_pairs": 746,
      "excluded_close_kin_pairs": 102,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18026,
      "day60_partnered_adults": 7.0,
      "day60_adults": 23.0,
      "births_from_day20": 119.0,
      "couple_births_from_day20": 119.0,
      "all_births": 133,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 1504.0,
      "bedded_couple_nights": 1522.0,
      "all_living_couple_nights": 1522,
      "bedless_person_nights": 13,
      "completed_fissions": 2,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 22.0,
      "split_couples": 4.0,
      "completed_departure_households_kept": 54,
      "completed_departure_households_split": 4,
      "sibling_unordered_pairs": 234,
      "unrelated_unordered_pairs": 2805,
      "excluded_close_kin_pairs": 275,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18027,
      "day60_partnered_adults": 13.0,
      "day60_adults": 27.0,
      "births_from_day20": 115.0,
      "couple_births_from_day20": 115.0,
      "all_births": 134,
      "all_fallback_births": 13,
      "same_shelter_couple_nights": 1129.0,
      "bedded_couple_nights": 1129.0,
      "all_living_couple_nights": 1129,
      "bedless_person_nights": 19,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 9.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 31,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 322,
      "unrelated_unordered_pairs": 3656,
      "excluded_close_kin_pairs": 273,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18028,
      "day60_partnered_adults": 9.0,
      "day60_adults": 20.0,
      "births_from_day20": 44.0,
      "couple_births_from_day20": 44.0,
      "all_births": 47,
      "all_fallback_births": 2,
      "same_shelter_couple_nights": 651.0,
      "bedded_couple_nights": 651.0,
      "all_living_couple_nights": 651,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 41,
      "unrelated_unordered_pairs": 653,
      "excluded_close_kin_pairs": 100,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18029,
      "day60_partnered_adults": 12.0,
      "day60_adults": 24.0,
      "births_from_day20": 77.0,
      "couple_births_from_day20": 77.0,
      "all_births": 94,
      "all_fallback_births": 5,
      "same_shelter_couple_nights": 1041.0,
      "bedded_couple_nights": 1046.0,
      "all_living_couple_nights": 1048,
      "bedless_person_nights": 8,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 11.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 23,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 86,
      "unrelated_unordered_pairs": 2327,
      "excluded_close_kin_pairs": 199,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18030,
      "day60_partnered_adults": 11.0,
      "day60_adults": 19.0,
      "births_from_day20": 53.0,
      "couple_births_from_day20": 53.0,
      "all_births": 69,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 934.0,
      "bedded_couple_nights": 934.0,
      "all_living_couple_nights": 934,
      "bedless_person_nights": 2,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 85,
      "unrelated_unordered_pairs": 1544,
      "excluded_close_kin_pairs": 148,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    },
    {
      "seed": 18031,
      "day60_partnered_adults": 16.0,
      "day60_adults": 28.0,
      "births_from_day20": 62.0,
      "couple_births_from_day20": 62.0,
      "all_births": 66,
      "all_fallback_births": 2,
      "same_shelter_couple_nights": 906.0,
      "bedded_couple_nights": 906.0,
      "all_living_couple_nights": 906,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 62,
      "unrelated_unordered_pairs": 1290,
      "excluded_close_kin_pairs": 137,
      "stage_pair_categories": [
        [
          "adult",
          "child"
        ],
        [
          "child",
          "child"
        ]
      ],
      "f3_unassigned_person_hours": 0,
      "f3_unassigned_people": 0
    }
  ]
}
```

## identity

```json
{
  "verdict": "PASS",
  "source": "/Users/rexxa/work/new-world-pass4/docs/revitalize/verification/p17-5_b_identity/identity.json",
  "sha256": "6b627b091095b84f1f207ddadd77706ce068dc1ef431f18edf60bb7d83c7d7a4",
  "payload": {
    "schema_version": 1,
    "stage": "b",
    "verdict": "PASS",
    "passed": true,
    "errors": [],
    "baseline_commit": "adfdecfece79ab019964729319389fd602685fab",
    "candidate_commit": "47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21",
    "stage_a_commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
    "stage_a_verdict_commit": "44aebafa5acf5668d21418f611d8f0ce3a5d13d3",
    "checks": {
      "I1": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i1_v19/summary.json",
            "sha256": "09a66a9b3bda8cb410eff73a7641237e7c85b2c8947f06f926759dc440843984"
          },
          {
            "path": "../../probe/p17-5_b_identity/i1_v19/summary.json",
            "sha256": "1168bd5b778d94818373b624ffd2ab6334b93eaadbbacf0ee0d9389a810da1a8"
          },
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "I1.txt",
            "sha256": "09de48fb3647a11aaaeb2376ba5303fffa9d4f3a033c09cae7b910a9db238374"
          },
          {
            "path": "I1.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "I3": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i3_v16/summary.json",
            "sha256": "efed1ea8d506a023d2d6cc5c3e8d5832f3dfb48e1432a7ac699965dbe97ef6e1"
          },
          {
            "path": "../../probe/p17-5_b_identity/i3_v16/summary.json",
            "sha256": "ba9f25faeab1cd33bf14366d13cb28ffcb32ac449a15dc857d21010a7503e368"
          },
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "I3.txt",
            "sha256": "09de48fb3647a11aaaeb2376ba5303fffa9d4f3a033c09cae7b910a9db238374"
          },
          {
            "path": "I3.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "I4": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i4_v20/summary.json",
            "sha256": "95102ee69f4cde2b4d09cf2be71a3ba1f3d3fd4d554a91eae8773b6093cb983c"
          },
          {
            "path": "../../probe/p17-5_b_identity/i4_v20/summary.json",
            "sha256": "c4321ed4d14f2a09d1f97c76b40f175d5bc42c7b7a4dd6507e28d62bbcc9b540"
          },
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "I4.txt",
            "sha256": "09de48fb3647a11aaaeb2376ba5303fffa9d4f3a033c09cae7b910a9db238374"
          },
          {
            "path": "I4.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "I2": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i2_sha256.txt",
            "sha256": "1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84"
          },
          {
            "path": "../../probe/p17-5_b_identity/i2_sha256.txt",
            "sha256": "1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84"
          },
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "I2.txt",
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
          },
          {
            "path": "I2.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "I5": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-a_eval/i5_v21/summary.json",
            "sha256": "a11aa8bfec756179865fa35ff8f040d7b933af241a93b1beaa0a1806af4ed587"
          },
          {
            "path": "../../probe/p17-5_b_identity/i5_v21/summary.json",
            "sha256": "31725a542c48c06214d44590a2af95491799b9d8d163a3ec09e2370653177a68"
          },
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "I5.txt",
            "sha256": "09de48fb3647a11aaaeb2376ba5303fffa9d4f3a033c09cae7b910a9db238374"
          },
          {
            "path": "I5.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "v19_pins": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "v19_pins.txt",
            "sha256": "01290a60b5e5a570f0c041b097c8e8b37181216c078ee4b7ca39b3e2c2229eb7"
          },
          {
            "path": "v19_pins.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      },
      "v20_pins": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "source_commit.txt",
            "sha256": "61e740891458edbfd6fe0b973905ece143136bee02134eee967a370c21f9fec4"
          },
          {
            "path": "binaries.sha256",
            "sha256": "213933e053dcc06e4c8a2f000a20edfd11d9a295e288eba4be3e38b39c15879c"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "b13a3b971b4e92028af1a61a087a70ce3ec1457986a1458d7142fd8b3f07cdea"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "daaaec418dc465ec6179f898f7b901d025638e8af082f4b3d49542897820ca02"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "8957b481229f89ab40318a0f745a72c74810c27b5943d5078ed3f13f4020f186"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "0b4a2e822e1d4010b556e4a9c2c06aa2af1b5bc6a266c8868d4efd7c46012a9e"
          },
          {
            "path": "launch_receipt.json",
            "sha256": "1aab320f41934cf855c8645ef78e6b2468caa4e156b3467e0b295bfef7786114"
          },
          {
            "path": "v20_pins.txt",
            "sha256": "f679b282b7e048bcd48ce5a6716b24d1c6f432e4cf44bfec6fe5348384a84b23"
          },
          {
            "path": "v20_pins.exit",
            "sha256": "9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa"
          }
        ]
      }
    }
  },
  "error": null
}
```

## performance

```json
{
  "verdict": "PASS",
  "source": "/Users/rexxa/work/new-world-pass4/docs/revitalize/perf/p17-b/perf.json",
  "sha256": "2338579e7538b790e686b2ba97d60176a50ddd00695c18e6942a9b3d91e61e3b",
  "payload": {
    "schema_version": 1,
    "stage": "b",
    "verdict": "PASS",
    "passed": true,
    "errors": [],
    "checks": {
      "mean_relative": {
        "value": 0.01202441891564071,
        "limit": 0.15,
        "operator": "<=",
        "passed": true
      },
      "mean_absolute": {
        "value": 0.4716239130434782,
        "limit": 4.0,
        "operator": "<=",
        "passed": true
      },
      "p999_relative": {
        "value": 0.020448289421942586,
        "limit": 0.35,
        "operator": "<=",
        "passed": true
      },
      "slope": {
        "value": 0.9881343412222895,
        "limit": 1.7,
        "operator": "<",
        "passed": true
      }
    },
    "repeats": {
      "old": [
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "0d09c30b7ce0f3f16f904f880b8278b976592b7cbec89ab0eddcee8dbe127fc8",
          "mean_ms": 0.4706,
          "p999_ms": 2.583,
          "max_ms": 2.834,
          "alive": 500.0,
          "decision_us": 344.1830746376812,
          "decision_share": 0.731370749336339,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "6075e93117b297a4212fde93ad8691df21e8fa2dc91f1ead1ec6b6f2d5ce923b",
          "mean_ms": 0.466018115942029,
          "p999_ms": 2.539,
          "max_ms": 3.071,
          "alive": 500.0,
          "decision_us": 341.061734057971,
          "decision_share": 0.7318636816694007,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "6f2a8df10e2859db0ba1c1fb7a1de3eb56860f19b2c2006320da972052f00bc0",
          "mean_ms": 0.4660202898550725,
          "p999_ms": 2.543,
          "max_ms": 3.024,
          "alive": 500.0,
          "decision_us": 340.9403079710145,
          "decision_share": 0.7315997079806191,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ],
      "new": [
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r1/stress_stress_v22_s15100_n500_side64_fed.json",
          "sha256": "8e261b07d9573e17817bf527df9ce5790e717fdaf3428b713760f186e7d150a9",
          "mean_ms": 0.47577971014492754,
          "p999_ms": 2.595,
          "max_ms": 3.137,
          "alive": 500.0,
          "decision_us": 347.4572173913043,
          "decision_share": 0.7302901111219418,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r2/stress_stress_v22_s15100_n500_side64_fed.json",
          "sha256": "5287416a758f9ed2ecdee43f9afd7fb919cb0f994a1a028167c7cda4306f43b1",
          "mean_ms": 0.47132246376811593,
          "p999_ms": 2.624,
          "max_ms": 3.07,
          "alive": 500.0,
          "decision_us": 344.10375217391305,
          "decision_share": 0.730081374485913,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r3/stress_stress_v22_s15100_n500_side64_fed.json",
          "sha256": "74aeec136018dcf4c4d96a4d993e9b5f19393bd337d266bc3c86ac947c96ea87",
          "mean_ms": 0.4716239130434782,
          "p999_ms": 2.56,
          "max_ms": 3.076,
          "alive": 500.0,
          "decision_us": 344.49191231884055,
          "decision_share": 0.7304377551506436,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ]
    },
    "medians": {
      "old": {
        "mean_ms": 0.4660202898550725,
        "p999_ms": 2.543
      },
      "new": {
        "mean_ms": 0.4716239130434782,
        "p999_ms": 2.595
      }
    },
    "slopes": {
      "old": {
        "b": 0.9885316536105887,
        "a": -0.19856894975199513,
        "n": 3,
        "se": 0.047107594848563986,
        "ci": [
          0.389972908835968,
          1.5870903983852094
        ],
        "r2": 0.997734231292559
      },
      "new": {
        "b": 0.9881343412222895,
        "a": -0.17413654248912458,
        "n": 3,
        "se": 0.04969259234051691,
        "ci": [
          0.356730089072417,
          1.619538593372162
        ],
        "r2": 0.9974773652695818
      },
      "reference": 1.06,
      "old_points": [
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
          "sha256": "709e9d2d3dc93c85a32c9d6d8ce6d0a9f68939825750e0c12dbf5fd52f7bca35",
          "mean_ms": 0.11312765957446809,
          "p999_ms": 0.265,
          "max_ms": 0.33,
          "alive": 100.0,
          "decision_us": 79.25254113475178,
          "decision_share": 0.7005584790922199,
          "fingerprint": "bea6a0b9ada4edc7",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "74f12df1bb3780d2724776460bc3156832077a05831345f3b5650b58d13dd90b",
          "mean_ms": 0.49170709219858155,
          "p999_ms": 2.714,
          "max_ms": 3.067,
          "alive": 500.0,
          "decision_us": 358.57689290780144,
          "decision_share": 0.7292489748408714,
          "fingerprint": "7ce50f96939b36b6",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
          "sha256": "701d0ede899ffaf0a115225d22134d4e6fe294dc8e896434b940ef8d56fe9600",
          "mean_ms": 1.0567570921985816,
          "p999_ms": 5.479,
          "max_ms": 6.308,
          "alive": 1000.0,
          "decision_us": 791.3486322695036,
          "decision_share": 0.7488462941120214,
          "fingerprint": "fbf486f2e08c29ff",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ],
      "new_points": [
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json",
          "sha256": "5f29476907a3fce6495276a60a54631ad3febb05e7aa5c5d33f26cfea5409b79",
          "mean_ms": 0.11577695035460993,
          "p999_ms": 0.26,
          "max_ms": 0.343,
          "alive": 100.0,
          "decision_us": 81.14816489361702,
          "decision_share": 0.70090086709894,
          "fingerprint": "bea6a0b9ada4edc7",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json",
          "sha256": "f383fbf91ca3cdf9bb15422dc2c3932ec456664a52da493545636ec0c5d3d019",
          "mean_ms": 0.49998156028368795,
          "p999_ms": 2.734,
          "max_ms": 3.162,
          "alive": 500.0,
          "decision_us": 365.28208758865253,
          "decision_share": 0.7305911189632526,
          "fingerprint": "7ce50f96939b36b6",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json",
          "sha256": "5acfd9c264603adbb124edb99aba44c01c8495c2d1dcaf0b434f9cdb40023cec",
          "mean_ms": 1.0780354609929077,
          "p999_ms": 5.566,
          "max_ms": 6.484,
          "alive": 1000.0,
          "decision_us": 810.643595035461,
          "decision_share": 0.7519637566363822,
          "fingerprint": "fbf486f2e08c29ff",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ]
    },
    "quick": {
      "base": {
        "file": "/Users/rexxa/work/p17-data/b-perf-final/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
        "sha256": "e588dae8b2bcd84859625d91feb08c3981f1aa701387d2853521381d39461810",
        "mean_ms": 0.3039905797101449,
        "p999_ms": 0.615,
        "max_ms": 0.811,
        "alive": 500.0,
        "decision_us": 184.16406594202897,
        "decision_share": 0.6058216215700811,
        "fingerprint": "54ac4928a770f238",
        "machine": {
          "cpu": "Apple M4 Max",
          "memsize": "38654705664",
          "ncpu": "14",
          "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
          "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
        }
      },
      "final": {
        "file": "/Users/rexxa/work/p17-data/b-perf-final/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
        "sha256": "28c0177d7395ffcf63c0ca01c37b98af4f6dbdc7545f9be16bddc06d761fc34d",
        "mean_ms": 0.3055442028985507,
        "p999_ms": 0.588,
        "max_ms": 0.832,
        "alive": 500.0,
        "decision_us": 186.8194956521739,
        "decision_share": 0.6114319757334857,
        "fingerprint": "54ac4928a770f238",
        "machine": {
          "cpu": "Apple M4 Max",
          "memsize": "38654705664",
          "ncpu": "14",
          "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
          "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
        }
      }
    },
    "provenance": {
      "schema_version": 1,
      "stage": "b",
      "status": 0,
      "builds": {
        "final": {
          "path": "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
          "commit": "47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21",
          "sha256": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f"
        },
        "baseline": {
          "path": "/Users/rexxa/work/p17-data/stage-a-bin/worldsim_perf",
          "commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
          "sha256": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38"
        }
      },
      "quiet": {
        "owner": "pass17-sim (Codex, new-world-pass4)",
        "pid": "38429",
        "purpose": "Stage B final performance",
        "started": "2026-10-15T10:43:58Z",
        "log": "/Users/rexxa/work/p17-data/b-perf-final/quiet.log",
        "prefix_bytes": 158,
        "prefix_sha256": "97e484d7f68f1808254ba4b3f48f636ba6535ae437281b117565c44d498668cc",
        "contended": false
      },
      "commands": {
        "file": "commands.jsonl",
        "sha256": "d7ff1bcfa3892f83e1cbecf2d483632456a8cb1ed3a1d25fe2a788f61efcdbde"
      },
      "runlog": {
        "file": "RUNLOG.txt",
        "sha256": "636ced85f0dc80259f3e9744bc0a38c19790c4ce4b37226f92f6862c9f6fb15c"
      },
      "indexes": {
        "quick_final/runs.json": "1ec46415d46a8fdea4c3f7d876eb2d119bd0539f25f2b49caf9ad9039941356c",
        "b_game_v21/runs.json": "c4800fd03cc51cf56f461af4feaa9571bbdbb8b295b5e383132883d81376c6a2",
        "b_game_v22/runs.json": "f4438bb15df7a7b64f28fbf91357aa4406e6530a3fcddd000bc1fc479888ce93",
        "quick_base/runs.json": "49bc21e7f9156dd592508a7f07fc042fb25f8c3eb491d2d671d05da9a424fd6f",
        "a/r1/runs.json": "264551da332ad018ee39d83849d758cc558c8a33ef1405afc2932383cb2bc07c",
        "a/r3/runs.json": "544b0a1e95b8aa9d488363bebb4bd590c50001bd291dd17e5bee6049cb3c883a",
        "a/r2/runs.json": "8c5ae0f452097b48760707228a8e729c7afa7cbb2cbcb4573fb83d85ce1db347"
      },
      "manifest_sha256": "8e8193193e62f480355ad2253fff450512e007a176668e8c5f31f3bbfd6c0640",
      "quiet_log_sha256": "9f4118c9a3702485bfc5bfd9cf9c3a7ec05e6585b2f176c7dd22bdc026c0003f",
      "input_directory": "/Users/rexxa/work/p17-data/b-perf-final",
      "command_records": [
        {
          "index": 0,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v21",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r1"
          ],
          "started": 1792061038.784953,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061039.888675,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "0d09c30b7ce0f3f16f904f880b8278b976592b7cbec89ab0eddcee8dbe127fc8"
          }
        },
        {
          "index": 1,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v22",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r1"
          ],
          "started": 1792061039.951224,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061040.721879,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r1/stress_stress_v22_s15100_n500_side64_fed.json": "8e261b07d9573e17817bf527df9ce5790e717fdaf3428b713760f186e7d150a9"
          }
        },
        {
          "index": 2,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v21",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r2"
          ],
          "started": 1792061040.78246,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061041.5382771,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "6075e93117b297a4212fde93ad8691df21e8fa2dc91f1ead1ec6b6f2d5ce923b"
          }
        },
        {
          "index": 3,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v22",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r2"
          ],
          "started": 1792061041.600702,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061042.364175,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r2/stress_stress_v22_s15100_n500_side64_fed.json": "5287416a758f9ed2ecdee43f9afd7fb919cb0f994a1a028167c7cda4306f43b1"
          }
        },
        {
          "index": 4,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v21",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r3"
          ],
          "started": 1792061042.426471,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061043.1825862,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "6f2a8df10e2859db0ba1c1fb7a1de3eb56860f19b2c2006320da972052f00bc0"
          }
        },
        {
          "index": 5,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "500",
            "--side",
            "64",
            "--days",
            "1",
            "--feed",
            "fed",
            "--rules",
            "game_v22",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/a/r3"
          ],
          "started": 1792061043.243872,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061044.008727,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "a/r3/stress_stress_v22_s15100_n500_side64_fed.json": "74aeec136018dcf4c4d96a4d993e9b5f19393bd337d266bc3c86ac947c96ea87"
          }
        },
        {
          "index": 6,
          "build": "baseline",
          "argv": [
            "/Users/rexxa/work/p17-data/stage-a-bin/worldsim_perf",
            "quick",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/quick_base"
          ],
          "started": 1792061044.071317,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792061044.84062,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "e588dae8b2bcd84859625d91feb08c3981f1aa701387d2853521381d39461810"
          }
        },
        {
          "index": 7,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "quick",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/quick_final"
          ],
          "started": 1792061044.903966,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061045.423543,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "28c0177d7395ffcf63c0ca01c37b98af4f6dbdc7545f9be16bddc06d761fc34d"
          }
        },
        {
          "index": 8,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "100,500,1000",
            "--side",
            "64",
            "--days",
            "2",
            "--feed",
            "fed",
            "--rules",
            "game_v22",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22"
          ],
          "started": 1792061045.485229,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061050.47699,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json": "5f29476907a3fce6495276a60a54631ad3febb05e7aa5c5d33f26cfea5409b79",
            "b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json": "f383fbf91ca3cdf9bb15422dc2c3932ec456664a52da493545636ec0c5d3d019",
            "b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json": "5acfd9c264603adbb124edb99aba44c01c8495c2d1dcaf0b434f9cdb40023cec"
          }
        },
        {
          "index": 9,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
            "stress",
            "--seeds",
            "15100..15101",
            "--agents",
            "100,500,1000",
            "--side",
            "64",
            "--days",
            "2",
            "--feed",
            "fed",
            "--rules",
            "game_v21",
            "--out",
            "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21"
          ],
          "started": 1792061050.538188,
          "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "finished": 1792061055.4371798,
          "exit_code": 0,
          "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
          "outputs": {
            "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "709e9d2d3dc93c85a32c9d6d8ce6d0a9f68939825750e0c12dbf5fd52f7bca35",
            "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "74f12df1bb3780d2724776460bc3156832077a05831345f3b5650b58d13dd90b",
            "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "701d0ede899ffaf0a115225d22134d4e6fe294dc8e896434b940ef8d56fe9600"
          }
        }
      ]
    }
  },
  "error": null
}
```
