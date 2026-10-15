# Pass 17 Stage B — FAIL

Mode: tuning.

| Rule | Verdict |
|---|---|
| F1 | PASS |
| F2 | PASS |
| F3 | PASS |
| F4 | PASS |
| F5 | FAIL |
| F6 | PASS |
| F7 | PENDING |
| F8 | PENDING |

| Metric | Candidate | Baseline | Estimate | 95% CI | Valid bootstrap draws | Bar | Verdict |
|---|---:|---:|---:|---|---:|---|---|
| F1 | undefined | undefined | 0.541744 | [0.491602, 0.596395] | 2000/2000 | pooled living mutually partnered adult share at elapsed day60 >= 0.50 | PASS |
| F2 | undefined | undefined | 0.9933 | [0.980391, 1] | 2000/2000 | couple-parent births from elapsed simulation day20 inclusive >= 0.90 | PASS |
| F3 | undefined | undefined | 0.0931514 | [0.0444767, 0.142244] | 2000/2000 | mean four-axis sibling Pearson minus unrelated Pearson >= 0.06; lower CI > 0 | PASS |
| F3-siblings | undefined | undefined | 0.0967146 | [0.0527042, 0.137132] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-ha | undefined | undefined | 0.0981048 | [0.0264473, 0.173982] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-ns | undefined | undefined | 0.0741526 | [0.0304034, 0.125394] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-p | undefined | undefined | 0.104208 | [0.0425386, 0.15345] | 2000/2000 | descriptive reference | PASS |
| F3-siblings-rd | undefined | undefined | 0.110393 | [0.0264477, 0.225952] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated | undefined | undefined | 0.00356325 | [-0.00892466, 0.0137495] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-ha | undefined | undefined | -0.00419865 | [-0.0254401, 0.0173416] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-ns | undefined | undefined | 0.0111941 | [-0.0103155, 0.0339851] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-p | undefined | undefined | 0.00420165 | [-0.0167295, 0.0223649] | 2000/2000 | descriptive reference | PASS |
| F3-unrelated-rd | undefined | undefined | 0.0030559 | [-0.0229623, 0.0309127] | 2000/2000 | descriptive reference | PASS |
| F4 | undefined | undefined | 0.995462 | [0.99021, 0.99944] | 2000/2000 | same-shelter share of living mutual bedded couple-nights at20:00 >= 0.90 | PASS |
| F5 | undefined | undefined | 0.1125 | [0.0549413, 0.180556] | 2000/2000 | split couples / parent couples at completed fission departures <= 0.10 | FAIL |
| F6-alive | 48.1875 | 49.75 | -0.031407 | [-0.106365, 0.0128023] | 2000/2000 | relative change within +/-0.10 | PASS |
| F6-deaths | 0.4375 | 0.125 | 0.3125 | [-0.0625, 0.689062] | 2000/2000 | non-old-age deaths difference <= +1.5/world | PASS |
| F6-food | 389.236 | 392.116 | -0.00734265 | [-0.0211443, 0.00398288] | 2000/2000 | noon food relative change within +/-0.15 | PASS |
| F6-shelters | 4.75 | 4.75 | 1 | [0.9, 1.13238] | 2000/2000 | completed shelters ratio >= 0.90 | PASS |
| F6-social | 1.72958 | 1.7162 | 1.00779 | [0.996172, 1.01851] | 2000/2000 | pooled adult social rate ratio >= 0.90 | PASS |
| F6-wood | 95.9969 | 97.7375 | -0.0178092 | [-0.0704743, 0.0152802] | 2000/2000 | noon wood relative change within +/-0.15 | PASS |

## Methods

- bootstrap: 2,000 whole-world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded with valid count; F6 keeps seed pairs together
- F1: Pooled living nonelder adults at exact elapsed day60 (after tick86399); partner must be alive and mutual; dead partner pointers do not count
- F2: Pooled births with fallback=false from one-based elapsed simulation day20; simulation_day=tick//1440+1, tick>=27360; display-clock offset does not change inclusion; all daily births/fallbacks retained
- F3: Pooled Pearson for each temperament axis, then mean four axes for siblings minus unrelated; each unique unordered pair contributes both orientations; bar>=0.06 and lower95%CI>0; exact per-world integer sufficient statistics preserve whole-world bootstrap
- F3_eligibility: Full siblings are all recorded people with both known matching ordered mother/father, including fallback births, independent of residence or coexistence. Their first shared hourly snapshot in a non-null settlement supplies reference tick/settlement/stages when present; otherwise all three are null, without excluding the pair. Unrelated controls exclude direct parent-child or any shared known parent across either slot, and use their first shared hourly snapshot in the same non-null settlement regardless of life-stage categories. Sibling stage-pair categories describe observed shared context only and never filter either population. Unassigned settlement=null observations cannot form unrelated same-settlement pairs; their counts are reported. Unknown founder kinship is not inferred. Empty worlds remain in resampling; pair IDs/ticks/stages reported
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
    "numerator": 292.0,
    "denominator": 539.0,
    "worlds": 16,
    "zero_denominator_worlds": 0
  },
  "F2": {
    "numerator": 593.0,
    "denominator": 597.0,
    "worlds": 16,
    "zero_denominator_worlds": 0
  },
  "F3": {
    "sibling_unordered_pairs": 1790,
    "unrelated_unordered_pairs": 27653,
    "sibling_worlds": 16,
    "unrelated_worlds": 16,
    "zero_sibling_worlds": 0,
    "zero_unrelated_worlds": 0,
    "excluded_close_kin_pairs": 1921,
    "orientations_per_pair": 2
  },
  "F4": {
    "numerator": 7459.0,
    "denominator": 7493.0,
    "worlds": 16,
    "zero_denominator_worlds": 0
  },
  "F5": {
    "numerator": 9.0,
    "denominator": 80.0,
    "worlds": 16,
    "zero_denominator_worlds": 8
  },
  "F6-shelters": {
    "definition": "final minus initial complete Shelter count, including daughter-camp boot shelters"
  },
  "F6-social": {
    "candidate_mean_world_rate": 1.7171706953659416,
    "baseline_mean_world_rate": 1.7065779275751536,
    "candidate_sensitivity_worlds": 16,
    "baseline_sensitivity_worlds": 16
  }
}
```

## game_v21

16 worlds × 60 days; seeds [18100, 18101, 18102, 18103, 18104, 18105, 18106, 18107, 18108, 18109, 18110, 18111, 18112, 18113, 18114, 18115].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 29373 | 0 | 0.0898104 | 0 | 0.91019 | 0 |
| 0 | child | 3314 | 0 | 0.002414 | 0 | 0.997586 | 0 |
| 0 | elder | 6479 | 0 | 0.00324124 | 0 | 0.996759 | 0 |
| 60 | adult | 29377 | 0.000102121 | 0.049086 | 0 | 0.950812 | 0 |
| 60 | child | 3317 | 0 | 0 | 0 | 1 | 0 |
| 60 | elder | 6467 | 0 | 0.00123705 | 0 | 0.998763 | 0 |
| 120 | adult | 29395 | 0 | 0.0213982 | 0 | 0.978602 | 0 |
| 120 | child | 3321 | 0 | 0 | 0 | 1 | 0 |
| 120 | elder | 6455 | 0 | 0.00526723 | 0 | 0.994733 | 0 |
| 180 | adult | 29401 | 0.000204075 | 0.0815278 | 0 | 0.918268 | 0 |
| 180 | child | 3324 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 6451 | 0.000155015 | 0.0996745 | 0 | 0.900171 | 0 |
| 240 | adult | 29392 | 0.000136091 | 0.096795 | 0 | 0.903069 | 0 |
| 240 | child | 3325 | 0 | 0 | 0 | 1 | 0 |
| 240 | elder | 6447 | 0 | 0.111059 | 0 | 0.888941 | 0 |
| 300 | adult | 29387 | 0.00163338 | 0.142376 | 0 | 0.855991 | 0 |
| 300 | child | 3324 | 0.000300842 | 0.000300842 | 0 | 0.999398 | 0 |
| 300 | elder | 6438 | 0.00248524 | 0.155172 | 0 | 0.842342 | 0 |
| 360 | adult | 29389 | 0.833679 | 0.117731 | 0 | 0.0485896 | 0 |
| 360 | child | 3328 | 0.980168 | 0.0147236 | 0 | 0.00510817 | 0 |
| 360 | elder | 6431 | 0.825999 | 0.126419 | 0 | 0.047582 | 0 |
| 420 | adult | 29415 | 0.157471 | 0.831651 | 0.00873704 | 0.000713921 | 0.00142784 |
| 420 | child | 3333 | 0.873687 | 0.109211 | 0 | 0.00030003 | 0.0168017 |
| 420 | elder | 6429 | 0.503033 | 0.481568 | 0.00948826 | 0.000777726 | 0.00513299 |
| 480 | adult | 29199 | 0.0454468 | 0.69862 | 0.00243159 | 0.0310627 | 0.222439 |
| 480 | child | 3285 | 0.0252664 | 0.0283105 | 0 | 0.00791476 | 0.938508 |
| 480 | elder | 6344 | 0.00252207 | 0.381305 | 0.00331021 | 0.0163934 | 0.596469 |
| 540 | adult | 29280 | 0.0900956 | 0.666667 | 0.00484973 | 0.0016735 | 0.236714 |
| 540 | child | 3286 | 0.122642 | 0.0398661 | 0 | 0 | 0.837492 |
| 540 | elder | 6357 | 0.106811 | 0.336008 | 0.00251691 | 0.000629228 | 0.554035 |
| 600 | adult | 29279 | 0.115953 | 0.629496 | 0.00174186 | 0.0369207 | 0.215889 |
| 600 | child | 3288 | 0.371655 | 0.0276764 | 0 | 0.0100365 | 0.590633 |
| 600 | elder | 6373 | 0.202887 | 0.264397 | 0.00172603 | 0.0241644 | 0.506826 |
| 660 | adult | 29274 | 0.155804 | 0.492587 | 0.00526064 | 0.0015372 | 0.344811 |
| 660 | child | 3290 | 0.17234 | 0.0367781 | 0 | 0 | 0.790881 |
| 660 | elder | 6376 | 0.147271 | 0.0801443 | 0.00250941 | 0 | 0.770075 |
| 720 | adult | 29274 | 0.160655 | 0.293127 | 0.00174216 | 0.0481998 | 0.496277 |
| 720 | child | 3291 | 0.271346 | 0.0382862 | 0 | 0.0103312 | 0.680036 |
| 720 | elder | 6379 | 0.200972 | 0.0727387 | 0.00203794 | 0.0203794 | 0.703872 |
| 780 | adult | 29263 | 0.148857 | 0.354612 | 0.00300721 | 0.000341728 | 0.493183 |
| 780 | child | 3293 | 0.257516 | 0.0324932 | 0 | 0 | 0.709991 |
| 780 | elder | 6389 | 0.214118 | 0.0691814 | 0.000626076 | 0 | 0.716075 |
| 840 | adult | 29274 | 0.127588 | 0.230102 | 0.00181048 | 0.0309831 | 0.609517 |
| 840 | child | 3297 | 0.236579 | 0.0312405 | 0 | 0.0124355 | 0.719745 |
| 840 | elder | 6389 | 0.207857 | 0.0637032 | 0.000782595 | 0.0198779 | 0.707779 |
| 900 | adult | 29278 | 0.126136 | 0.351868 | 0.00399617 | 0 | 0.518 |
| 900 | child | 3300 | 0.234848 | 0.0475758 | 0 | 0.00030303 | 0.717273 |
| 900 | elder | 6385 | 0.22177 | 0.0606108 | 0.00140955 | 0 | 0.71621 |
| 960 | adult | 29275 | 0.117985 | 0.234125 | 0.000922289 | 0.0314261 | 0.615542 |
| 960 | child | 3301 | 0.246895 | 0.044532 | 0 | 0.0118146 | 0.696759 |
| 960 | elder | 6381 | 0.214073 | 0.0708353 | 0.000470146 | 0.0208431 | 0.693778 |
| 1020 | adult | 29274 | 0.118228 | 0.337193 | 0.00317688 | 0 | 0.541402 |
| 1020 | child | 3302 | 0.242883 | 0.0384615 | 0 | 0 | 0.718655 |
| 1020 | elder | 6384 | 0.205201 | 0.0697055 | 0.000783208 | 0 | 0.724311 |
| 1080 | adult | 29270 | 0.725897 | 0.240656 | 0.000819952 | 0.0326273 | 0 |
| 1080 | child | 3304 | 0.955811 | 0.0320823 | 0 | 0.0121065 | 0 |
| 1080 | elder | 6381 | 0.907695 | 0.0695816 | 0.000626861 | 0.0220969 | 0 |
| 1140 | adult | 29285 | 0.565067 | 0.428035 | 0.00686358 | 3.41472e-05 | 0 |
| 1140 | child | 3306 | 0.970659 | 0.0293406 | 0 | 0 | 0 |
| 1140 | elder | 6386 | 0.925775 | 0.0721892 | 0.0020357 | 0 | 0 |
| 1200 | adult | 29280 | 0.677186 | 0.282684 | 0.00372268 | 0.0364071 | 0 |
| 1200 | child | 3306 | 0.954628 | 0.03781 | 0 | 0.00756201 | 0 |
| 1200 | elder | 6468 | 0.905535 | 0.0746753 | 0.00170068 | 0.0180891 | 0 |
| 1260 | adult | 29364 | 0.305953 | 0.605129 | 0.00156654 | 0.0873519 | 0 |
| 1260 | child | 3310 | 0.445921 | 0.344713 | 0 | 0.209366 | 0 |
| 1260 | elder | 6482 | 0.452947 | 0.371799 | 0.00092564 | 0.174329 | 0 |
| 1320 | adult | 29365 | 0 | 0.386719 | 0 | 0.613281 | 0 |
| 1320 | child | 3310 | 0 | 0.0371601 | 0 | 0.96284 | 0 |
| 1320 | elder | 6489 | 0 | 0.0693481 | 0 | 0.930652 | 0 |
| 1380 | adult | 29372 | 3.4046e-05 | 0.197433 | 0 | 0.802533 | 0 |
| 1380 | child | 3312 | 0 | 0.00603865 | 0 | 0.993961 | 0 |
| 1380 | elder | 6480 | 0 | 0.0118827 | 0 | 0.988117 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 892091,
    "adult_minutes": 890942,
    "all_life_stages_share": 0.12958963063314202,
    "adult_share": 0.24735938466527405,
    "all_life_stages_starts": 29753
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 672324,
    "adult_minutes": 671895,
    "all_life_stages_share": 0.09766516961363422,
    "adult_share": 0.18654360638478631,
    "all_life_stages_starts": 23473
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 731721,
    "adult_minutes": 731231,
    "all_life_stages_share": 0.10629347691716799,
    "adult_share": 0.20301753672873543,
    "all_life_stages_starts": 22577
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 2011348,
    "adult_minutes": 985422,
    "all_life_stages_share": 0.2921785382822032,
    "adult_share": 0.27359062605155404,
    "all_life_stages_starts": 52200
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 317371,
    "adult_minutes": 316584,
    "all_life_stages_share": 0.04610290952791914,
    "adult_share": 0.08789575913456893,
    "all_life_stages_starts": 19415
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 1218916,
    "adult_minutes": 5738,
    "all_life_stages_share": 0.177065875805077,
    "adult_share": 0.0015930870350812313,
    "all_life_stages_starts": 16844
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 1040198,
    "adult_minutes": 0,
    "all_life_stages_share": 0.15110439922085644,
    "adult_share": 0.0,
    "all_life_stages_starts": 18820
  }
]
```

### health_by_world

```json
[
  {
    "alive": 75.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 439.23333333333335,
    "wood": 122.58333333333333,
    "seed": 18100,
    "old_age_deaths": 22,
    "social_completed": 3998.0,
    "adult_days": 2243.5416666666665,
    "social_rate": 1.7820039000835732,
    "no_place_redraws": 498,
    "no_place_failures": 0
  },
  {
    "alive": 41.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 422.1666666666667,
    "wood": 83.58333333333333,
    "seed": 18101,
    "old_age_deaths": 22,
    "social_completed": 3061.0,
    "adult_days": 1749.2916666666667,
    "social_rate": 1.7498511302193744,
    "no_place_redraws": 3,
    "no_place_failures": 0
  },
  {
    "alive": 36.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 266.96666666666664,
    "wood": 80.51666666666667,
    "seed": 18102,
    "old_age_deaths": 23,
    "social_completed": 2545.0,
    "adult_days": 1539.875,
    "social_rate": 1.6527315528857862,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 46.0,
    "deaths": 0.0,
    "shelters": 6.0,
    "food": 445.0,
    "wood": 87.76666666666667,
    "seed": 18103,
    "old_age_deaths": 36,
    "social_completed": 3166.0,
    "adult_days": 1886.375,
    "social_rate": 1.6783513352329202,
    "no_place_redraws": 66,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 208.15,
    "wood": 84.76666666666667,
    "seed": 18104,
    "old_age_deaths": 21,
    "social_completed": 1662.0,
    "adult_days": 1007.625,
    "social_rate": 1.6494231484927429,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 49.0,
    "deaths": 0.0,
    "shelters": 4.0,
    "food": 492.15,
    "wood": 85.08333333333333,
    "seed": 18105,
    "old_age_deaths": 23,
    "social_completed": 3616.0,
    "adult_days": 2123.125,
    "social_rate": 1.7031498380924346,
    "no_place_redraws": 40,
    "no_place_failures": 1
  },
  {
    "alive": 22.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 228.1,
    "wood": 73.63333333333334,
    "seed": 18106,
    "old_age_deaths": 22,
    "social_completed": 1669.0,
    "adult_days": 984.5833333333334,
    "social_rate": 1.6951333051206092,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 39.0,
    "deaths": 2.0,
    "shelters": 3.0,
    "food": 333.15,
    "wood": 77.91666666666667,
    "seed": 18107,
    "old_age_deaths": 27,
    "social_completed": 2588.0,
    "adult_days": 1649.1666666666667,
    "social_rate": 1.5692774128347649,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 75.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 383.25,
    "wood": 126.36666666666666,
    "seed": 18108,
    "old_age_deaths": 33,
    "social_completed": 3847.0,
    "adult_days": 2121.5,
    "social_rate": 1.8133396181946735,
    "no_place_redraws": 643,
    "no_place_failures": 0
  },
  {
    "alive": 83.0,
    "deaths": 0.0,
    "shelters": 11.0,
    "food": 523.6166666666667,
    "wood": 120.63333333333334,
    "seed": 18109,
    "old_age_deaths": 34,
    "social_completed": 4455.0,
    "adult_days": 2624.0833333333335,
    "social_rate": 1.6977357172345897,
    "no_place_redraws": 1074,
    "no_place_failures": 0
  },
  {
    "alive": 28.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 299.53333333333336,
    "wood": 74.01666666666667,
    "seed": 18110,
    "old_age_deaths": 24,
    "social_completed": 1967.0,
    "adult_days": 1214.9583333333333,
    "social_rate": 1.6189855619191331,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 62.0,
    "deaths": 0.0,
    "shelters": 6.0,
    "food": 396.55,
    "wood": 137.86666666666667,
    "seed": 18111,
    "old_age_deaths": 28,
    "social_completed": 3282.0,
    "adult_days": 1931.3333333333333,
    "social_rate": 1.6993441491197792,
    "no_place_redraws": 603,
    "no_place_failures": 0
  },
  {
    "alive": 81.0,
    "deaths": 0.0,
    "shelters": 10.0,
    "food": 672.5,
    "wood": 127.25,
    "seed": 18112,
    "old_age_deaths": 28,
    "social_completed": 4972.0,
    "adult_days": 2787.7916666666665,
    "social_rate": 1.7834905166873423,
    "no_place_redraws": 947,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 509.98333333333335,
    "wood": 113.6,
    "seed": 18113,
    "old_age_deaths": 27,
    "social_completed": 3711.0,
    "adult_days": 2171.6666666666665,
    "social_rate": 1.708825786646201,
    "no_place_redraws": 727,
    "no_place_failures": 0
  },
  {
    "alive": 42.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 454.45,
    "wood": 90.15,
    "seed": 18114,
    "old_age_deaths": 25,
    "social_completed": 3352.0,
    "adult_days": 1848.6666666666667,
    "social_rate": 1.8131987017670392,
    "no_place_redraws": 3,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 199.05,
    "wood": 78.06666666666666,
    "seed": 18115,
    "old_age_deaths": 32,
    "social_completed": 2432.0,
    "adult_days": 1438.7083333333333,
    "social_rate": 1.6904051666714937,
    "no_place_redraws": 0,
    "no_place_failures": 0
  }
]
```

## game_v22

16 worlds × 60 days; seeds [18100, 18101, 18102, 18103, 18104, 18105, 18106, 18107, 18108, 18109, 18110, 18111, 18112, 18113, 18114, 18115].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 28820 | 0 | 0.0824427 | 0 | 0.917557 | 0 |
| 0 | child | 3194 | 0 | 0.00156544 | 0 | 0.998435 | 0 |
| 0 | elder | 6479 | 0 | 0.00354993 | 0 | 0.99645 | 0 |
| 60 | adult | 28824 | 3.46933e-05 | 0.0430544 | 0 | 0.956911 | 0 |
| 60 | child | 3197 | 0 | 0 | 0 | 1 | 0 |
| 60 | elder | 6468 | 0 | 0.00108225 | 0 | 0.998918 | 0 |
| 120 | adult | 28842 | 3.46717e-05 | 0.020491 | 0 | 0.979474 | 0 |
| 120 | child | 3200 | 0 | 0 | 0 | 1 | 0 |
| 120 | elder | 6457 | 0 | 0.00542047 | 0 | 0.99458 | 0 |
| 180 | adult | 28850 | 0.000242634 | 0.0814905 | 0 | 0.918267 | 0 |
| 180 | child | 3201 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 6452 | 0.000154991 | 0.0964042 | 0 | 0.903441 | 0 |
| 240 | adult | 28839 | 0.000138701 | 0.0963279 | 0 | 0.903533 | 0 |
| 240 | child | 3204 | 0 | 0 | 0 | 1 | 0 |
| 240 | elder | 6448 | 0 | 0.112748 | 0 | 0.887252 | 0 |
| 300 | adult | 28832 | 0.00131798 | 0.141995 | 0 | 0.856687 | 0 |
| 300 | child | 3199 | 0 | 0.000625195 | 0 | 0.999375 | 0 |
| 300 | elder | 6440 | 0.00263975 | 0.153261 | 0 | 0.844099 | 0 |
| 360 | adult | 28831 | 0.833721 | 0.115882 | 0 | 0.0503971 | 0 |
| 360 | child | 3204 | 0.98377 | 0.0106117 | 0 | 0.00561798 | 0 |
| 360 | elder | 6432 | 0.830535 | 0.122046 | 0 | 0.0474192 | 0 |
| 420 | adult | 28867 | 0.157412 | 0.830291 | 0.0084872 | 0.000554266 | 0.00325631 |
| 420 | child | 3211 | 0.855808 | 0.123949 | 0 | 0.000311429 | 0.0199315 |
| 420 | elder | 6430 | 0.540591 | 0.444168 | 0.00762053 | 0.000311042 | 0.00730949 |
| 480 | adult | 28689 | 0.0520409 | 0.684339 | 0.00247482 | 0.0334623 | 0.227683 |
| 480 | child | 3148 | 0.0266836 | 0.0260483 | 0 | 0.0111182 | 0.93615 |
| 480 | elder | 6345 | 0.00441292 | 0.352088 | 0.00267928 | 0.012766 | 0.628054 |
| 540 | adult | 28744 | 0.0923323 | 0.651092 | 0.00504453 | 0.00153075 | 0.25 |
| 540 | child | 3155 | 0.14897 | 0.0453249 | 0 | 0 | 0.805705 |
| 540 | elder | 6358 | 0.10994 | 0.308116 | 0.00298836 | 0.000471846 | 0.578484 |
| 600 | adult | 28744 | 0.112371 | 0.630427 | 0.00247008 | 0.0356248 | 0.219107 |
| 600 | child | 3158 | 0.366054 | 0.0341989 | 0 | 0.00854972 | 0.591197 |
| 600 | elder | 6373 | 0.209791 | 0.249961 | 0.00172603 | 0.0208693 | 0.517653 |
| 660 | adult | 28741 | 0.152187 | 0.495216 | 0.00462754 | 0.00167009 | 0.3463 |
| 660 | child | 3161 | 0.178425 | 0.032901 | 0 | 0 | 0.788674 |
| 660 | elder | 6377 | 0.147248 | 0.080759 | 0.00141132 | 0 | 0.770582 |
| 720 | adult | 28742 | 0.161715 | 0.298066 | 0.00167003 | 0.0462737 | 0.492276 |
| 720 | child | 3162 | 0.256483 | 0.0452245 | 0 | 0.0110689 | 0.687223 |
| 720 | elder | 6380 | 0.203762 | 0.073511 | 0.00125392 | 0.0178683 | 0.703605 |
| 780 | adult | 28729 | 0.150684 | 0.351874 | 0.00368965 | 0.000208848 | 0.493543 |
| 780 | child | 3165 | 0.259084 | 0.0296998 | 0 | 0 | 0.711216 |
| 780 | elder | 6390 | 0.212363 | 0.0640063 | 0.00156495 | 0 | 0.722066 |
| 840 | adult | 28739 | 0.126309 | 0.232472 | 0.000800306 | 0.0314207 | 0.608998 |
| 840 | child | 3168 | 0.224116 | 0.0299874 | 0 | 0.00599747 | 0.739899 |
| 840 | elder | 6391 | 0.214051 | 0.0633704 | 0.00109529 | 0.0208105 | 0.700673 |
| 900 | adult | 28744 | 0.127818 | 0.346577 | 0.00327025 | 0.000139159 | 0.522196 |
| 900 | child | 3169 | 0.246134 | 0.0426002 | 0 | 0 | 0.711265 |
| 900 | elder | 6387 | 0.206357 | 0.0573039 | 0.000626272 | 0 | 0.735713 |
| 960 | adult | 28740 | 0.117363 | 0.242136 | 0.0013222 | 0.0309325 | 0.608246 |
| 960 | child | 3174 | 0.232829 | 0.0478891 | 0 | 0.0148078 | 0.704474 |
| 960 | elder | 6383 | 0.215259 | 0.0667398 | 0.000626665 | 0.0225599 | 0.694814 |
| 1020 | adult | 28737 | 0.118871 | 0.33048 | 0.00292306 | 0 | 0.547726 |
| 1020 | child | 3175 | 0.249134 | 0.0349606 | 0 | 0 | 0.715906 |
| 1020 | elder | 6385 | 0.206891 | 0.0725137 | 0.00125294 | 0 | 0.719342 |
| 1080 | adult | 28731 | 0.731231 | 0.236226 | 0.000696112 | 0.0318471 | 0 |
| 1080 | child | 3177 | 0.951841 | 0.0352534 | 0 | 0.0129053 | 0 |
| 1080 | elder | 6382 | 0.91147 | 0.0691006 | 0.000940144 | 0.0184895 | 0 |
| 1140 | adult | 28751 | 0.568502 | 0.423255 | 0.00817363 | 6.95628e-05 | 0 |
| 1140 | child | 3181 | 0.969821 | 0.0301792 | 0 | 0 | 0 |
| 1140 | elder | 6387 | 0.931893 | 0.066698 | 0.00140911 | 0 | 0 |
| 1200 | adult | 28745 | 0.670899 | 0.288085 | 0.00320056 | 0.0378153 | 0 |
| 1200 | child | 3181 | 0.96039 | 0.0308079 | 0 | 0.00880226 | 0 |
| 1200 | elder | 6469 | 0.91096 | 0.0698717 | 0.00340083 | 0.0157675 | 0 |
| 1260 | adult | 28807 | 0.323741 | 0.588364 | 0.00104141 | 0.0868539 | 0 |
| 1260 | child | 3188 | 0.465496 | 0.309285 | 0 | 0.22522 | 0 |
| 1260 | elder | 6483 | 0.46861 | 0.357551 | 0.000925497 | 0.172914 | 0 |
| 1320 | adult | 28810 | 0 | 0.389344 | 0 | 0.610656 | 0 |
| 1320 | child | 3190 | 0 | 0.0407524 | 0 | 0.959248 | 0 |
| 1320 | elder | 6490 | 0 | 0.0719569 | 0 | 0.928043 | 0 |
| 1380 | adult | 28819 | 0 | 0.197647 | 0 | 0.802353 | 0 |
| 1380 | child | 3192 | 0 | 0.00814536 | 0 | 0.991855 | 0 |
| 1380 | elder | 6480 | 0 | 0.0131173 | 0 | 0.986883 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 859519,
    "adult_minutes": 858870,
    "all_life_stages_share": 0.12628179803087025,
    "adult_share": 0.24224508025618122,
    "all_life_stages_starts": 28952
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 672872,
    "adult_minutes": 672079,
    "all_life_stages_share": 0.09885934575573982,
    "adult_share": 0.18956050542398037,
    "all_life_stages_starts": 23549
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 720632,
    "adult_minutes": 719982,
    "all_life_stages_share": 0.10587631533285721,
    "adult_share": 0.20307159101261643,
    "all_life_stages_starts": 22374
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 2015750,
    "adult_minutes": 966869,
    "all_life_stages_share": 0.29615696032400296,
    "adult_share": 0.2727062983946507,
    "all_life_stages_starts": 52271
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 322039,
    "adult_minutes": 321395,
    "all_life_stages_share": 0.04731444442305921,
    "adult_share": 0.09064975790158622,
    "all_life_stages_starts": 19314
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 1170433,
    "adult_minutes": 6264,
    "all_life_stages_share": 0.17196174106059967,
    "adult_share": 0.0017667670109850376,
    "all_life_stages_starts": 16223
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 1045112,
    "adult_minutes": 0,
    "all_life_stages_share": 0.15354939507287085,
    "adult_share": 0.0,
    "all_life_stages_starts": 18922
  }
]
```

### health_by_world

```json
[
  {
    "alive": 75.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 439.8333333333333,
    "wood": 129.21666666666667,
    "seed": 18100,
    "old_age_deaths": 23,
    "social_completed": 4133.0,
    "adult_days": 2243.2916666666665,
    "social_rate": 1.842381916454615,
    "no_place_redraws": 542,
    "no_place_failures": 0
  },
  {
    "alive": 41.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 423.31666666666666,
    "wood": 84.43333333333334,
    "seed": 18101,
    "old_age_deaths": 22,
    "social_completed": 3032.0,
    "adult_days": 1749.2916666666667,
    "social_rate": 1.7332729914489198,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 36.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 261.4166666666667,
    "wood": 76.71666666666667,
    "seed": 18102,
    "old_age_deaths": 24,
    "social_completed": 2527.0,
    "adult_days": 1539.875,
    "social_rate": 1.6410422923938632,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 48.0,
    "deaths": 0.0,
    "shelters": 6.0,
    "food": 442.2,
    "wood": 88.0,
    "seed": 18103,
    "old_age_deaths": 36,
    "social_completed": 3183.0,
    "adult_days": 1886.375,
    "social_rate": 1.6873633291365715,
    "no_place_redraws": 54,
    "no_place_failures": 0
  },
  {
    "alive": 23.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 209.21666666666667,
    "wood": 85.95,
    "seed": 18104,
    "old_age_deaths": 21,
    "social_completed": 1615.0,
    "adult_days": 1007.625,
    "social_rate": 1.6027788115618409,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 49.0,
    "deaths": 2.0,
    "shelters": 4.0,
    "food": 488.45,
    "wood": 84.75,
    "seed": 18105,
    "old_age_deaths": 24,
    "social_completed": 3709.0,
    "adult_days": 2116.875,
    "social_rate": 1.7521110126956008,
    "no_place_redraws": 73,
    "no_place_failures": 2
  },
  {
    "alive": 22.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 226.71666666666667,
    "wood": 72.6,
    "seed": 18106,
    "old_age_deaths": 22,
    "social_completed": 1620.0,
    "adult_days": 982.125,
    "social_rate": 1.6494845360824741,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 41.0,
    "deaths": 1.0,
    "shelters": 6.0,
    "food": 325.23333333333335,
    "wood": 78.56666666666666,
    "seed": 18107,
    "old_age_deaths": 27,
    "social_completed": 2725.0,
    "adult_days": 1653.4583333333333,
    "social_rate": 1.6480608824937633,
    "no_place_redraws": 6,
    "no_place_failures": 0
  },
  {
    "alive": 45.0,
    "deaths": 1.0,
    "shelters": 5.0,
    "food": 346.96666666666664,
    "wood": 88.2,
    "seed": 18108,
    "old_age_deaths": 34,
    "social_completed": 2798.0,
    "adult_days": 1600.7916666666667,
    "social_rate": 1.7478851609880528,
    "no_place_redraws": 41,
    "no_place_failures": 0
  },
  {
    "alive": 83.0,
    "deaths": 0.0,
    "shelters": 11.0,
    "food": 532.6666666666666,
    "wood": 122.86666666666666,
    "seed": 18109,
    "old_age_deaths": 35,
    "social_completed": 4473.0,
    "adult_days": 2632.4583333333335,
    "social_rate": 1.6991721932920747,
    "no_place_redraws": 1147,
    "no_place_failures": 0
  },
  {
    "alive": 28.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 298.6666666666667,
    "wood": 73.88333333333334,
    "seed": 18110,
    "old_age_deaths": 23,
    "social_completed": 2026.0,
    "adult_days": 1214.9583333333333,
    "social_rate": 1.6675468980417711,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 62.0,
    "deaths": 0.0,
    "shelters": 6.0,
    "food": 395.5833333333333,
    "wood": 138.35,
    "seed": 18111,
    "old_age_deaths": 29,
    "social_completed": 3367.0,
    "adult_days": 1927.625,
    "social_rate": 1.746709033136632,
    "no_place_redraws": 611,
    "no_place_failures": 0
  },
  {
    "alive": 81.0,
    "deaths": 2.0,
    "shelters": 10.0,
    "food": 666.05,
    "wood": 128.68333333333334,
    "seed": 18112,
    "old_age_deaths": 30,
    "social_completed": 5034.0,
    "adult_days": 2765.375,
    "social_rate": 1.8203679428648918,
    "no_place_redraws": 969,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 515.1166666666667,
    "wood": 113.21666666666667,
    "seed": 18113,
    "old_age_deaths": 27,
    "social_completed": 3686.0,
    "adult_days": 2172.25,
    "social_rate": 1.6968580964437794,
    "no_place_redraws": 758,
    "no_place_failures": 0
  },
  {
    "alive": 43.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 445.4166666666667,
    "wood": 90.98333333333333,
    "seed": 18114,
    "old_age_deaths": 21,
    "social_completed": 3410.0,
    "adult_days": 1848.6666666666667,
    "social_rate": 1.844572664983772,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 210.93333333333334,
    "wood": 79.53333333333333,
    "seed": 18115,
    "old_age_deaths": 32,
    "social_completed": 2439.0,
    "adult_days": 1438.8333333333333,
    "social_rate": 1.6951233638364416,
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
      "births": 16,
      "couple_births": 0,
      "fallback_births": 16
    },
    {
      "day": 2,
      "births": 29,
      "couple_births": 0,
      "fallback_births": 29
    },
    {
      "day": 3,
      "births": 26,
      "couple_births": 0,
      "fallback_births": 26
    },
    {
      "day": 4,
      "births": 26,
      "couple_births": 6,
      "fallback_births": 20
    },
    {
      "day": 5,
      "births": 23,
      "couple_births": 14,
      "fallback_births": 9
    },
    {
      "day": 6,
      "births": 21,
      "couple_births": 19,
      "fallback_births": 2
    },
    {
      "day": 7,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 8,
      "births": 18,
      "couple_births": 18,
      "fallback_births": 0
    },
    {
      "day": 9,
      "births": 15,
      "couple_births": 15,
      "fallback_births": 0
    },
    {
      "day": 10,
      "births": 11,
      "couple_births": 11,
      "fallback_births": 0
    },
    {
      "day": 11,
      "births": 10,
      "couple_births": 10,
      "fallback_births": 0
    },
    {
      "day": 12,
      "births": 6,
      "couple_births": 6,
      "fallback_births": 0
    },
    {
      "day": 13,
      "births": 3,
      "couple_births": 3,
      "fallback_births": 0
    },
    {
      "day": 14,
      "births": 2,
      "couple_births": 2,
      "fallback_births": 0
    },
    {
      "day": 15,
      "births": 2,
      "couple_births": 2,
      "fallback_births": 0
    },
    {
      "day": 16,
      "births": 4,
      "couple_births": 4,
      "fallback_births": 0
    },
    {
      "day": 17,
      "births": 2,
      "couple_births": 2,
      "fallback_births": 0
    },
    {
      "day": 18,
      "births": 3,
      "couple_births": 3,
      "fallback_births": 0
    },
    {
      "day": 19,
      "births": 4,
      "couple_births": 4,
      "fallback_births": 0
    },
    {
      "day": 20,
      "births": 5,
      "couple_births": 5,
      "fallback_births": 0
    },
    {
      "day": 21,
      "births": 5,
      "couple_births": 5,
      "fallback_births": 0
    },
    {
      "day": 22,
      "births": 11,
      "couple_births": 11,
      "fallback_births": 0
    },
    {
      "day": 23,
      "births": 12,
      "couple_births": 12,
      "fallback_births": 0
    },
    {
      "day": 24,
      "births": 7,
      "couple_births": 7,
      "fallback_births": 0
    },
    {
      "day": 25,
      "births": 10,
      "couple_births": 10,
      "fallback_births": 0
    },
    {
      "day": 26,
      "births": 13,
      "couple_births": 13,
      "fallback_births": 0
    },
    {
      "day": 27,
      "births": 12,
      "couple_births": 12,
      "fallback_births": 0
    },
    {
      "day": 28,
      "births": 8,
      "couple_births": 8,
      "fallback_births": 0
    },
    {
      "day": 29,
      "births": 13,
      "couple_births": 13,
      "fallback_births": 0
    },
    {
      "day": 30,
      "births": 18,
      "couple_births": 18,
      "fallback_births": 0
    },
    {
      "day": 31,
      "births": 23,
      "couple_births": 22,
      "fallback_births": 1
    },
    {
      "day": 32,
      "births": 18,
      "couple_births": 18,
      "fallback_births": 0
    },
    {
      "day": 33,
      "births": 23,
      "couple_births": 22,
      "fallback_births": 1
    },
    {
      "day": 34,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 35,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 36,
      "births": 29,
      "couple_births": 29,
      "fallback_births": 0
    },
    {
      "day": 37,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 38,
      "births": 23,
      "couple_births": 23,
      "fallback_births": 0
    },
    {
      "day": 39,
      "births": 28,
      "couple_births": 28,
      "fallback_births": 0
    },
    {
      "day": 40,
      "births": 19,
      "couple_births": 19,
      "fallback_births": 0
    },
    {
      "day": 41,
      "births": 17,
      "couple_births": 17,
      "fallback_births": 0
    },
    {
      "day": 42,
      "births": 21,
      "couple_births": 21,
      "fallback_births": 0
    },
    {
      "day": 43,
      "births": 20,
      "couple_births": 20,
      "fallback_births": 0
    },
    {
      "day": 44,
      "births": 18,
      "couple_births": 17,
      "fallback_births": 1
    },
    {
      "day": 45,
      "births": 21,
      "couple_births": 21,
      "fallback_births": 0
    },
    {
      "day": 46,
      "births": 12,
      "couple_births": 12,
      "fallback_births": 0
    },
    {
      "day": 47,
      "births": 16,
      "couple_births": 16,
      "fallback_births": 0
    },
    {
      "day": 48,
      "births": 12,
      "couple_births": 11,
      "fallback_births": 1
    },
    {
      "day": 49,
      "births": 13,
      "couple_births": 13,
      "fallback_births": 0
    },
    {
      "day": 50,
      "births": 17,
      "couple_births": 17,
      "fallback_births": 0
    },
    {
      "day": 51,
      "births": 5,
      "couple_births": 5,
      "fallback_births": 0
    },
    {
      "day": 52,
      "births": 4,
      "couple_births": 4,
      "fallback_births": 0
    },
    {
      "day": 53,
      "births": 6,
      "couple_births": 6,
      "fallback_births": 0
    },
    {
      "day": 54,
      "births": 4,
      "couple_births": 4,
      "fallback_births": 0
    },
    {
      "day": 55,
      "births": 6,
      "couple_births": 6,
      "fallback_births": 0
    },
    {
      "day": 56,
      "births": 3,
      "couple_births": 3,
      "fallback_births": 0
    },
    {
      "day": 57,
      "births": 13,
      "couple_births": 13,
      "fallback_births": 0
    },
    {
      "day": 58,
      "births": 10,
      "couple_births": 10,
      "fallback_births": 0
    },
    {
      "day": 59,
      "births": 17,
      "couple_births": 17,
      "fallback_births": 0
    },
    {
      "day": 60,
      "births": 26,
      "couple_births": 26,
      "fallback_births": 0
    }
  ],
  "worlds": [
    {
      "seed": 18100,
      "day60_partnered_adults": 35.0,
      "day60_adults": 54.0,
      "births_from_day20": 56.0,
      "couple_births_from_day20": 56.0,
      "all_births": 78,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 656.0,
      "bedded_couple_nights": 658.0,
      "all_living_couple_nights": 658,
      "bedless_person_nights": 3,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 13.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 23,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 141,
      "unrelated_unordered_pairs": 2316,
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
      "seed": 18101,
      "day60_partnered_adults": 5.0,
      "day60_adults": 18.0,
      "births_from_day20": 22.0,
      "couple_births_from_day20": 22.0,
      "all_births": 45,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 455.0,
      "bedded_couple_nights": 455.0,
      "all_living_couple_nights": 455,
      "bedless_person_nights": 6,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 1,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 67,
      "unrelated_unordered_pairs": 1497,
      "excluded_close_kin_pairs": 135,
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
      "seed": 18102,
      "day60_partnered_adults": 11.0,
      "day60_adults": 19.0,
      "births_from_day20": 24.0,
      "couple_births_from_day20": 24.0,
      "all_births": 42,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 394.0,
      "bedded_couple_nights": 394.0,
      "all_living_couple_nights": 394,
      "bedless_person_nights": 3,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 62,
      "unrelated_unordered_pairs": 1311,
      "excluded_close_kin_pairs": 95,
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
      "seed": 18103,
      "day60_partnered_adults": 16.0,
      "day60_adults": 30.0,
      "births_from_day20": 38.0,
      "couple_births_from_day20": 38.0,
      "all_births": 54,
      "all_fallback_births": 8,
      "same_shelter_couple_nights": 502.0,
      "bedded_couple_nights": 502.0,
      "all_living_couple_nights": 502,
      "bedless_person_nights": 12,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 11.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 29,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 83,
      "unrelated_unordered_pairs": 2334,
      "excluded_close_kin_pairs": 112,
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
      "seed": 18104,
      "day60_partnered_adults": 5.0,
      "day60_adults": 21.0,
      "births_from_day20": 20.0,
      "couple_births_from_day20": 17.0,
      "all_births": 23,
      "all_fallback_births": 5,
      "same_shelter_couple_nights": 213.0,
      "bedded_couple_nights": 213.0,
      "all_living_couple_nights": 213,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 53,
      "unrelated_unordered_pairs": 592,
      "excluded_close_kin_pairs": 65,
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
      "seed": 18105,
      "day60_partnered_adults": 15.0,
      "day60_adults": 23.0,
      "births_from_day20": 25.0,
      "couple_births_from_day20": 25.0,
      "all_births": 56,
      "all_fallback_births": 9,
      "same_shelter_couple_nights": 550.0,
      "bedded_couple_nights": 550.0,
      "all_living_couple_nights": 550,
      "bedless_person_nights": 7,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 103,
      "unrelated_unordered_pairs": 2190,
      "excluded_close_kin_pairs": 153,
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
      "seed": 18106,
      "day60_partnered_adults": 12.0,
      "day60_adults": 21.0,
      "births_from_day20": 21.0,
      "couple_births_from_day20": 21.0,
      "all_births": 24,
      "all_fallback_births": 1,
      "same_shelter_couple_nights": 313.0,
      "bedded_couple_nights": 313.0,
      "all_living_couple_nights": 313,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 19,
      "unrelated_unordered_pairs": 647,
      "excluded_close_kin_pairs": 48,
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
      "seed": 18107,
      "day60_partnered_adults": 14.0,
      "day60_adults": 23.0,
      "births_from_day20": 30.0,
      "couple_births_from_day20": 30.0,
      "all_births": 47,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 558.0,
      "bedded_couple_nights": 560.0,
      "all_living_couple_nights": 560,
      "bedless_person_nights": 0,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 10.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 22,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 74,
      "unrelated_unordered_pairs": 1663,
      "excluded_close_kin_pairs": 94,
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
      "seed": 18108,
      "day60_partnered_adults": 18.0,
      "day60_adults": 30.0,
      "births_from_day20": 42.0,
      "couple_births_from_day20": 42.0,
      "all_births": 50,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 407.0,
      "bedded_couple_nights": 407.0,
      "all_living_couple_nights": 407,
      "bedless_person_nights": 3,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 9.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 27,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 61,
      "unrelated_unordered_pairs": 1890,
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
      "seed": 18109,
      "day60_partnered_adults": 33.0,
      "day60_adults": 68.0,
      "births_from_day20": 72.0,
      "couple_births_from_day20": 72.0,
      "all_births": 88,
      "all_fallback_births": 6,
      "same_shelter_couple_nights": 592.0,
      "bedded_couple_nights": 595.0,
      "all_living_couple_nights": 595,
      "bedless_person_nights": 1,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 10.0,
      "split_couples": 1.0,
      "completed_departure_households_kept": 31,
      "completed_departure_households_split": 1,
      "sibling_unordered_pairs": 231,
      "unrelated_unordered_pairs": 3139,
      "excluded_close_kin_pairs": 182,
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
      "seed": 18110,
      "day60_partnered_adults": 12.0,
      "day60_adults": 20.0,
      "births_from_day20": 22.0,
      "couple_births_from_day20": 22.0,
      "all_births": 33,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 387.0,
      "bedded_couple_nights": 387.0,
      "all_living_couple_nights": 387,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 35,
      "unrelated_unordered_pairs": 890,
      "excluded_close_kin_pairs": 74,
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
      "seed": 18111,
      "day60_partnered_adults": 23.0,
      "day60_adults": 54.0,
      "births_from_day20": 56.0,
      "couple_births_from_day20": 55.0,
      "all_births": 65,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 424.0,
      "bedded_couple_nights": 440.0,
      "all_living_couple_nights": 440,
      "bedless_person_nights": 1,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 7.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 21,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 383,
      "unrelated_unordered_pairs": 1498,
      "excluded_close_kin_pairs": 158,
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
      "seed": 18112,
      "day60_partnered_adults": 32.0,
      "day60_adults": 61.0,
      "births_from_day20": 67.0,
      "couple_births_from_day20": 67.0,
      "all_births": 88,
      "all_fallback_births": 7,
      "same_shelter_couple_nights": 672.0,
      "bedded_couple_nights": 672.0,
      "all_living_couple_nights": 672,
      "bedless_person_nights": 11,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 11.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 33,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 199,
      "unrelated_unordered_pairs": 2933,
      "excluded_close_kin_pairs": 215,
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
      "seed": 18113,
      "day60_partnered_adults": 29.0,
      "day60_adults": 48.0,
      "births_from_day20": 51.0,
      "couple_births_from_day20": 51.0,
      "all_births": 65,
      "all_fallback_births": 11,
      "same_shelter_couple_nights": 479.0,
      "bedded_couple_nights": 490.0,
      "all_living_couple_nights": 490,
      "bedless_person_nights": 2,
      "completed_fissions": 1,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 9.0,
      "split_couples": 2.0,
      "completed_departure_households_kept": 25,
      "completed_departure_households_split": 2,
      "sibling_unordered_pairs": 128,
      "unrelated_unordered_pairs": 1744,
      "excluded_close_kin_pairs": 155,
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
      "seed": 18114,
      "day60_partnered_adults": 14.0,
      "day60_adults": 19.0,
      "births_from_day20": 19.0,
      "couple_births_from_day20": 19.0,
      "all_births": 45,
      "all_fallback_births": 5,
      "same_shelter_couple_nights": 508.0,
      "bedded_couple_nights": 508.0,
      "all_living_couple_nights": 508,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 114,
      "unrelated_unordered_pairs": 1581,
      "excluded_close_kin_pairs": 90,
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
      "seed": 18115,
      "day60_partnered_adults": 18.0,
      "day60_adults": 30.0,
      "births_from_day20": 32.0,
      "couple_births_from_day20": 32.0,
      "all_births": 35,
      "all_fallback_births": 3,
      "same_shelter_couple_nights": 349.0,
      "bedded_couple_nights": 349.0,
      "all_living_couple_nights": 349,
      "bedless_person_nights": 0,
      "completed_fissions": 0,
      "cancelled_fissions": 0,
      "pending_fissions": 0,
      "departure_couples": 0.0,
      "split_couples": 0.0,
      "completed_departure_households_kept": 0,
      "completed_departure_households_split": 0,
      "sibling_unordered_pairs": 37,
      "unrelated_unordered_pairs": 1428,
      "excluded_close_kin_pairs": 82,
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
  "verdict": "PENDING",
  "source": null,
  "sha256": null,
  "payload": null,
  "error": "identity evidence was not supplied"
}
```

## performance

```json
{
  "verdict": "PENDING",
  "source": null,
  "sha256": null,
  "payload": null,
  "error": "performance evidence was not supplied"
}
```
