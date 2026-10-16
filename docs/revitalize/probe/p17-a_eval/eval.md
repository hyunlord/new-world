# Pass 17 Stage A — FAIL

Mode: evaluation.

| Rule | Verdict |
|---|---|
| R1 | PASS |
| R3-P | PASS |
| R3-NS | PASS |
| R3-RD | PASS |
| R2 | PASS |
| R4 | PASS |
| R5 | FAIL |

| Metric | Candidate | Baseline | Estimate | 95% CI | Valid bootstrap draws | Bar | Verdict |
|---|---:|---:|---:|---|---:|---|---|
| R1 | 0.116436 | 0.595078 | 0.116436 | [0.10948, 0.124192] | 2000/2000 | 0.10 <= pooled adult Idle share <= 0.45 | PASS |
| R1-baseline | undefined | undefined | 0.595078 | [0.578528, 0.610544] | 2000/2000 | reference only | PASS |
| R2-alive | 60.1875 | 59.5312 | 0.0110236 | [-0.00328975, 0.0283783] | 2000/2000 | relative change within +/-0.10 | PASS |
| R2-deaths | 0.90625 | 0.8125 | 0.09375 | [-0.125, 0.375] | 2000/2000 | non-old-age deaths difference <= +1.5/world | PASS |
| R2-shelters | 6.5625 | 6.375 | 1.02941 | [0.994381, 1.0765] | 2000/2000 | completed shelters ratio >= 0.90 | PASS |
| R2-food | 475.382 | 469.534 | 0.0124547 | [-0.00516335, 0.0310675] | 2000/2000 | noon food relative change within +/-0.15 | PASS |
| R2-wood | 116.592 | 113.965 | 0.0230494 | [-0.00428097, 0.0541416] | 2000/2000 | noon wood relative change within +/-0.15 | PASS |
| R2-social | 1.7195 | 1.73121 | 0.993232 | [0.98699, 0.999905] | 2000/2000 | pooled adult social rate ratio >= 0.90 | PASS |
| R3-P | undefined | undefined | 0.89793 | [0.881573, 0.915361] | 2000/2000 | rho >= 0.25; CI excludes zero | PASS |
| R3-NS | undefined | undefined | 0.491714 | [0.450427, 0.535105] | 2000/2000 | rho >= 0.25; CI excludes zero | PASS |
| R3-RD | undefined | undefined | 0.350282 | [0.325262, 0.378364] | 2000/2000 | rho >= 0.20; CI excludes zero | PASS |

## Methods

- bootstrap: 2,000 world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded and count reported
- R1: Pooled nonelder-adult hourly observations from 08:00 through 17:00 inclusive; Idle state 0
- R2: Paired worlds; equal-world health means; daily noon stores sum settlements; social primary pools completions/adult-days, mean world rate is sensitivity
- shelters: Final minus initial complete Shelter count, including daughter-camp boot shelters
- R3: Pooled-person Spearman average tie ranks; >=14,400 nonelder-adult ticks; zero-activity NS/RD shares=0; P requires >=3 normally completed activities
- activity_minutes: Consuming Activity ticks only, excluding travel; adult and all-life-stage totals shown separately

## Eligibility and sensitivity

```json
{
  "R2-shelters": {
    "definition": "final minus initial complete Shelter count, including daughter-camp boot shelters"
  },
  "R2-social": {
    "candidate_mean_world_rate": 1.6997733412133542,
    "baseline_mean_world_rate": 1.7100893954686132,
    "candidate_sensitivity_worlds": 32,
    "baseline_sensitivity_worlds": 32
  },
  "R3-P": {
    "eligible_people": 3928,
    "adult_10day_people": 3943,
    "zero_activity_people": 0,
    "no_place_redraws": 48885
  },
  "R3-NS": {
    "eligible_people": 3943,
    "adult_10day_people": 3943,
    "zero_activity_people": 0,
    "no_place_redraws": 48885
  },
  "R3-RD": {
    "eligible_people": 3943,
    "adult_10day_people": 3943,
    "zero_activity_people": 0,
    "no_place_redraws": 48885
  }
}
```

## game_v20

32 worlds × 120 days; seeds [17000, 17001, 17002, 17003, 17004, 17005, 17006, 17007, 17008, 17009, 17010, 17011, 17012, 17013, 17014, 17015, 17016, 17017, 17018, 17019, 17020, 17021, 17022, 17023, 17024, 17025, 17026, 17027, 17028, 17029, 17030, 17031].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 130956 | 0 | 0.0749488 | 0 | 0.925051 | 0 |
| 0 | child | 14044 | 0 | 0.00541156 | 0 | 0.994588 | 0 |
| 0 | elder | 31927 | 0 | 0.00704733 | 0 | 0.992953 | 0 |
| 60 | adult | 130969 | 1.52708e-05 | 0.0376272 | 0 | 0.962358 | 0 |
| 60 | child | 14052 | 0 | 0.00113863 | 0 | 0.998861 | 0 |
| 60 | elder | 31905 | 0 | 0.00263282 | 0 | 0.997367 | 0 |
| 120 | adult | 130983 | 0.000229037 | 0.0192926 | 0 | 0.980478 | 0 |
| 120 | child | 14060 | 0 | 0.000142248 | 0 | 0.999858 | 0 |
| 120 | elder | 31903 | 0.000282105 | 0.00570479 | 0 | 0.994013 | 0 |
| 180 | adult | 130992 | 0.0010764 | 0.0892726 | 0 | 0.909651 | 0 |
| 180 | child | 14064 | 0 | 7.11035e-05 | 0 | 0.999929 | 0 |
| 180 | elder | 31866 | 0.00106697 | 0.0921986 | 0 | 0.906734 | 0 |
| 240 | adult | 130981 | 0.00099251 | 0.113963 | 0 | 0.885044 | 0 |
| 240 | child | 14057 | 0 | 0.000284556 | 0 | 0.999715 | 0 |
| 240 | elder | 31818 | 0.00100572 | 0.115124 | 0 | 0.883871 | 0 |
| 300 | adult | 130979 | 0.00406172 | 0.165278 | 0 | 0.83066 | 0 |
| 300 | child | 14055 | 0.000142298 | 0.00106724 | 0 | 0.99879 | 0 |
| 300 | elder | 31782 | 0.00349254 | 0.166132 | 0 | 0.830376 | 0 |
| 360 | adult | 130975 | 0.799206 | 0.15151 | 0 | 0.0492842 | 0 |
| 360 | child | 14061 | 0.977313 | 0.0168551 | 0 | 0.00583173 | 0 |
| 360 | elder | 31725 | 0.806241 | 0.14569 | 0 | 0.0480693 | 0 |
| 420 | adult | 131057 | 0.136841 | 0.853461 | 0.00925551 | 0.000442556 | 0 |
| 420 | child | 14073 | 0.818944 | 0.180772 | 0 | 0.000284232 | 0 |
| 420 | elder | 31762 | 0.524621 | 0.466564 | 0.00840627 | 0.000409294 | 0 |
| 480 | adult | 130504 | 0.314374 | 0.643337 | 0.00301906 | 0.0392708 | 0 |
| 480 | child | 13894 | 0.941126 | 0.0382899 | 0 | 0.0205844 | 0 |
| 480 | elder | 31528 | 0.655132 | 0.32866 | 0.00209338 | 0.0141144 | 0 |
| 540 | adult | 130542 | 0.395559 | 0.597708 | 0.00571464 | 0.00101883 | 0 |
| 540 | child | 13922 | 0.975506 | 0.0244936 | 0 | 0 | 0 |
| 540 | elder | 31683 | 0.732191 | 0.263738 | 0.00344033 | 0.000631253 | 0 |
| 600 | adult | 130591 | 0.372307 | 0.596481 | 0.00179951 | 0.0294124 | 0 |
| 600 | child | 13925 | 0.962083 | 0.028079 | 0 | 0.00983842 | 0 |
| 600 | elder | 31795 | 0.767794 | 0.216418 | 0.00141532 | 0.0143733 | 0 |
| 660 | adult | 130619 | 0.549308 | 0.446053 | 0.00387386 | 0.000765585 | 0 |
| 660 | child | 13928 | 0.967547 | 0.0324526 | 0 | 0 | 0 |
| 660 | elder | 31749 | 0.914832 | 0.0842861 | 0.000881918 | 0 | 0 |
| 720 | adult | 130658 | 0.683594 | 0.279087 | 0.00130876 | 0.03601 | 0 |
| 720 | child | 13935 | 0.951991 | 0.0375314 | 0 | 0.0104772 | 0 |
| 720 | elder | 31745 | 0.891699 | 0.0905654 | 0.000598519 | 0.0171366 | 0 |
| 780 | adult | 130662 | 0.684973 | 0.311927 | 0.00289296 | 0.00020664 | 0 |
| 780 | child | 13944 | 0.965577 | 0.0343517 | 0 | 7.17154e-05 | 0 |
| 780 | elder | 31710 | 0.9152 | 0.0839798 | 0.000819931 | 0 | 0 |
| 840 | adult | 130664 | 0.763738 | 0.207104 | 0.00126278 | 0.027896 | 0 |
| 840 | child | 13952 | 0.951548 | 0.0362672 | 0 | 0.0121846 | 0 |
| 840 | elder | 31683 | 0.899031 | 0.0818104 | 0.000789067 | 0.0183695 | 0 |
| 900 | adult | 130709 | 0.717097 | 0.280111 | 0.00275421 | 3.82529e-05 | 0 |
| 900 | child | 13957 | 0.946622 | 0.0533066 | 0 | 7.16486e-05 | 0 |
| 900 | elder | 31628 | 0.921841 | 0.0769571 | 0.00120147 | 0 | 0 |
| 960 | adult | 130726 | 0.764806 | 0.207487 | 0.000826155 | 0.0268807 | 0 |
| 960 | child | 13959 | 0.939752 | 0.0482843 | 0 | 0.0119636 | 0 |
| 960 | elder | 31586 | 0.902393 | 0.0799088 | 0.000569873 | 0.0171278 | 0 |
| 1020 | adult | 130740 | 0.704176 | 0.293001 | 0.0028224 | 0 | 0 |
| 1020 | child | 13970 | 0.95748 | 0.0425197 | 0 | 0 | 0 |
| 1020 | elder | 31575 | 0.911956 | 0.0865558 | 0.00148852 | 0 | 0 |
| 1080 | adult | 130741 | 0.754438 | 0.218554 | 0.000726627 | 0.026281 | 0 |
| 1080 | child | 13975 | 0.946834 | 0.0410018 | 0 | 0.0121646 | 0 |
| 1080 | elder | 31541 | 0.892584 | 0.0874417 | 0.000538981 | 0.019435 | 0 |
| 1140 | adult | 130800 | 0.593402 | 0.399159 | 0.00742355 | 1.52905e-05 | 0 |
| 1140 | child | 13986 | 0.953024 | 0.046904 | 0 | 7.15001e-05 | 0 |
| 1140 | elder | 31552 | 0.911575 | 0.0856681 | 0.00275735 | 0 | 0 |
| 1200 | adult | 130809 | 0.710838 | 0.256832 | 0.00287442 | 0.0294552 | 0 |
| 1200 | child | 13995 | 0.952197 | 0.0393712 | 0 | 0.00843158 | 0 |
| 1200 | elder | 31697 | 0.886709 | 0.0926586 | 0.00214531 | 0.0184876 | 0 |
| 1260 | adult | 130849 | 0.340545 | 0.570451 | 0.00169661 | 0.0873067 | 0 |
| 1260 | child | 14025 | 0.470731 | 0.338538 | 0 | 0.190731 | 0 |
| 1260 | elder | 31891 | 0.464959 | 0.380766 | 0.000501709 | 0.153774 | 0 |
| 1320 | adult | 130904 | 0 | 0.355467 | 0 | 0.644533 | 0 |
| 1320 | child | 14029 | 0 | 0.057595 | 0 | 0.942405 | 0 |
| 1320 | elder | 31982 | 0 | 0.0899256 | 0 | 0.910074 | 0 |
| 1380 | adult | 130919 | 6.11065e-05 | 0.173504 | 0 | 0.826435 | 0 |
| 1380 | child | 14031 | 0 | 0.0173188 | 0 | 0.982681 | 0 |
| 1380 | elder | 31955 | 0 | 0.0243467 | 0 | 0.975653 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  }
]
```

### health_by_world

```json
[
  {
    "alive": 30.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 333.8833333333333,
    "wood": 77.91666666666667,
    "seed": 17000,
    "old_age_deaths": 58,
    "social_completed": 4559.0,
    "adult_days": 2621.7083333333335,
    "social_rate": 1.7389424834316045,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 57.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 509.775,
    "wood": 146.14166666666668,
    "seed": 17001,
    "old_age_deaths": 85,
    "social_completed": 7803.0,
    "adult_days": 4517.041666666667,
    "social_rate": 1.7274580523757253,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 105.0,
    "deaths": 2.0,
    "shelters": 15.0,
    "food": 896.4083333333333,
    "wood": 194.78333333333333,
    "seed": 17002,
    "old_age_deaths": 131,
    "social_completed": 13217.0,
    "adult_days": 7426.333333333333,
    "social_rate": 1.7797477445127698,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 46.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 351.775,
    "wood": 91.64166666666667,
    "seed": 17003,
    "old_age_deaths": 61,
    "social_completed": 4844.0,
    "adult_days": 2957.2083333333335,
    "social_rate": 1.638031364039846,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 133.0,
    "deaths": 2.0,
    "shelters": 16.0,
    "food": 691.5833333333334,
    "wood": 145.5,
    "seed": 17004,
    "old_age_deaths": 85,
    "social_completed": 11547.0,
    "adult_days": 6424.166666666667,
    "social_rate": 1.7974315734855364,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 46.0,
    "deaths": 3.0,
    "shelters": 5.0,
    "food": 324.575,
    "wood": 78.85,
    "seed": 17005,
    "old_age_deaths": 66,
    "social_completed": 4604.0,
    "adult_days": 2842.125,
    "social_rate": 1.61991467651845,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 96.0,
    "deaths": 3.0,
    "shelters": 13.0,
    "food": 540.8,
    "wood": 139.46666666666667,
    "seed": 17006,
    "old_age_deaths": 76,
    "social_completed": 8528.0,
    "adult_days": 5074.083333333333,
    "social_rate": 1.680697662960469,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 58.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 409.675,
    "wood": 134.93333333333334,
    "seed": 17007,
    "old_age_deaths": 86,
    "social_completed": 7312.0,
    "adult_days": 4380.0,
    "social_rate": 1.6694063926940639,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 120.0,
    "deaths": 0.0,
    "shelters": 15.0,
    "food": 688.1833333333333,
    "wood": 154.93333333333334,
    "seed": 17008,
    "old_age_deaths": 90,
    "social_completed": 10067.0,
    "adult_days": 6008.625,
    "social_rate": 1.6754249100251721,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 20.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 213.63333333333333,
    "wood": 78.01666666666667,
    "seed": 17009,
    "old_age_deaths": 43,
    "social_completed": 2917.0,
    "adult_days": 1778.625,
    "social_rate": 1.6400309227633705,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 2.0,
    "shelters": 2.0,
    "food": 308.125,
    "wood": 84.175,
    "seed": 17010,
    "old_age_deaths": 63,
    "social_completed": 5138.0,
    "adult_days": 2966.5833333333335,
    "social_rate": 1.7319587628865978,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 122.0,
    "deaths": 1.0,
    "shelters": 16.0,
    "food": 765.4083333333333,
    "wood": 171.3,
    "seed": 17011,
    "old_age_deaths": 110,
    "social_completed": 12450.0,
    "adult_days": 6924.041666666667,
    "social_rate": 1.7980827671699453,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 49.0,
    "deaths": 0.0,
    "shelters": 4.0,
    "food": 538.0416666666666,
    "wood": 90.69166666666666,
    "seed": 17012,
    "old_age_deaths": 79,
    "social_completed": 7222.0,
    "adult_days": 4237.25,
    "social_rate": 1.704407339666057,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 350.85833333333335,
    "wood": 91.86666666666666,
    "seed": 17013,
    "old_age_deaths": 60,
    "social_completed": 4599.0,
    "adult_days": 2789.7083333333335,
    "social_rate": 1.6485594372171521,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 369.675,
    "wood": 83.86666666666666,
    "seed": 17014,
    "old_age_deaths": 60,
    "social_completed": 4646.0,
    "adult_days": 2846.2916666666665,
    "social_rate": 1.632299336856436,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 70.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 492.775,
    "wood": 124.00833333333334,
    "seed": 17015,
    "old_age_deaths": 63,
    "social_completed": 8553.0,
    "adult_days": 4610.041666666667,
    "social_rate": 1.8552977648430509,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 54.0,
    "deaths": 2.0,
    "shelters": 6.0,
    "food": 367.31666666666666,
    "wood": 74.625,
    "seed": 17016,
    "old_age_deaths": 68,
    "social_completed": 5331.0,
    "adult_days": 3124.75,
    "social_rate": 1.7060564845187616,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 42.0,
    "deaths": 3.0,
    "shelters": 3.0,
    "food": 457.01666666666665,
    "wood": 76.96666666666667,
    "seed": 17017,
    "old_age_deaths": 68,
    "social_completed": 6387.0,
    "adult_days": 3600.5,
    "social_rate": 1.773920288848771,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 331.4583333333333,
    "wood": 77.10833333333333,
    "seed": 17018,
    "old_age_deaths": 66,
    "social_completed": 4677.0,
    "adult_days": 2860.25,
    "social_rate": 1.6351717507210908,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 22.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 242.675,
    "wood": 81.575,
    "seed": 17019,
    "old_age_deaths": 47,
    "social_completed": 3025.0,
    "adult_days": 1971.4166666666667,
    "social_rate": 1.5344295557340322,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 71.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 629.1833333333333,
    "wood": 163.125,
    "seed": 17020,
    "old_age_deaths": 108,
    "social_completed": 9580.0,
    "adult_days": 5484.0,
    "social_rate": 1.7469000729394601,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 359.85833333333335,
    "wood": 83.325,
    "seed": 17021,
    "old_age_deaths": 66,
    "social_completed": 4651.0,
    "adult_days": 2921.0833333333335,
    "social_rate": 1.5922174992154736,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 30.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 309.44166666666666,
    "wood": 89.58333333333333,
    "seed": 17022,
    "old_age_deaths": 60,
    "social_completed": 4888.0,
    "adult_days": 2636.0416666666665,
    "social_rate": 1.8542954240101164,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 116.0,
    "deaths": 0.0,
    "shelters": 14.0,
    "food": 800.525,
    "wood": 212.8,
    "seed": 17023,
    "old_age_deaths": 115,
    "social_completed": 13463.0,
    "adult_days": 7320.875,
    "social_rate": 1.8389878259087882,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 74.0,
    "deaths": 1.0,
    "shelters": 10.0,
    "food": 613.8083333333333,
    "wood": 135.575,
    "seed": 17024,
    "old_age_deaths": 107,
    "social_completed": 10047.0,
    "adult_days": 5514.416666666667,
    "social_rate": 1.821951551236909,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 92.0,
    "deaths": 0.0,
    "shelters": 12.0,
    "food": 535.3833333333333,
    "wood": 110.13333333333334,
    "seed": 17025,
    "old_age_deaths": 79,
    "social_completed": 8305.0,
    "adult_days": 4852.833333333333,
    "social_rate": 1.7113713638080847,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 370.2416666666667,
    "wood": 85.64166666666667,
    "seed": 17026,
    "old_age_deaths": 64,
    "social_completed": 4933.0,
    "adult_days": 2861.875,
    "social_rate": 1.7236951299410352,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 291.9166666666667,
    "wood": 75.66666666666667,
    "seed": 17027,
    "old_age_deaths": 54,
    "social_completed": 3552.0,
    "adult_days": 2337.125,
    "social_rate": 1.51981601326416,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 356.0416666666667,
    "wood": 86.5,
    "seed": 17028,
    "old_age_deaths": 60,
    "social_completed": 4492.0,
    "adult_days": 2921.5,
    "social_rate": 1.537566318671915,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 53.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 407.7,
    "wood": 148.45833333333334,
    "seed": 17029,
    "old_age_deaths": 82,
    "social_completed": 7229.0,
    "adult_days": 3988.0,
    "social_rate": 1.8126880641925778,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 67.0,
    "deaths": 4.0,
    "shelters": 7.0,
    "food": 495.80833333333334,
    "wood": 112.59166666666667,
    "seed": 17030,
    "old_age_deaths": 74,
    "social_completed": 8210.0,
    "adult_days": 4571.458333333333,
    "social_rate": 1.7959258077746891,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 72.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 671.5416666666666,
    "wood": 145.11666666666667,
    "seed": 17031,
    "old_age_deaths": 97,
    "social_completed": 9676.0,
    "adult_days": 5435.416666666667,
    "social_rate": 1.7801763127635108,
    "no_place_redraws": 0,
    "no_place_failures": 0
  }
]
```

## game_v21

32 worlds × 120 days; seeds [17000, 17001, 17002, 17003, 17004, 17005, 17006, 17007, 17008, 17009, 17010, 17011, 17012, 17013, 17014, 17015, 17016, 17017, 17018, 17019, 17020, 17021, 17022, 17023, 17024, 17025, 17026, 17027, 17028, 17029, 17030, 17031].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 133945 | 0 | 0.0800627 | 0 | 0.919937 | 0 |
| 0 | child | 14332 | 0 | 0.00523304 | 0 | 0.994767 | 0 |
| 0 | elder | 32318 | 0 | 0.00850919 | 0 | 0.991491 | 0 |
| 60 | adult | 133988 | 7.46335e-05 | 0.0399961 | 0 | 0.959929 | 0 |
| 60 | child | 14342 | 0 | 0.00125506 | 0 | 0.998745 | 0 |
| 60 | elder | 32295 | 0 | 0.00275584 | 0 | 0.997244 | 0 |
| 120 | adult | 134001 | 7.46263e-05 | 0.0208282 | 0 | 0.979097 | 0 |
| 120 | child | 14346 | 0 | 6.97058e-05 | 0 | 0.99993 | 0 |
| 120 | elder | 32293 | 3.09665e-05 | 0.00548106 | 0 | 0.994488 | 0 |
| 180 | adult | 133996 | 0.000470163 | 0.0916669 | 0 | 0.907863 | 0 |
| 180 | child | 14352 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 32270 | 0.000278897 | 0.0924388 | 0 | 0.907282 | 0 |
| 240 | adult | 133998 | 0.000402991 | 0.118241 | 0 | 0.881356 | 0 |
| 240 | child | 14344 | 0 | 6.97156e-05 | 0 | 0.99993 | 0 |
| 240 | elder | 32228 | 0.000217202 | 0.116607 | 0 | 0.883176 | 0 |
| 300 | adult | 133987 | 0.00370932 | 0.169173 | 0 | 0.827118 | 0 |
| 300 | child | 14343 | 0.000139441 | 0.000906365 | 0 | 0.998954 | 0 |
| 300 | elder | 32192 | 0.00304423 | 0.168085 | 0 | 0.828871 | 0 |
| 360 | adult | 133991 | 0.795233 | 0.155936 | 0 | 0.0488316 | 0 |
| 360 | child | 14350 | 0.98007 | 0.0140767 | 0 | 0.00585366 | 0 |
| 360 | elder | 32129 | 0.803106 | 0.149304 | 0 | 0.0475894 | 0 |
| 420 | adult | 134064 | 0.127693 | 0.859903 | 0.0100325 | 0.000320742 | 0.00205126 |
| 420 | child | 14358 | 0.799485 | 0.187213 | 0 | 0 | 0.0133027 |
| 420 | elder | 32170 | 0.513025 | 0.467734 | 0.010258 | 0.000248679 | 0.00873485 |
| 480 | adult | 133481 | 0.0480293 | 0.656138 | 0.00291427 | 0.0407249 | 0.252193 |
| 480 | child | 14207 | 0.00971352 | 0.0465264 | 0 | 0.0194974 | 0.924263 |
| 480 | elder | 31917 | 0.0102767 | 0.33844 | 0.00153523 | 0.0135351 | 0.636213 |
| 540 | adult | 133536 | 0.0856473 | 0.610292 | 0.00578121 | 0.000958543 | 0.297321 |
| 540 | child | 14228 | 0.12117 | 0.0373911 | 0 | 0 | 0.841439 |
| 540 | elder | 32063 | 0.115086 | 0.27736 | 0.00377382 | 0.000187132 | 0.603593 |
| 600 | adult | 133570 | 0.100936 | 0.607135 | 0.00194654 | 0.0311447 | 0.258838 |
| 600 | child | 14231 | 0.34453 | 0.0407561 | 0 | 0.00885391 | 0.60586 |
| 600 | elder | 32172 | 0.210804 | 0.233837 | 0.00142981 | 0.0181835 | 0.535745 |
| 660 | adult | 133596 | 0.137317 | 0.476354 | 0.00437887 | 0.000688643 | 0.381261 |
| 660 | child | 14233 | 0.186328 | 0.0404693 | 0 | 0 | 0.773203 |
| 660 | elder | 32125 | 0.14951 | 0.0961556 | 0.00177432 | 6.22568e-05 | 0.752498 |
| 720 | adult | 133615 | 0.157011 | 0.29017 | 0.00159413 | 0.0400853 | 0.51114 |
| 720 | child | 14240 | 0.257795 | 0.0436798 | 0 | 0.0105337 | 0.687992 |
| 720 | elder | 32119 | 0.194402 | 0.101715 | 0.000996295 | 0.0204863 | 0.6824 |
| 780 | adult | 133662 | 0.146923 | 0.334493 | 0.00323951 | 8.97787e-05 | 0.515255 |
| 780 | child | 14250 | 0.246807 | 0.0421053 | 0 | 0 | 0.711088 |
| 780 | elder | 32083 | 0.20581 | 0.0929464 | 0.00165197 | 0 | 0.699592 |
| 840 | adult | 133663 | 0.130395 | 0.225216 | 0.00131674 | 0.030921 | 0.612151 |
| 840 | child | 14255 | 0.23269 | 0.036759 | 0 | 0.00996142 | 0.720589 |
| 840 | elder | 32060 | 0.185745 | 0.0932314 | 0.000499064 | 0.0185278 | 0.701996 |
| 900 | adult | 133699 | 0.130158 | 0.313024 | 0.0029918 | 1.4959e-05 | 0.553811 |
| 900 | child | 14259 | 0.233537 | 0.0555439 | 0 | 0 | 0.710919 |
| 900 | elder | 32013 | 0.19895 | 0.0871833 | 0.00124949 | 0 | 0.712617 |
| 960 | adult | 133728 | 0.117283 | 0.230086 | 0.000904822 | 0.0291338 | 0.622592 |
| 960 | child | 14261 | 0.225861 | 0.0521001 | 0 | 0.0120609 | 0.709978 |
| 960 | elder | 31988 | 0.194448 | 0.090784 | 0.000468926 | 0.0191634 | 0.695136 |
| 1020 | adult | 133733 | 0.110549 | 0.328229 | 0.00281905 | 2.99103e-05 | 0.558374 |
| 1020 | child | 14268 | 0.246145 | 0.0434539 | 0 | 0 | 0.710401 |
| 1020 | elder | 31983 | 0.195666 | 0.0943939 | 0.0010318 | 0 | 0.708908 |
| 1080 | adult | 133739 | 0.728695 | 0.24073 | 0.000882316 | 0.0296922 | 0 |
| 1080 | child | 14276 | 0.945643 | 0.0416083 | 0 | 0.0127487 | 0 |
| 1080 | elder | 31942 | 0.881535 | 0.0977397 | 0.000751362 | 0.0199737 | 0 |
| 1140 | adult | 133793 | 0.575598 | 0.417301 | 0.00710052 | 0 | 0 |
| 1140 | child | 14282 | 0.951337 | 0.0485926 | 0 | 7.00182e-05 | 0 |
| 1140 | elder | 31951 | 0.901161 | 0.0958342 | 0.0030046 | 0 | 0 |
| 1200 | adult | 133817 | 0.683441 | 0.284493 | 0.00278739 | 0.0292788 | 0 |
| 1200 | child | 14295 | 0.948024 | 0.043022 | 0 | 0.00895418 | 0 |
| 1200 | elder | 32088 | 0.880765 | 0.0981052 | 0.00196335 | 0.019166 | 0 |
| 1260 | adult | 133872 | 0.333901 | 0.577567 | 0.00167324 | 0.0868591 | 0 |
| 1260 | child | 14319 | 0.471192 | 0.335778 | 0 | 0.19303 | 0 |
| 1260 | elder | 32277 | 0.462342 | 0.378257 | 0.000557673 | 0.158844 | 0 |
| 1320 | adult | 133915 | 0 | 0.36665 | 0 | 0.63335 | 0 |
| 1320 | child | 14323 | 0 | 0.0599735 | 0 | 0.940027 | 0 |
| 1320 | elder | 32364 | 0 | 0.09835 | 0 | 0.90165 | 0 |
| 1380 | adult | 133928 | 5.22669e-05 | 0.186339 | 0 | 0.813609 | 0 |
| 1380 | child | 14324 | 0 | 0.0188495 | 0 | 0.981151 | 0 |
| 1380 | elder | 32334 | 0 | 0.0292262 | 0 | 0.970774 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 4538882,
    "adult_minutes": 4536240,
    "all_life_stages_share": 0.13661121283060343,
    "adult_share": 0.2566390671686101,
    "all_life_stages_starts": 145542
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 2777583,
    "adult_minutes": 2775438,
    "all_life_stages_share": 0.08359965788219786,
    "adult_share": 0.15702119361063632,
    "all_life_stages_starts": 86792
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 3611652,
    "adult_minutes": 3608583,
    "all_life_stages_share": 0.10870345605857887,
    "adult_share": 0.20415660875978886,
    "all_life_stages_starts": 112942
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 10098420,
    "adult_minutes": 5000663,
    "all_life_stages_share": 0.30394211699551177,
    "adult_share": 0.2829139303794736,
    "all_life_stages_starts": 254056
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 1733520,
    "adult_minutes": 1731610,
    "all_life_stages_share": 0.05217546295896383,
    "adult_share": 0.09796632786180559,
    "all_life_stages_starts": 96736
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 5199424,
    "adult_minutes": 23029,
    "all_life_stages_share": 0.1564921975633091,
    "adult_share": 0.0013028722196854494,
    "all_life_stages_starts": 71507
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 5265332,
    "adult_minutes": 0,
    "all_life_stages_share": 0.15847589571083515,
    "adult_share": 0.0,
    "all_life_stages_starts": 95135
  }
]
```

### health_by_world

```json
[
  {
    "alive": 30.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 338.1666666666667,
    "wood": 78.16666666666667,
    "seed": 17000,
    "old_age_deaths": 58,
    "social_completed": 4489.0,
    "adult_days": 2621.7083333333335,
    "social_rate": 1.712242335627215,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 57.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 441.93333333333334,
    "wood": 141.68333333333334,
    "seed": 17001,
    "old_age_deaths": 85,
    "social_completed": 7776.0,
    "adult_days": 4515.291666666667,
    "social_rate": 1.7221478863491653,
    "no_place_redraws": 2218,
    "no_place_failures": 0
  },
  {
    "alive": 105.0,
    "deaths": 2.0,
    "shelters": 15.0,
    "food": 895.8166666666667,
    "wood": 195.04166666666666,
    "seed": 17002,
    "old_age_deaths": 131,
    "social_completed": 13039.0,
    "adult_days": 7426.333333333333,
    "social_rate": 1.7557789846941065,
    "no_place_redraws": 6410,
    "no_place_failures": 185
  },
  {
    "alive": 48.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 348.6166666666667,
    "wood": 92.88333333333334,
    "seed": 17003,
    "old_age_deaths": 61,
    "social_completed": 4729.0,
    "adult_days": 2958.5416666666665,
    "social_rate": 1.5984226462925146,
    "no_place_redraws": 51,
    "no_place_failures": 0
  },
  {
    "alive": 133.0,
    "deaths": 1.0,
    "shelters": 16.0,
    "food": 686.025,
    "wood": 145.15833333333333,
    "seed": 17004,
    "old_age_deaths": 81,
    "social_completed": 11208.0,
    "adult_days": 6368.5,
    "social_rate": 1.7599120672057784,
    "no_place_redraws": 3912,
    "no_place_failures": 1
  },
  {
    "alive": 50.0,
    "deaths": 3.0,
    "shelters": 5.0,
    "food": 332.475,
    "wood": 78.55833333333334,
    "seed": 17005,
    "old_age_deaths": 66,
    "social_completed": 4597.0,
    "adult_days": 2844.0,
    "social_rate": 1.6163853727144866,
    "no_place_redraws": 82,
    "no_place_failures": 0
  },
  {
    "alive": 97.0,
    "deaths": 2.0,
    "shelters": 13.0,
    "food": 601.6333333333333,
    "wood": 170.18333333333334,
    "seed": 17006,
    "old_age_deaths": 97,
    "social_completed": 10074.0,
    "adult_days": 6078.791666666667,
    "social_rate": 1.657237252469309,
    "no_place_redraws": 3945,
    "no_place_failures": 0
  },
  {
    "alive": 58.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 416.7,
    "wood": 129.38333333333333,
    "seed": 17007,
    "old_age_deaths": 86,
    "social_completed": 7321.0,
    "adult_days": 4386.833333333333,
    "social_rate": 1.668857566201892,
    "no_place_redraws": 2148,
    "no_place_failures": 0
  },
  {
    "alive": 131.0,
    "deaths": 0.0,
    "shelters": 18.0,
    "food": 795.2583333333333,
    "wood": 196.80833333333334,
    "seed": 17008,
    "old_age_deaths": 115,
    "social_completed": 12821.0,
    "adult_days": 7485.5,
    "social_rate": 1.7127780375392425,
    "no_place_redraws": 4464,
    "no_place_failures": 0
  },
  {
    "alive": 20.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 219.69166666666666,
    "wood": 78.25,
    "seed": 17009,
    "old_age_deaths": 43,
    "social_completed": 2943.0,
    "adult_days": 1777.875,
    "social_rate": 1.6553469732124024,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 313.25,
    "wood": 84.225,
    "seed": 17010,
    "old_age_deaths": 63,
    "social_completed": 5114.0,
    "adult_days": 2972.3333333333335,
    "social_rate": 1.7205338118201188,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 122.0,
    "deaths": 1.0,
    "shelters": 16.0,
    "food": 755.725,
    "wood": 172.475,
    "seed": 17011,
    "old_age_deaths": 110,
    "social_completed": 12354.0,
    "adult_days": 6908.333333333333,
    "social_rate": 1.7882750301568155,
    "no_place_redraws": 3947,
    "no_place_failures": 0
  },
  {
    "alive": 50.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 533.9916666666667,
    "wood": 89.275,
    "seed": 17012,
    "old_age_deaths": 79,
    "social_completed": 7074.0,
    "adult_days": 4237.25,
    "social_rate": 1.6694790253112277,
    "no_place_redraws": 104,
    "no_place_failures": 0
  },
  {
    "alive": 32.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 342.90833333333336,
    "wood": 89.13333333333334,
    "seed": 17013,
    "old_age_deaths": 60,
    "social_completed": 4602.0,
    "adult_days": 2789.7083333333335,
    "social_rate": 1.649634818454737,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 370.06666666666666,
    "wood": 85.05833333333334,
    "seed": 17014,
    "old_age_deaths": 60,
    "social_completed": 4607.0,
    "adult_days": 2846.2916666666665,
    "social_rate": 1.6185972976533796,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 70.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 498.34166666666664,
    "wood": 122.41666666666667,
    "seed": 17015,
    "old_age_deaths": 63,
    "social_completed": 8459.0,
    "adult_days": 4610.041666666667,
    "social_rate": 1.8349074936054446,
    "no_place_redraws": 1744,
    "no_place_failures": 0
  },
  {
    "alive": 62.0,
    "deaths": 2.0,
    "shelters": 7.0,
    "food": 398.94166666666666,
    "wood": 97.75833333333334,
    "seed": 17016,
    "old_age_deaths": 68,
    "social_completed": 6329.0,
    "adult_days": 3651.5416666666665,
    "social_rate": 1.7332405262617387,
    "no_place_redraws": 669,
    "no_place_failures": 0
  },
  {
    "alive": 42.0,
    "deaths": 3.0,
    "shelters": 3.0,
    "food": 460.4583333333333,
    "wood": 77.86666666666666,
    "seed": 17017,
    "old_age_deaths": 68,
    "social_completed": 6414.0,
    "adult_days": 3604.875,
    "social_rate": 1.7792572557994382,
    "no_place_redraws": 3,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 335.81666666666666,
    "wood": 78.49166666666666,
    "seed": 17018,
    "old_age_deaths": 66,
    "social_completed": 4529.0,
    "adult_days": 2860.25,
    "social_rate": 1.5834280220260466,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 22.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 244.575,
    "wood": 82.6,
    "seed": 17019,
    "old_age_deaths": 47,
    "social_completed": 2962.0,
    "adult_days": 1971.4166666666667,
    "social_rate": 1.5024728410195713,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 71.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 640.35,
    "wood": 158.99166666666667,
    "seed": 17020,
    "old_age_deaths": 108,
    "social_completed": 9792.0,
    "adult_days": 5484.041666666667,
    "social_rate": 1.7855444205535758,
    "no_place_redraws": 2056,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 359.475,
    "wood": 86.00833333333334,
    "seed": 17021,
    "old_age_deaths": 66,
    "social_completed": 4718.0,
    "adult_days": 2913.2083333333335,
    "social_rate": 1.619520288341891,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 30.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 309.26666666666665,
    "wood": 87.625,
    "seed": 17022,
    "old_age_deaths": 60,
    "social_completed": 4769.0,
    "adult_days": 2636.4166666666665,
    "social_rate": 1.8088946486708601,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 116.0,
    "deaths": 0.0,
    "shelters": 14.0,
    "food": 799.6083333333333,
    "wood": 212.84166666666667,
    "seed": 17023,
    "old_age_deaths": 115,
    "social_completed": 13391.0,
    "adult_days": 7320.875,
    "social_rate": 1.829152935953694,
    "no_place_redraws": 5994,
    "no_place_failures": 0
  },
  {
    "alive": 74.0,
    "deaths": 4.0,
    "shelters": 10.0,
    "food": 616.65,
    "wood": 135.76666666666668,
    "seed": 17024,
    "old_age_deaths": 105,
    "social_completed": 9947.0,
    "adult_days": 5546.625,
    "social_rate": 1.793342798548667,
    "no_place_redraws": 4127,
    "no_place_failures": 42
  },
  {
    "alive": 86.0,
    "deaths": 0.0,
    "shelters": 11.0,
    "food": 527.1666666666666,
    "wood": 111.875,
    "seed": 17025,
    "old_age_deaths": 79,
    "social_completed": 8198.0,
    "adult_days": 4816.5,
    "social_rate": 1.7020658154261392,
    "no_place_redraws": 1061,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 374.975,
    "wood": 84.70833333333333,
    "seed": 17026,
    "old_age_deaths": 64,
    "social_completed": 5063.0,
    "adult_days": 2861.875,
    "social_rate": 1.7691198951736187,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 293.53333333333336,
    "wood": 73.3,
    "seed": 17027,
    "old_age_deaths": 54,
    "social_completed": 3454.0,
    "adult_days": 2337.1666666666665,
    "social_rate": 1.477857805034586,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 367.175,
    "wood": 85.66666666666667,
    "seed": 17028,
    "old_age_deaths": 60,
    "social_completed": 4588.0,
    "adult_days": 2921.5833333333335,
    "social_rate": 1.5703813571408196,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 53.0,
    "deaths": 0.0,
    "shelters": 5.0,
    "food": 417.78333333333336,
    "wood": 149.25,
    "seed": 17029,
    "old_age_deaths": 82,
    "social_completed": 7439.0,
    "adult_days": 4046.0416666666665,
    "social_rate": 1.838587096441996,
    "no_place_redraws": 2038,
    "no_place_failures": 0
  },
  {
    "alive": 67.0,
    "deaths": 6.0,
    "shelters": 7.0,
    "food": 501.5833333333333,
    "wood": 113.875,
    "seed": 17030,
    "old_age_deaths": 74,
    "social_completed": 8055.0,
    "adult_days": 4569.708333333333,
    "social_rate": 1.7626945556335654,
    "no_place_redraws": 1216,
    "no_place_failures": 0
  },
  {
    "alive": 72.0,
    "deaths": 0.0,
    "shelters": 8.0,
    "food": 674.2666666666667,
    "wood": 145.61666666666667,
    "seed": 17031,
    "old_age_deaths": 97,
    "social_completed": 9222.0,
    "adult_days": 5435.416666666667,
    "social_rate": 1.6966500574932923,
    "no_place_redraws": 2696,
    "no_place_failures": 0
  }
]
```

## identity

```json
{
  "verdict": "PASS",
  "source": "/Users/rexxa/work/new-world-pass4/docs/revitalize/verification/p17-2_a_identity_observerfix/identity.json",
  "sha256": "692f27584155b429f2e105f6cd54d3d600c2a2333b47ce7793085dda3b59a2c6",
  "payload": {
    "schema_version": 1,
    "stage": "a",
    "verdict": "PASS",
    "passed": true,
    "errors": [],
    "baseline_commit": "adfdecfece79ab019964729319389fd602685fab",
    "candidate_commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
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
            "path": "../../probe/p17-2_a_identity_observerfix/i1_v19/summary.json",
            "sha256": "8ece19d7dc82fa2f5c4a16bf2f4a1cd00b249c17e841fcd3d53d5c5fcf483203"
          },
          {
            "path": "source_commit.txt",
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
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
      "I2": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i2_sha256.txt",
            "sha256": "1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84"
          },
          {
            "path": "../../probe/p17-2_a_identity_observerfix/i2_sha256.txt",
            "sha256": "1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84"
          },
          {
            "path": "source_commit.txt",
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
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
      "I3": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "../../probe/p17-0_base/i3_v16/summary.json",
            "sha256": "efed1ea8d506a023d2d6cc5c3e8d5832f3dfb48e1432a7ac699965dbe97ef6e1"
          },
          {
            "path": "../../probe/p17-2_a_identity_observerfix/i3_v16/summary.json",
            "sha256": "deb4619f130bcf9fe02d1050c19659b72e07f3de8b477a8ae06a50e786191ecb"
          },
          {
            "path": "source_commit.txt",
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
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
            "path": "../../probe/p17-2_a_identity_observerfix/i4_v20/summary.json",
            "sha256": "8006a6d085b81d65a54ed5ef56511acb631467d218b6fcc6431eaa13e569ee66"
          },
          {
            "path": "source_commit.txt",
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
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
      "v19_pins": {
        "passed": true,
        "exit_code": 0,
        "evidence": [
          {
            "path": "source_commit.txt",
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
          },
          {
            "path": "v19_pins.txt",
            "sha256": "07d1bdeec99f0e306e999e195960c7724c3d75bc2e800d826b142c4047fc5493"
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
            "sha256": "5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa"
          },
          {
            "path": "binaries.sha256",
            "sha256": "a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70"
          },
          {
            "path": "binary_directory.txt",
            "sha256": "387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32"
          },
          {
            "path": "binaries_verified.txt",
            "sha256": "f0d03438aa186c7dd564f4e7380efb006e36374004b6f56d1045fe9736231275"
          },
          {
            "path": "RUNLOG.txt",
            "sha256": "6b5f2f47ac1e8ac8b543ae215905f9a8692fe3002eb8eb5aadb72c79dbba38ad"
          },
          {
            "path": "raw_sha256.txt",
            "sha256": "2d1d12e82dd69399d4245454846315b06e2c7c2be82dea640b81c1104ac853a5"
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
  "verdict": "FAIL",
  "source": "/Users/rexxa/work/new-world-pass4/docs/revitalize/perf/p17-a/perf.json",
  "sha256": "4c88573122101832e3c526d8e99d2476bc4d52f3b908d9ab7cf7d04117f8150f",
  "payload": {
    "schema_version": 1,
    "stage": "a",
    "verdict": "FAIL",
    "passed": false,
    "errors": [],
    "checks": {
      "mean_relative": {
        "value": 0.33040145588540026,
        "limit": 0.15,
        "operator": "<=",
        "passed": false
      },
      "mean_absolute": {
        "value": 0.4698833333333333,
        "limit": 4.0,
        "operator": "<=",
        "passed": true
      },
      "p999_relative": {
        "value": 3.3986371379897786,
        "limit": 0.35,
        "operator": "<=",
        "passed": false
      },
      "slope": {
        "value": 1.0013160279134086,
        "limit": 1.7,
        "operator": "<",
        "passed": true
      }
    },
    "repeats": {
      "old": [
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1/stress_stress_v20_s15100_n500_side64_fed.json",
          "sha256": "a96118800d530c26dce375d9a90bc92e5215593a845920d695bb0f9aecfeb8ca",
          "mean_ms": 0.36824855072463764,
          "p999_ms": 1.267,
          "max_ms": 2.607,
          "alive": 500.0,
          "decision_us": 244.37835942028985,
          "decision_share": 0.6636234112514586,
          "fingerprint": "0fa49639a76b0281",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2/stress_stress_v20_s15100_n500_side64_fed.json",
          "sha256": "d5684802df474541941e7313f99bc31b36b389781553854e4fa6de78484bbfe9",
          "mean_ms": 0.35226231884057974,
          "p999_ms": 0.567,
          "max_ms": 0.834,
          "alive": 500.0,
          "decision_us": 232.32263913043477,
          "decision_share": 0.6595160103842246,
          "fingerprint": "0fa49639a76b0281",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3/stress_stress_v20_s15100_n500_side64_fed.json",
          "sha256": "2c430f7fb57adc384acd8b4820cdd92fd23f16b6ac99a90061b286f4179a0fc7",
          "mean_ms": 0.3531891304347826,
          "p999_ms": 0.587,
          "max_ms": 0.849,
          "alive": 500.0,
          "decision_us": 233.18183115942028,
          "decision_share": 0.6602180278661718,
          "fingerprint": "0fa49639a76b0281",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ],
      "new": [
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "f0add76cb2142d4601c7d51acfb8f1aee3b77da2c5fe05fbd44500bc0b838f1c",
          "mean_ms": 0.4701536231884058,
          "p999_ms": 2.646,
          "max_ms": 3.114,
          "alive": 500.0,
          "decision_us": 342.7775869565217,
          "decision_share": 0.7290757106835262,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "5ca518ccdec62c5f1e32527b4c44e09c5934a2cda1b7e0003448a88bafc38d70",
          "mean_ms": 0.46749855072463764,
          "p999_ms": 2.53,
          "max_ms": 3.028,
          "alive": 500.0,
          "decision_us": 340.66000942028984,
          "decision_share": 0.7286867710974846,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "5fe914ece0fd2f11cfaf3e99f75d58f5f519a7815d3e008cbff8146f0a393af3",
          "mean_ms": 0.4698833333333333,
          "p999_ms": 2.582,
          "max_ms": 3.207,
          "alive": 500.0,
          "decision_us": 342.6569405797101,
          "decision_share": 0.729238336990835,
          "fingerprint": "14458e1fa711de0c",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ]
    },
    "medians": {
      "old": {
        "mean_ms": 0.3531891304347826,
        "p999_ms": 0.587
      },
      "new": {
        "mean_ms": 0.4698833333333333,
        "p999_ms": 2.582
      }
    },
    "slopes": {
      "old": {
        "b": 1.0516753956326028,
        "a": -1.1488279535971158,
        "n": 3,
        "se": 0.04053454003967519,
        "ci": [
          0.5366352310018206,
          1.5667155602633849
        ],
        "r2": 0.9985166542748841
      },
      "new": {
        "b": 1.0013160279134086,
        "a": -0.3899074213967184,
        "n": 3,
        "se": 0.053493246070234104,
        "ci": [
          0.3216198913424435,
          1.6810121644843736
        ],
        "r2": 0.9971541116780275
      },
      "reference": 1.06,
      "old_points": [
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json",
          "sha256": "8d7d25df03bb4a92fd0a303a4e70e25a6edc37c96176f7cd6e1e64477f90f916",
          "mean_ms": 0.07218014184397163,
          "p999_ms": 0.172,
          "max_ms": 0.191,
          "alive": 100.0,
          "decision_us": 40.87577907801419,
          "decision_share": 0.5663022825083027,
          "fingerprint": "e44f8265b2219b8e",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json",
          "sha256": "6a750daa10dc741441ae81e3a9891697929095538260f65a4da1202b3e66ea19",
          "mean_ms": 0.3370964539007092,
          "p999_ms": 0.656,
          "max_ms": 0.987,
          "alive": 500.0,
          "decision_us": 207.06565886524822,
          "decision_share": 0.6142623467829146,
          "fingerprint": "10746baa35b729ed",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json",
          "sha256": "4d84647b7594dfc032d2da8834f5f38db27e595e2183cfa6dd347f4e877cc347",
          "mean_ms": 0.7230078014184397,
          "p999_ms": 1.369,
          "max_ms": 1.75,
          "alive": 1000.0,
          "decision_us": 470.3858843971631,
          "decision_share": 0.6505958628307082,
          "fingerprint": "603177818e8fb210",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ],
      "new_points": [
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
          "sha256": "e325de81268b32f44ea53f2c0aa8bc68b8bd03d3546b21f19d8e3cdcfcd9326d",
          "mean_ms": 0.1033372340425532,
          "p999_ms": 0.258,
          "max_ms": 0.323,
          "alive": 100.0,
          "decision_us": 69.59764219858155,
          "decision_share": 0.6735001458421268,
          "fingerprint": "bea6a0b9ada4edc7",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
          "sha256": "4d20c5c4bf3e9eda927b002d601c62f9d5738c3984c86969e3046d9178ab137b",
          "mean_ms": 0.45132801418439716,
          "p999_ms": 2.737,
          "max_ms": 3.079,
          "alive": 500.0,
          "decision_us": 317.9090875886525,
          "decision_share": 0.704385895839308,
          "fingerprint": "7ce50f96939b36b6",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        },
        {
          "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
          "sha256": "5ad1d301c1aa8bdb440e5b0d4fb00b7dcb5ccabdf61955eb32d759648f009c93",
          "mean_ms": 0.9876166666666667,
          "p999_ms": 5.574,
          "max_ms": 6.175,
          "alive": 1000.0,
          "decision_us": 718.1260308510638,
          "decision_share": 0.7271303280804602,
          "fingerprint": "fbf486f2e08c29ff",
          "machine": {
            "cpu": "Apple M4 Max",
            "memsize": "38654705664",
            "ncpu": "14",
            "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
            "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
          }
        }
      ]
    },
    "quick": {
      "base": {
        "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
        "sha256": "ca79b3e496236651e472ed4972ec4c561d5dcd6794fca3bcdb2f94053a410a75",
        "mean_ms": 0.3047492753623188,
        "p999_ms": 0.597,
        "max_ms": 0.804,
        "alive": 500.0,
        "decision_us": 184.99849347826088,
        "decision_share": 0.6070514630701408,
        "fingerprint": "54ac4928a770f238",
        "machine": {
          "cpu": "Apple M4 Max",
          "memsize": "38654705664",
          "ncpu": "14",
          "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
          "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
        }
      },
      "final": {
        "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
        "sha256": "39c0ac7d4607224d43999918e0399cf42f27deb6610170f42ce7f6e5a280ee04",
        "mean_ms": 0.30321884057971016,
        "p999_ms": 0.609,
        "max_ms": 0.816,
        "alive": 500.0,
        "decision_us": 184.470518115942,
        "decision_share": 0.6083741952289684,
        "fingerprint": "54ac4928a770f238",
        "machine": {
          "cpu": "Apple M4 Max",
          "memsize": "38654705664",
          "ncpu": "14",
          "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
          "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
        }
      }
    },
    "provenance": {
      "schema_version": 1,
      "stage": "a",
      "status": 0,
      "builds": {
        "final": {
          "path": "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
          "commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
          "sha256": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38"
        },
        "baseline": {
          "path": "/Users/rexxa/work/p17-data/base-bin/worldsim_perf",
          "commit": "adfdecfece79ab019964729319389fd602685fab",
          "sha256": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded"
        }
      },
      "quiet": {
        "owner": "pass17-sim (Codex, new-world-pass4)",
        "pid": "24639",
        "purpose": "Stage A corrected-build performance",
        "started": "2026-10-15T08:20:08Z",
        "log": "/Users/rexxa/work/p17-data/a-perf-observerfix/quiet.log",
        "prefix_bytes": 178,
        "prefix_sha256": "74be75ba1be87ded142d6dd7920997e90a59517b636e8145e8ea240dca45b874",
        "contended": false
      },
      "commands": {
        "file": "commands.jsonl",
        "sha256": "2ee5b361c678b83237518fdadf8ac4da20494b97879e46132379f7adcc5a292f"
      },
      "runlog": {
        "file": "RUNLOG.txt",
        "sha256": "3c03c15fba1b03bf66b14b7ac0745947ba556132a82828018a0a5ab842375991"
      },
      "indexes": {
        "quick_final/runs.json": "1f1c5433072e6935567c92f64ed9e2a39a90242a5ef710cbdcfd529f7d43a493",
        "b_game_v21/runs.json": "c4864a3d924e60836e322d4d05c6579133b8ef644793b7a09b8640b6e59d17ad",
        "b_game_v20/runs.json": "f00b469612661cc5f79db3d66f4e14f6f4d42a8342576224fd58f138f0e728be",
        "quick_base/runs.json": "8f91108365cfe0a4f8f5b2b89be53333ed64833120df6a905ade959c0a75a8f7",
        "a/r1/runs.json": "f14ca8a99273a0d5a04558b27e1cc211eec5f41e76bf472ad659ee60dc178adc",
        "a/r3/runs.json": "29373f16c285309c9e2b5962c82ae3032b197d8e528898083282fd70e2fd61fb",
        "a/r2/runs.json": "41d3f20c15c0b64d44fb95260df887bf0851c3aab2bf033d90c757b6df4a1476"
      },
      "manifest_sha256": "8eb105e3c79faa9ce22c2a945b16f31bc49ed7401633ce051041dd18c5a82388",
      "quiet_log_sha256": "bffb2e993061681b4749e4508a214f9bff6ae30e670e28591e7de19c9facd8f9",
      "input_directory": "/Users/rexxa/work/p17-data/a-perf-observerfix",
      "command_records": [
        {
          "index": 0,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "game_v20",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1"
          ],
          "started": 1792052408.3640478,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052409.906317,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r1/stress_stress_v20_s15100_n500_side64_fed.json": "a96118800d530c26dce375d9a90bc92e5215593a845920d695bb0f9aecfeb8ca"
          }
        },
        {
          "index": 1,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1"
          ],
          "started": 1792052409.9684951,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052410.7348511,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "f0add76cb2142d4601c7d51acfb8f1aee3b77da2c5fe05fbd44500bc0b838f1c"
          }
        },
        {
          "index": 2,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "game_v20",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2"
          ],
          "started": 1792052410.796533,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052411.38445,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r2/stress_stress_v20_s15100_n500_side64_fed.json": "d5684802df474541941e7313f99bc31b36b389781553854e4fa6de78484bbfe9"
          }
        },
        {
          "index": 3,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2"
          ],
          "started": 1792052411.445913,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052412.2049751,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "5ca518ccdec62c5f1e32527b4c44e09c5934a2cda1b7e0003448a88bafc38d70"
          }
        },
        {
          "index": 4,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "game_v20",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3"
          ],
          "started": 1792052412.267819,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052412.8559191,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r3/stress_stress_v20_s15100_n500_side64_fed.json": "2c430f7fb57adc384acd8b4820cdd92fd23f16b6ac99a90061b286f4179a0fc7"
          }
        },
        {
          "index": 5,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3"
          ],
          "started": 1792052412.9158552,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052413.677444,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "5fe914ece0fd2f11cfaf3e99f75d58f5f519a7815d3e008cbff8146f0a393af3"
          }
        },
        {
          "index": 6,
          "build": "baseline",
          "argv": [
            "/Users/rexxa/work/p17-data/base-bin/worldsim_perf",
            "quick",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_base"
          ],
          "started": 1792052413.738869,
          "binary_sha256_before": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded",
          "finished": 1792052414.536443,
          "exit_code": 0,
          "binary_sha256_after": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded",
          "outputs": {
            "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "ca79b3e496236651e472ed4972ec4c561d5dcd6794fca3bcdb2f94053a410a75"
          }
        },
        {
          "index": 7,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
            "quick",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_final"
          ],
          "started": 1792052414.597124,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052415.112693,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "39c0ac7d4607224d43999918e0399cf42f27deb6610170f42ce7f6e5a280ee04"
          }
        },
        {
          "index": 8,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21"
          ],
          "started": 1792052415.173277,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052419.733844,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "e325de81268b32f44ea53f2c0aa8bc68b8bd03d3546b21f19d8e3cdcfcd9326d",
            "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "4d20c5c4bf3e9eda927b002d601c62f9d5738c3984c86969e3046d9178ab137b",
            "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "5ad1d301c1aa8bdb440e5b0d4fb00b7dcb5ccabdf61955eb32d759648f009c93"
          }
        },
        {
          "index": 9,
          "build": "final",
          "argv": [
            "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
            "game_v20",
            "--out",
            "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20"
          ],
          "started": 1792052419.796577,
          "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "finished": 1792052423.164577,
          "exit_code": 0,
          "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
          "outputs": {
            "b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json": "8d7d25df03bb4a92fd0a303a4e70e25a6edc37c96176f7cd6e1e64477f90f916",
            "b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json": "6a750daa10dc741441ae81e3a9891697929095538260f65a4da1202b3e66ea19",
            "b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json": "4d84647b7594dfc032d2da8834f5f38db27e595e2183cfa6dd347f4e877cc347"
          }
        }
      ]
    }
  },
  "error": null
}
```
